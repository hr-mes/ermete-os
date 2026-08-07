use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;
use tokio::time;
use zbus::{ConnectionBuilder, interface, SignalContext};

#[derive(Serialize, Deserialize, Default, Debug)]
struct HealthReport {
    nvme: Option<String>,
    bcachefs: Option<String>,
}

struct SystemHealth;

#[interface(name = "os.ermete.SystemHealth")]
impl SystemHealth {
    #[zbus(signal)]
    async fn system_health_update(
        ctxt: &SignalContext<'_>,
        health_json: &str,
    ) -> zbus::Result<()>;
}

async fn get_nvme_health() -> Option<String> {
    let output = Command::new("smartctl")
        .args(["-i", "-A", "-j", "/dev/nvme0n1"])
        .output()
        .await
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

async fn get_bcachefs_health() -> Option<String> {
    let output = Command::new("bcachefs")
        .args(["device", "stats", "/"])
        .output()
        .await
        .ok()?;
        
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system_health = SystemHealth;
    let _conn = ConnectionBuilder::system()?
        .name("os.ermete.SystemHealth")?
        .serve_at("/os/ermete/SystemHealth", system_health)?
        .build()
        .await?;

    // Wait for the object to be registered and name to be acquired
    let context = SignalContext::new(&_conn, "/os/ermete/SystemHealth")?;

    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let nvme = get_nvme_health().await;
        let bcachefs = get_bcachefs_health().await;

        let report = HealthReport { nvme, bcachefs };

        if let Ok(json) = serde_json::to_string(&report) {
            if let Err(e) = SystemHealth::system_health_update(&context, &json).await {
                eprintln!("Failed to emit system_health_update signal: {}", e);
            }
        }
    }
}
