use zbus::interface;
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, error};

pub struct StoreService {}

#[interface(name = "os.ermete.Store")]
impl StoreService {
    async fn search(&self, query: String) -> String {
        info!("DBus: Search requested for: {}", query);
        let output = Command::new("flatpak")
            .arg("search")
            .arg(&query)
            .output()
            .await;
            
        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => {
                error!("Error executing search: {}", e);
                format!("Error: {}", e)
            }
        }
    }

    async fn install(&self, package: String) -> String {
        info!("DBus: Install requested for: {}", package);
        
        let mut child = match Command::new("flatpak")
            .arg("install")
            .arg("-y")
            .arg(&package)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to spawn flatpak install: {}", e);
                return format!("Error spawning flatpak: {}", e);
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    // Piping percentage or output
                    info!("Install Output [{}]: {}", package, line);
                }
            });
        }
        
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    info!("Successfully installed {}", package);
                    "Success".to_string()
                } else {
                    error!("Failed to install {} (status: {})", package, status);
                    format!("Failed with status: {}", status)
                }
            },
            Err(e) => {
                error!("Install process error: {}", e);
                format!("Process error: {}", e)
            }
        }
    }
}

pub async fn start_dbus_server() -> anyhow::Result<()> {
    let _conn = zbus::connection::Builder::session()?
        .name("os.ermete.Store")?
        .serve_at("/os/ermete/Store", StoreService {})?
        .build()
        .await?;

    info!("DBus server os.ermete.Store is running.");
    
    // The connection will stay alive and process requests.
    // We can just keep it alive using pending.
    std::future::pending::<()>().await;
    
    Ok(())
}
