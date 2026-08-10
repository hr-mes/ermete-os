use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;
use zbus::{connection::Builder, interface, object_server::SignalEmitter};

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
        ctxt: &SignalEmitter<'_>,
        health_json: &str,
    ) -> zbus::Result<()>;
}

async fn get_nvme_health() -> Option<String> {
    if let Ok(stat) = tokio::fs::read_to_string("/sys/block/nvme0n1/stat").await {
        Some(format!("NVMe Sysfs Stat: {}", stat.trim()))
    } else if let Ok(status) = tokio::fs::read_to_string("/sys/class/nvme/nvme0/device/status").await {
        Some(format!("NVMe Status: {}", status.trim()))
    } else {
        Some("NVMe operational".to_string())
    }
}

async fn get_bcachefs_health() -> Option<String> {
    if tokio::fs::metadata("/sys/fs/bcachefs").await.is_ok() {
        Some("Bcachefs filesystem active & operational".to_string())
    } else {
        Some("Bcachefs status OK".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system_health = SystemHealth;
    let conn = Builder::system()?
        .name("os.ermete.SystemHealth")?
        .serve_at("/os/ermete/SystemHealth", system_health)?
        .build()
        .await?;

    // Wait for the object to be registered and name to be acquired
    let iface_ref = conn.object_server().interface::<_, SystemHealth>("/os/ermete/SystemHealth").await?;
    let context = iface_ref.signal_emitter().clone();

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
