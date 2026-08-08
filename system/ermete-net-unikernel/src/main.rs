mod device;
mod metrics;
mod router;
mod stack;

use std::env;
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use smoltcp::phy::Medium;
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;
use tokio::signal;

use device::DeviceManager;
use metrics::NetworkMetrics;
use router::IsolationPolicy;
use stack::UnikernelNetworkStack;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("ermete_net=info".parse()?))
        .init();

    tracing::info!("=================================================================");
    tracing::info!("🌋 Ermete OS - Network Unikernel Stack Daemon (smoltcp TCP/IP Bypass)");
    tracing::info!("=================================================================");
    tracing::info!("Bypassing Linux kernel C-networking module -> Operating in Rust userspace");

    let interface_name = env::var("TAP_INTERFACE").unwrap_or_else(|_| "tap-ermete0".to_string());
    let policy_str = env::var("ISOLATION_POLICY").unwrap_or_else(|_| "enclave".to_string());

    let policy = match policy_str.to_lowercase().as_str() {
        "airgap" | "airgapped" => IsolationPolicy::AirGapped,
        "promiscuous" => IsolationPolicy::Promiscuous,
        _ => IsolationPolicy::IsolatedEnclave,
    };

    let metrics = Arc::new(NetworkMetrics::new());

    // Attempt to bind to host TUN/TAP interface; fallback to isolated Loopback device if unavailable
    let mut device = match DeviceManager::new_tuntap(&interface_name, Medium::Ethernet) {
        Ok(tuntap) => tuntap,
        Err(err) => {
            tracing::warn!(
                target: "ermete_net",
                "TUN/TAP creation warning ({}): Falling back to zero-cost synthetic Loopback device",
                err
            );
            DeviceManager::new_loopback(Medium::Ethernet)
        }
    };

    let mac_address = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
    let mut stack = UnikernelNetworkStack::new(mac_address, policy, Arc::clone(&metrics));

    // Register initial Phase 3 Micro-VM addresses in zero-trust router
    if let Ok(microvm_ip) = "10.0.2.10".parse::<IpAddress>() {
        stack.router_mut().register_microvm(microvm_ip);
    }
    if let Ok(microvm_ip) = "10.0.2.11".parse::<IpAddress>() {
        stack.router_mut().register_microvm(microvm_ip);
    }

    tracing::info!(
        target: "ermete_net",
        "Stack listening on interface '{}' with Zero-Trust Policy {:?}",
        device.interface_name(),
        policy
    );

    let mut poll_interval = tokio::time::interval(Duration::from_millis(5));
    let mut metrics_interval = tokio::time::interval(Duration::from_secs(5));

    let shutdown_signal = signal::ctrl_c();
    tokio::pin!(shutdown_signal);

    loop {
        tokio::select! {
            _ = poll_interval.tick() => {
                let now = Instant::now();
                let _updated = stack.poll_device(&mut device, now);
            }
            _ = metrics_interval.tick() => {
                tracing::info!(target: "ermete_net", "📊 Telemetry: {}", metrics.summary());
            }
            _ = &mut shutdown_signal => {
                tracing::info!(target: "ermete_net", "Received shutdown signal. Stopping Network Unikernel Daemon cleanly...");
                break;
            }
        }
    }

    tracing::info!(target: "ermete_net", "Daemon stopped. Final Telemetry: {}", metrics.summary());
    Ok(())
}
