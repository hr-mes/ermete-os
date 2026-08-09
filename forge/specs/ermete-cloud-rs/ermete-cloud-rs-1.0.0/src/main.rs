use anyhow::Result;
use zbus::interface;
use tokio::process::Command;
use tracing::{info, error};

mod dbus;
mod sync;
mod zk;
mod bft;
mod discovery;
mod listener;
mod clipboard;

pub struct CloudSyncIface {}

#[interface(name = "os.ermete.CloudSync")]
impl CloudSyncIface {
    async fn authenticate_oauth(&self, provider: String, token: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Authenticating OAuth with provider: {}", provider);

        if token.trim().is_empty() {
            return Err(zbus::fdo::Error::InvalidArgs("OAuth token cannot be empty".into()));
        }

        let client = reqwest::Client::new();
        let url = match provider.to_lowercase().as_str() {
            "google" => format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", token),
            "github" => "https://api.github.com/user".to_string(),
            _ => format!("https://{}/userinfo", provider),
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "ErmeteOS-CloudSync")
            .send()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(zbus::fdo::Error::Failed(format!(
                "OAuth validation failed for provider '{}' with status {}",
                provider,
                response.status()
            )));
        }

        Ok(format!("Authenticated securely with {}", provider))
    }


    async fn mount_fuse(&self, remote: String, mountpoint: String) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Orchestrating FUSE mount for remote '{}' at '{}'", remote, mountpoint);
        let remote_clone = remote.clone();
        let mountpoint_clone = mountpoint.clone();
        
        tokio::spawn(async move {
            let child = Command::new("rclone")
                .arg("mount")
                .arg(&remote_clone)
                .arg(&mountpoint_clone)
                .arg("--vfs-cache-mode")
                .arg("full")
                .spawn();

            match child {
                Ok(mut c) => {
                    info!("Started rclone mount {} -> {}", remote_clone, mountpoint_clone);
                    if let Err(e) = c.wait().await {
                        error!("rclone mount process exited with error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to spawn rclone mount: {}", e);
                }
            }
        });

        Ok(format!("Initiated FUSE mount for {}", remote))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Cloud Daemon (Level 15: ZK-Mesh Computing & Byzantine Consensus)");

    let sync_engine = std::sync::Arc::new(sync::SyncEngine::new());
    
    // Export D-Bus interfaces
    let _conn = zbus::connection::Builder::session()?
        .name("os.ermete.CloudSync")?
        .serve_at("/os/ermete/CloudSync", CloudSyncIface {})?
        .serve_at("/os/ermete/Cloud", dbus::CloudIface { engine: sync_engine.clone() })?
        .build()
        .await?;

    info!("D-Bus Interfaces 'os.ermete.CloudSync' and 'os.ermete.Cloud' registered.");

    // Start local mDNS & ZK discovery loop
    sync_engine.start_discovery().await?;

    // Purely asynchronous event loop
    std::future::pending::<()>().await;
    
    Ok(())
}
