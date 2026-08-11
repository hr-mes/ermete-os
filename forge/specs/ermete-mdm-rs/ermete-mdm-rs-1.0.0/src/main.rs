use std::os::unix::fs::OpenOptionsExt;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use serde::Deserialize;
use tokio::process::Command;
use tracing::{error, info, warn};
use zbus::{connection::Builder, interface, object_server::SignalEmitter};

#[derive(Deserialize, Debug)]
struct MdmPayload {
    action: String,
}

struct MdmDBusInterface;

impl MdmDBusInterface {
    async fn disable_usb(&self) -> bool {
        info!("Disabling USB storage...");
        // Applying the policy directly to disk via non-blocking I/O
        let res = async {
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open("/etc/modprobe.d/disable-usb-storage.conf")
                .await?;
            tokio::io::AsyncWriteExt::write_all(&mut file, b"install usb-storage /bin/true\n").await
        }.await;
        if let Err(e) = res {
            error!("Failed to write modprobe config: {}", e);
            return false;
        }

        // Execute system command asynchronously
        let _ = Command::new("rmmod").arg("usb_storage").output().await;

        true
    }

    async fn force_vpn(&self) -> bool {
        info!("Forcing VPN...");
        let output = Command::new("systemctl")
            .args(["enable", "--now", "openvpn-client@ermete.service"])
            .output()
            .await;

        match output {
            Ok(out) => out.status.success(),
            Err(e) => {
                error!("Failed to execute systemctl: {}", e);
                false
            }
        }
    }
}

#[interface(name = "os.ermete.Mdm")]
impl MdmDBusInterface {
    async fn apply_policy(
        &self,
        payload_json: &str,
        #[zbus(signal_emitter)] ctxt: SignalEmitter<'_>,
    ) -> String {
        info!("Received policy payload: {}", payload_json);
        let payload: Result<MdmPayload, serde_json::Error> = serde_json::from_str(payload_json);
        match payload {
            Ok(p) => {
                let success = match p.action.as_str() {
                    "disable_usb" => self.disable_usb().await,
                    "force_vpn" => self.force_vpn().await,
                    _ => {
                        warn!("Unknown action: {}", p.action);
                        false
                    }
                };

                if success {
                    info!("Action {} applied successfully", p.action);
                    let _ = Self::policy_applied(&ctxt, &p.action).await;
                    format!("Policy {} applied successfully.", p.action)
                } else {
                    error!("Action {} failed", p.action);
                    format!("Policy {} execution failed.", p.action)
                }
            }
            Err(e) => {
                error!("Invalid payload: {}", e);
                format!("Invalid payload JSON: {}", e)
            }
        }
    }

    #[zbus(signal)]
    async fn policy_applied(ctxt: &SignalEmitter<'_>, action: &str) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting ermete-mdm-rs via DBus...");

    let mdm_interface = MdmDBusInterface;

    // Set up DBus connection for the interface
    let _conn = Builder::system()?
        .name("os.ermete.Mdm")?
        .serve_at("/os/ermete/Mdm", mdm_interface)?
        .build()
        .await?;

    info!("DBus interface os.ermete.Mdm is ready.");

    // Keep the daemon alive and listen for exit signals
    let mut exit_sig =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut int_sig =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

    tokio::select! {
        _ = exit_sig.recv() => {
            info!("Received SIGTERM, shutting down.");
        }
        _ = int_sig.recv() => {
            info!("Received SIGINT, shutting down.");
        }
    }

    Ok(())
}
