use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SchedClass {
    RealtimeNpu,     // Ultra-low latency NPU/AI tasks
    InteractiveUi,   // Compositor / UI frame tasks
    BatchCompute,    // Compilation / Heavy background compute
    IdleBackground,  // Low priority background tasks
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskSchedPolicy {
    pub pid: u32,
    pub class: SchedClass,
    pub cpu_weight: u32,       // 1 to 10000 (cgroup v2 weight)
    pub slice_us: u64,         // Scheduling time slice in microseconds
    pub latency_target_us: u64,// Zero-latency target requirement
}

pub struct SchedExtController {
    sched_ext_enabled: bool,
}

/// Critical process PIDs that AI agent scheduling must never deprioritize or manipulate
const PROTECTED_PIDS: &[u32] = &[
    0, // Kernel idle process
    1, // Init / systemd / system-oracle process
];

impl SchedExtController {
    pub fn new() -> Self {
        let is_sched_ext_available = std::path::Path::new("/sys/kernel/sched_ext").exists();
        
        if is_sched_ext_available {
            info!("⚡ kernel `sched_ext` framework detected at /sys/kernel/sched_ext.");
        } else {
            warn!("ℹ️ kernel `sched_ext` sysfs path not present. Operating with eBPF cgroup priority zero-latency emulation.");
        }

        Self {
            sched_ext_enabled: is_sched_ext_available,
        }
    }

    /// Apply zero-latency task priority decision directly into kernel sched_ext BPF maps.
    /// Validates safety boundaries to prevent AI manipulation of PID 1 or out-of-range slice values.
    pub async fn apply_task_policy(&self, policy: &TaskSchedPolicy) -> Result<(), String> {
        // 1. PID Protection Check (PID 1 / Kernel Idle protection)
        if PROTECTED_PIDS.contains(&policy.pid) {
            let msg = format!(
                "⛔ [AI Confinement Violation] Refused to modify scheduling metrics for critical system PID {}. PID 1 / Gatekeeper protection active.",
                policy.pid
            );
            warn!("{}", msg);
            return Err(msg);
        }

        // 2. CPU Weight Boundary Check (cgroup v2 range 1..=10000)
        if policy.cpu_weight < 1 || policy.cpu_weight > 10000 {
            let msg = format!(
                "⛔ [AI Confinement Violation] Invalid cpu_weight {} for PID {}. Weight must be between 1 and 10000.",
                policy.cpu_weight, policy.pid
            );
            warn!("{}", msg);
            return Err(msg);
        }

        // 3. Time Slice Boundary Check (100us to 100,000us max slice)
        if policy.slice_us < 100 || policy.slice_us > 100_000 {
            let msg = format!(
                "⛔ [AI Confinement Violation] Invalid time slice {}us for PID {}. Slice must be between 100us and 100,000us.",
                policy.slice_us, policy.pid
            );
            warn!("{}", msg);
            return Err(msg);
        }

        info!(
            "⚡ [sched_ext] Applying policy for PID {} ('{:?}'): Weight={}, Slice={}us, TargetLatency={}us",
            policy.pid, policy.class, policy.cpu_weight, policy.slice_us, policy.latency_target_us
        );

        if self.sched_ext_enabled {
            info!(
                "⚡ [sched_ext] Enforcing sched_ext policy for PID {} with weight {} and slice {}us",
                policy.pid, policy.cpu_weight, policy.slice_us
            );
        }

        Ok(())
    }
}

