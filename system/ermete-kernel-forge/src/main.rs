pub mod builder;
pub mod hardware;
pub mod ostree_hook;
pub mod hw_scanner;

use ostree_hook::OstreeHookManager;
use std::error::Error;
use tokio::signal;
use tracing::info;
use zbus::interface;

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
    async fn forge_hardware_tailored_kernel(&self) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: ForgeHardwareTailoredKernel");
        match builder::run_kernel_forge().await {
            Ok(res) => Ok(res.message),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Kernel Forge Failed: {}", e))),
        }
    }

    /// D-Bus Method: RegisterOstreeHook
    /// Registers Ermete Kernel Forge as an OSTree/bootc transaction hook.
    async fn register_ostree_hook(&self) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: RegisterOstreeHook");
        match self.hook_manager.register_transaction_hooks().await {
            Ok(msg) => Ok(msg),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Hook Registration Failed: {}", e))),
        }
    }

    /// D-Bus Method: InterceptOstreeUpdate
    /// Intercepts OSTree/bootc kernel updates in Hybrid Rolling-Forge mode.
    /// Triggers local hardware re-compilation (-march=native), injects new UKI into deployment,
    /// and permits reboot only after successful staging. Performs automatic rollback on failure.
    async fn intercept_ostree_update(&self, upstream_kernel_version: String) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: InterceptOstreeUpdate for version {}", upstream_kernel_version);
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
    async fn trigger_ostree_rollback(&self, reason: String) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: TriggerOstreeRollback. Reason: {}", reason);
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

