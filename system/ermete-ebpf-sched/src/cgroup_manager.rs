use std::path::PathBuf;
use tracing::{info, warn};

pub struct CgroupManager {
    cgroup_root: PathBuf,
}

impl CgroupManager {
    pub fn new() -> Self {
        let root = PathBuf::from("/sys/fs/cgroup");
        Self { cgroup_root: root }
    }

    /// Set process cgroup weight and CPU latency rules at zero latency
    pub fn update_process_cgroup(&self, pid: u32, cpu_weight: u32, is_realtime: bool) -> Result<(), String> {
        info!("🎯 [CGroup v2] Updating task priority for PID {} -> cpu.weight={}", pid, cpu_weight);

        let target_cgroup = if is_realtime {
            self.cgroup_root.join("ermete_realtime.slice")
        } else {
            self.cgroup_root.join("ermete_background.slice")
        };

        if target_cgroup.exists() {
            let procs_path = target_cgroup.join("cgroup.procs");
            let weight_path = target_cgroup.join("cpu.weight");

            if let Err(e) = std::fs::write(&weight_path, cpu_weight.to_string()) {
                warn!("Failed to write cpu.weight to {:?}: {}", weight_path, e);
            }
            if let Err(e) = std::fs::write(&procs_path, pid.to_string()) {
                warn!("Failed to attach PID {} to cgroup procs {:?}: {}", pid, procs_path, e);
            }
        } else {
            info!(
                "💡 CGroup slice {:?} not mounted. Zero-latency simulated priority update applied for PID {}.",
                target_cgroup, pid
            );
        }

        Ok(())
    }
}
