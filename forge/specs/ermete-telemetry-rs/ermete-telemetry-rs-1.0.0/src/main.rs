use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod coredump;
mod dbus;
mod github;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Ermete Telemetry Daemon (Crash/Coredump Opt-in Reporter)");

    // Export D-Bus interface
    let _conn = zbus::ConnectionBuilder::system()?
        .name("os.ermete.Telemetry")?
        .serve_at("/os/ermete/Telemetry", dbus::TelemetryIface)?
        .build()
        .await?;

    info!("D-Bus Interface 'os.ermete.Telemetry' registered.");

    let mut watcher = coredump::CoredumpWatcher::new();

    // Main event loop
    loop {
        // Here we would use `inotify` on `/var/lib/systemd/coredump`
        // or listen to `systemd-coredump` D-Bus signals.
        if let Some(crash) = watcher.poll_new_crashes().await {
            info!("Detected new crash: {:?}", crash);
            // We do NOT send it automatically. We wait for the User UI to call our D-Bus method `SubmitCrash(crash_id)`.
        }
        
        sleep(Duration::from_secs(60)).await;
    }
}
