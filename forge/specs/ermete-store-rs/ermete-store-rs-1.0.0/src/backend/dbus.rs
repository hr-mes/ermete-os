use zbus::interface;
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, error};

pub struct StoreService {}

#[interface(name = "os.ermete.Store")]
impl StoreService {
    async fn search(&self, query: String) -> String {
        if query.starts_with('-') {
            return "Error: Invalid query".to_string();
        }
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

    async fn verify_pqc_package(&self, package: String, signature_hex: String, pubkey_hex: String) -> String {
        info!("DBus PQC Level 13: Verifying Dilithium5 signature for package: {}", package);
        
        let sig_bytes = match hex::decode(&signature_hex) {
            Ok(b) => b,
            Err(_) => return "Invalid signature hex encoding".to_string(),
        };
        let pubkey_bytes = match hex::decode(&pubkey_hex) {
            Ok(b) => b,
            Err(_) => return "Invalid pubkey hex encoding".to_string(),
        };

        let payload = format!("ERMETE_STORE_PACKAGE:{}", package);
        if pqc_dilithium::verify(&sig_bytes, payload.as_bytes(), &pubkey_bytes).is_ok() {
            info!("Dilithium5 PQC signature VERIFIED for {}", package);
            "PQC_VERIFIED_OK".to_string()
        } else {
            error!("Dilithium5 PQC signature VERIFICATION FAILED for {}", package);
            "PQC_VERIFICATION_FAILED".to_string()
        }
    }


    async fn install(&self, package: String) -> String {
        if package.starts_with('-') {
            return "Error: Invalid package".to_string();
        }
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
            let pkg_name = package.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    // Piping percentage or output
                    info!("Install Output [{}]: {}", pkg_name, line);
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
