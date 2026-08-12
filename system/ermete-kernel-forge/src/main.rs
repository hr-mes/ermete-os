pub mod builder;
pub mod hardware;
pub mod ostree_hook;
pub mod hw_scanner;
pub mod fal_client;

use ostree_hook::OstreeHookManager;
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tracing::info;
use zbus::interface;
use zbus::zvariant::{OwnedValue, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitSubject {
    pub kind: String,
    pub details: HashMap<String, OwnedValue>,
}

impl PolkitSubject {
    pub fn system_bus_name(name: impl Into<String>) -> Self {
        let mut details = HashMap::new();
        let val: Value = Value::from(name.into());
        if let Ok(owned) = val.try_into() {
            details.insert("name".to_string(), owned);
        }
        Self {
            kind: "system-bus-name".to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitAuthorizationResult {
    pub is_authorized: bool,
    pub is_challenge: bool,
    pub details: HashMap<String, String>,
}

#[zbus::proxy(
    interface = "org.freedesktop.PolicyKit1.Authority",
    default_service = "org.freedesktop.PolicyKit1",
    default_path = "/org/freedesktop/PolicyKit1/Authority"
)]
pub trait PolicyKitAuthority {
    fn check_authorization(
        &self,
        subject: &PolkitSubject,
        action_id: &str,
        details: &HashMap<&str, &str>,
        flags: u32,
        cancellation_id: &str,
    ) -> zbus::Result<PolkitAuthorizationResult>;
}

pub async fn check_polkit_auth_zbus(
    conn: &zbus::Connection,
    sender: &str,
    action_id: &str,
    allow_user_interaction: bool,
) -> Result<bool, zbus::Error> {
    if let Ok(creds) = conn.peer_creds().await {
        if creds.unix_user_id() == Some(0) {
            return Ok(true);
        }
    }

    let proxy = PolicyKitAuthorityProxy::new(conn).await?;
    let subject = PolkitSubject::system_bus_name(sender);
    let details = HashMap::<&str, &str>::new();
    let flags = if allow_user_interaction { 1u32 } else { 0u32 };

    let result = proxy
        .check_authorization(&subject, action_id, &details, flags, "")
        .await?;

    Ok(result.is_authorized)
}

pub struct KernelForgeDaemon {
    pub hook_manager: OstreeHookManager,
}

impl KernelForgeDaemon {
    pub fn new() -> Self {
        Self {
            hook_manager: OstreeHookManager::new(),
        }
    }
}

impl Default for KernelForgeDaemon {
    fn default() -> Self {
        Self::new()
    }
}

#[interface(name = "org.ermete.KernelForge")]
impl KernelForgeDaemon {
    /// D-Bus Method: ForgeHardwareTailoredKernel
    /// Extracts local kernel sources, detects CPU/hardware flags (-march=native),
    /// executes Gentoo-style LTO/AutoFDO kernel build with driver pruning,
    /// and forges a super-optimized Unified Kernel Image (UKI).
    async fn forge_hardware_tailored_kernel(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: ForgeHardwareTailoredKernel");

        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.kernelforge.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for KernelForge".into()));
        }

        // 1. Generate hardware profile hash
        let hash = hw_scanner::generate_hardware_hash()
            .map_err(|e| zbus::fdo::Error::Failed(format!("Hardware scanning failed: {}", e)))?;
        info!("Calculated Hardware Hash: {}", hash);

        // 2. Initialize FAL Client with GitHub Token if available
        let token = std::env::var("GITHUB_TOKEN")
            .ok()
            .or_else(|| std::fs::read_to_string("/home/ermete/.github_token").ok().map(|s| s.trim().to_string()));
        let fal_client = fal_client::FalClient::new(token);

        // 3. Query Global Cache (GHCR)
        let cache_hit = fal_client.check_global_cache(&hash)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Global Cache check failed: {}", e)))?;

        // 4. On Cache Hit -> Pull & Deploy precompiled UKI without compiling
        if cache_hit {
            info!("Global Cache Hit for hash {}. Pulling and deploying pre-compiled Kernel...", hash);
            fal_client.pull_and_deploy(&hash)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Pull and deploy failed: {}", e)))?;

            return Ok(format!(
                "✅ GLOBAL CACHE HIT: Pre-compiled UKI for hardware hash {} pulled and deployed successfully (0s compile time).",
                hash
            ));
        }

        // 5. On Cache Miss -> Switch between Remote Cloud Forge or Local Build
        let use_remote_forge = std::env::var("ERMETE_USE_REMOTE_FORGE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if use_remote_forge {
            info!("Global Cache Miss for hash {}. Innesco compilazione remota (Remote Cloud Forge)...", hash);
            fal_client.trigger_remote_build(&hash)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Remote build trigger failed: {}", e)))?;

            Ok(format!(
                "❌ GLOBAL CACHE MISS: Remote Cloud Forge workflow triggered on GitHub Actions for Hardware Hash {}.",
                hash
            ))
        } else {
            info!("Global Cache Miss for hash {}. Innesco compilazione locale (Gentoo-Style Local Forge)...", hash);
            match builder::run_kernel_forge().await {
                Ok(res) => Ok(format!(
                    "❌ GLOBAL CACHE MISS: Local Kernel compilation completed for Hardware Hash {}.\n{}",
                    hash, res.message
                )),
                Err(e) => Err(zbus::fdo::Error::Failed(format!("Local Kernel Forge Failed: {}", e))),
            }
        }
    }

    /// D-Bus Method: RegisterOstreeHook
    /// Registers Ermete Kernel Forge as an OSTree/bootc transaction hook.
    async fn register_ostree_hook(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: RegisterOstreeHook");
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.kernelforge.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }

        match self.hook_manager.register_transaction_hooks().await {
            Ok(msg) => Ok(msg),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Hook Registration Failed: {}", e))),
        }
    }

    /// D-Bus Method: InterceptOstreeUpdate
    /// Intercepts OSTree/bootc kernel updates in Hybrid Rolling-Forge mode.
    /// Triggers local hardware re-compilation (-march=native), injects new UKI into deployment,
    /// and permits reboot only after successful staging. Performs automatic rollback on failure.
    async fn intercept_ostree_update(
        &self,
        upstream_kernel_version: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: InterceptOstreeUpdate for version {}", upstream_kernel_version);
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.kernelforge.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }

        let kver = if upstream_kernel_version.is_empty() {
            None
        } else {
            Some(upstream_kernel_version.as_str())
        };
        match self.hook_manager.handle_ostree_update_transaction(kver).await {
            Ok(res) => Ok(res.message),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Transaction Interception Failed: {}", e))),
        }
    }

    /// D-Bus Method: GetOstreeTransactionStatus
    /// Returns current state of the OSTree/bootc Hybrid Rolling-Forge hook.
    async fn get_ostree_transaction_status(&self) -> zbus::fdo::Result<String> {
        let st = self.hook_manager.get_status().await;
        serde_json::to_string(&st)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization Error: {}", e)))
    }

    /// D-Bus Method: TriggerOstreeRollback
    /// Triggers an immediate rollback of the OSTree/bootc deployment.
    async fn trigger_ostree_rollback(
        &self,
        reason: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: TriggerOstreeRollback. Reason: {}", reason);
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.kernelforge.manage", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }

        match self.hook_manager.rollback_transaction(reason).await {
            Ok(res) => Ok(res.message),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Rollback Failed: {}", e))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    info!("==========================================================================");
    info!("🧬 Ermete OS Gentoo-Style Hardware-Tailored Kernel Forge Daemon Starting");
    info!("   Modello Ibrido Rolling-Forge & OSTree/bootc Hooks: ACTIVE");
    info!("   D-Bus Service: org.ermete.KernelForge");
    info!("   Object Path:   /org/ermete/KernelForge");
    info!("==========================================================================");

    let daemon = KernelForgeDaemon::new();

    // Automatically register OSTree/bootc transaction hook on daemon launch
    if let Err(e) = daemon.hook_manager.register_transaction_hooks().await {
        info!("Warning: Initial OSTree transaction hook registration note: {}", e);
    }
    
    // Try registering on session bus first, or fallback gracefully to system bus
    let conn_builder = match zbus::connection::Builder::session() {
        Ok(b) => b,
        Err(_) => zbus::connection::Builder::system()?,
    };

    let _conn = conn_builder
        .name("org.ermete.KernelForge")?
        .serve_at("/org/ermete/KernelForge", daemon)?
        .build()
        .await?;

    info!("🚀 D-Bus Service org.ermete.KernelForge successfully exported and listening!");

    signal::ctrl_c().await?;
    info!("Clean shutdown of Ermete Kernel Forge daemon.");
    Ok(())
}

