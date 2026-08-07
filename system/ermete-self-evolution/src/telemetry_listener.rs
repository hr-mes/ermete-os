use tracing::{info, warn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum BottleneckKind {
    SyscallSpike,
    MemoryPressure,
    CPULoopHotspot,
    NetworkInboundProcessing,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryBottleneckAlert {
    pub timestamp: u64,
    pub bottleneck_type: BottleneckKind,
    pub target_crate: String,
    pub hotspot_symbol: String,
    pub severity: f64,
    pub syscall_rate_hz: u64,
}

pub struct TelemetryListener {
    simulated_ticks: u64,
    syscall_threshold_hz: u64,
}

impl TelemetryListener {
    pub fn new(syscall_threshold_hz: u64) -> Self {
        Self {
            simulated_ticks: 0,
            syscall_threshold_hz,
        }
    }

    /// Listens for kernel telemetry and evaluates execution path bottlenecks
    pub async fn poll_bottleneck(&mut self) -> Option<TelemetryBottleneckAlert> {
        self.simulated_ticks += 1;

        // Simulate monitoring Ring-0 telemetry stream
        let current_syscall_rate = 18500 + (self.simulated_ticks * 750) % 12000;

        if current_syscall_rate > self.syscall_threshold_hz {
            let alert = TelemetryBottleneckAlert {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                bottleneck_type: BottleneckKind::SyscallSpike,
                target_crate: "ermete-agentic-kernel".to_string(),
                hotspot_symbol: "ebpf_monitor::collect_telemetry".to_string(),
                severity: 0.88,
                syscall_rate_hz: current_syscall_rate,
            };

            warn!(
                "🚨 [Telemetry Listener] Ring-0 Bottleneck Detected! Syscall Rate: {} Hz (Threshold: {} Hz). Target Crate: '{}'",
                current_syscall_rate, self.syscall_threshold_hz, alert.target_crate
            );

            return Some(alert);
        }

        info!("🟢 [Telemetry Listener] Ring-0 execution telemetry optimal (Syscalls: {} Hz).", current_syscall_rate);
        None
    }
}
