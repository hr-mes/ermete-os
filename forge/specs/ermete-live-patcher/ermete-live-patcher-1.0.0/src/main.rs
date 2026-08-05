use tokio::process::Command;
use tracing::{error, info};
use zbus::{connection, interface};
use std::future::pending;

struct LivePatcher;

#[interface(name = "os.ermete.LivePatcher1")]
impl LivePatcher {
    async fn apply_kernel_patch(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        patch_path: &str,
    ) -> zbus::fdo::Result<String> {
        info!("Received request to apply kernel live patch: {}", patch_path);

        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("Missing sender on DBus call".into()))?;

        info!("Verifying Polkit authorization for DBus caller: {}", sender);

        let polkit_status = Command::new("pkcheck")
            .arg("--system-bus-name")
            .arg(sender.as_str())
            .arg("--action-id")
            .arg("os.ermete.livepatcher.apply")
            .arg("--allow-user-interaction")
            .status()
            .await;

        match polkit_status {
            Ok(status) if status.success() => {
                info!("Polkit authorization granted for sender {}", sender);
            }
            Ok(_) => {
                let err_msg = "Polkit authorization denied: insufficient privileges".to_string();
                error!("{}", err_msg);
                return Err(zbus::fdo::Error::Failed(err_msg));
            }
            Err(e) => {
                let err_msg = format!("Failed to execute pkcheck for authorization: {}", e);
                error!("{}", err_msg);
                return Err(zbus::fdo::Error::Failed(err_msg));
            }
        }

        // Mocking the kpatch or livepatch execution
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
