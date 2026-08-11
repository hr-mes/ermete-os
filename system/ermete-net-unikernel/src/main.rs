mod device;
mod ipc;
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
use ipc::ZeroCopyRingBuffer;
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
    tracing::info!("🌋 Ermete OS - Sealed Network Unikernel Stack (Blind Mode / Phase 7)");
    tracing::info!("=================================================================");
    tracing::info!("🔒 Zero-Trust Isolation: Direct DBus / RPC Exports Eradicated.");
    tracing::info!("⚡ IPC Channel: Lock-Free Shared Memory ZeroCopyRingBuffer exclusively.");

    let interface_name = env::var("TAP_INTERFACE").unwrap_or_else(|_| "tap-ermete0".to_string());
    let policy_str = env::var("ISOLATION_POLICY").unwrap_or_else(|_| "enclave".to_string());

    let policy = match policy_str.to_lowercase().as_str() {
        "airgap" | "airgapped" => IsolationPolicy::AirGapped,
        "promiscuous" => IsolationPolicy::Promiscuous,
        _ => IsolationPolicy::IsolatedEnclave,
    };

    let metrics = Arc::new(NetworkMetrics::new());

    // Initialize Shared Memory ZeroCopyRingBuffer IPC channels for UI integration (Blind Mode)
    let rx_ring_buffer = ZeroCopyRingBuffer::create_named("ermete-net-ui-rx", 2 * 1024 * 1024)
        .or_else(|_| ZeroCopyRingBuffer::create_anonymous("ermete-net-ui-rx", 2 * 1024 * 1024))?;
    let tx_ring_buffer = ZeroCopyRingBuffer::create_named("ermete-net-ui-tx", 2 * 1024 * 1024)
        .or_else(|_| ZeroCopyRingBuffer::create_anonymous("ermete-net-ui-tx", 2 * 1024 * 1024))?;

    tracing::info!(
        target: "ermete_net",
        "Blind Mode IPC active: rx_ring (2MB) & tx_ring (2MB) bound for UI communication."
    );

    // Attempt to bind to host TUN/TAP interface; fallback to synthetic Loopback device if unavailable
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
    if let Ok(microvm_ip) = std::env::var("ERMETE_UNIKERNEL_IP").unwrap_or_else(|_| "10.0.2.10".to_string()).as_str().parse::<IpAddress>() {
        stack.router_mut().register_microvm(microvm_ip);
    }
    if let Ok(microvm_ip) = std::env::var("ERMETE_UNIKERNEL_IP").unwrap_or_else(|_| "10.0.2.10".to_string()).as_str().parse::<IpAddress>() {
        stack.router_mut().register_microvm(microvm_ip);
    }

    tracing::info!(
        target: "ermete_net",
        "Sealed Unikernel listening on interface '{}' with Zero-Trust Policy {:?}",
        device.interface_name(),
        policy
    );

    let mut poll_interval = tokio::time::interval(Duration::from_millis(5));
    let mut metrics_interval = tokio::time::interval(Duration::from_secs(5));

    let shutdown_signal = signal::ctrl_c();
    tokio::pin!(shutdown_signal);

    // Async main loop operating in sealed Blind Mode
    loop {
        tokio::select! {
            _ = poll_interval.tick() => {
                let now = Instant::now();

                // 1. Process ingress IPC frame requests from UI via ZeroCopyRingBuffer
                stack.process_ipc_ingress(&rx_ring_buffer);

                // 2. Poll hardware TUN/TAP device & execute smoltcp network stack
                let _updated = stack.poll_device(&mut device, now);

                // 3. Process egress IPC frames / telemetry push to UI via ZeroCopyRingBuffer
                stack.process_ipc_egress(&tx_ring_buffer);
            }
            _ = metrics_interval.tick() => {
                tracing::info!(target: "ermete_net", "📊 Telemetry [Blind Mode]: {}", metrics.summary());
                let telemetry_frame = metrics.summary().into_bytes();
                let _ = tx_ring_buffer.push_frame(0x0001, &telemetry_frame);
            }
            _ = &mut shutdown_signal => {
                tracing::info!(target: "ermete_net", "Received shutdown signal. Stopping Sealed Network Unikernel Daemon cleanly...");
                break;
            }
        }
    }

    tracing::info!(target: "ermete_net", "Sealed Daemon stopped. Final Telemetry: {}", metrics.summary());
    Ok(())
}
