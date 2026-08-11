use tokio::process::Command;
use tracing::{error, info};
use zbus::{connection, interface};
use std::collections::HashMap;
use std::future::pending;
use serde::{Deserialize, Serialize};
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
    if let Ok(creds) = conn.peer_credentials().await {
        if creds.uid() == Some(0) {
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

struct LivePatcher;

#[interface(name = "os.ermete.LivePatcher1")]
impl LivePatcher {
    async fn apply_kernel_patch(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
        patch_path: &str,
    ) -> zbus::fdo::Result<String> {
        info!("Received request to apply kernel live patch: {}", patch_path);

        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::AccessDenied("Missing sender on DBus call".into()))?;

        info!("Verifying Polkit authorization for DBus caller: {}", sender);

        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.livepatcher.apply", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Failed to query PolicyKit authority: {}", e)))?;

        if is_auth {
            info!("Polkit authorization granted for sender {}", sender);
        } else {
            let err_msg = "Polkit authorization denied: insufficient privileges".to_string();
            error!("{}", err_msg);
            return Err(zbus::fdo::Error::AccessDenied(err_msg));
        }

        info!("Verifying module signature for patch: {}", patch_path);

        let modinfo_output = Command::new("modinfo")
            .arg("--field=signer")
            .arg(patch_path)
            .output()
            .await;

        match modinfo_output {
            Ok(output) if output.status.success() => {
                let signer = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if signer.is_empty() {
                    let err_msg = format!("Module signer is empty: {}", patch_path);
                    error!("{}", err_msg);
                    return Err(zbus::fdo::Error::Failed(err_msg));
                }
                info!("Module signer: {}", signer);
            }
            Ok(output) => {
                let err_msg = format!(
                    "Failed to check module signature: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                error!("{}", err_msg);
                return Err(zbus::fdo::Error::Failed(err_msg));
            }
            Err(e) => {
                let err_msg = format!("Failed to execute modinfo: {}", e);
                error!("{}", err_msg);
                return Err(zbus::fdo::Error::Failed(err_msg));
            }
        }

        // Execute real BPF livepatch via bpftool
        // Example: kpatch load /path/to/patch.ko
        let output = Command::new("kpatch")
            .arg("load")
            .arg(patch_path)
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let msg = format!("Successfully applied live patch: {}", patch_path);
                info!("{}", msg);
                Ok(msg)
            }
            Ok(output) => {
                let err_msg = format!("Failed to apply patch: {}", String::from_utf8_lossy(&output.stderr));
                error!("{}", err_msg);
                Err(zbus::fdo::Error::Failed(err_msg))
            }
            Err(e) => {
                let err_msg = format!("Failed to execute kpatch: {}", e);
                error!("{}", err_msg);
                Err(zbus::fdo::Error::Failed(err_msg))
            }
        }
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting ermete-live-patcher daemon...");

    let _conn = connection::Builder::system()?
        .name("os.ermete.LivePatcher")?
        .serve_at("/os/ermete/LivePatcher", LivePatcher)?
        .build()
        .await?;

    info!("Daemon is listening on DBus");
    pending::<()>().await;

    Ok(())
}
