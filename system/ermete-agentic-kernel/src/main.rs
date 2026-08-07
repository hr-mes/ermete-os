#![allow(unsafe_code)]

pub mod ai_client;
pub mod auto_healer;
pub mod ebpf_monitor;

use ai_client::AiDaemonClient;
use auto_healer::AutoHealer;
use ebpf_monitor::EbpfMonitor;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("==========================================================================");
    info!("🧠 Level 14 Autonomous Ring-0 Agentic OS Controller Starting...");
    info!("   Making the Ermete OS Kernel Conscious with NPU AI & Aya eBPF Probes");
    info!("==========================================================================");

    // Bump memlock rlimit for eBPF map allocations
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    unsafe {
        libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim);
    }

    // Initialize subsystems
    let mut ebpf_mon = EbpfMonitor::new().await;
    let ai_client = AiDaemonClient::new().await;
    let auto_healer = AutoHealer::new();

    let mut interval = tokio::time::interval(Duration::from_secs(2));

    info!("Autonomous Ring-0 Control Loop active. Monitoring kernel telemetry...");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // 1. Observe Ring-0 kernel telemetry
                let telemetry = ebpf_mon.collect_telemetry().await;
                info!(
                    "[Ring-0 Telemetry] Syscalls: {}/s | MemPressure: {}MB | Pkts: {} pass / {} drop | TCP Scans: {}",
                    telemetry.syscall_frequency_hz,
                    telemetry.memory_pressure_mb,
                    telemetry.network_passed_packets,
                    telemetry.network_dropped_packets,
                    telemetry.tcp_scans_detected
                );

                // 2. Query local NPU AI engine for decision
                let decision = ai_client.evaluate_telemetry(&telemetry).await;

                if decision.anomaly_detected {
                    warn!(
                        "⚡ Autonomous Action Triggered! AI Risk Score: {:.2}. Recommended Actions: {:?}",
                        decision.risk_score, decision.recommended_actions
                    );

                    // 3a. Auto-Healing: Inject sysctl parameters to adjust kernel resource allocation
                    auto_healer.apply_autonomic_reallocation(&decision.sysctl_mitigations);

                    // 3b. Hot-rewrite eBPF rules in Ring-0
                    for ip_str in &decision.block_ips {
                        if let Ok(ip) = Ipv4Addr::from_str(ip_str) {
                            if let Err(e) = ebpf_mon.hot_block_ip(ip).await {
                                warn!("Failed to hot-rewrite eBPF blocklist map for {}: {}", ip, e);
                            }
                        }
                    }

                    if decision.zero_trust_enforce {
                        if let Err(e) = ebpf_mon.hot_set_zero_trust(true).await {
                            warn!("Failed to enable zero-trust eBPF mode: {}", e);
                        }
                    }
                } else {
                    info!("Kernel health optimal. Zero autonomic intervention required.");
                }
            }
            _ = signal::ctrl_c() => {
                info!("Received Ctrl-C signal. Shutting down Agentic OS Ring-0 Controller cleanly.");
                break;
            }
        }
    }

    Ok(())
}
