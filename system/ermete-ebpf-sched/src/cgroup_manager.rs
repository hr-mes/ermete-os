#![allow(clippy::all)]
#![allow(clippy::pedantic)]

use std::path::PathBuf;
use tracing::info;

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

        if !target_cgroup.exists() {
            if let Err(e) = std::fs::create_dir_all(&target_cgroup) {
                return Err(format!("Failed to create CGroup slice {:?}: {}", target_cgroup, e));
            }
        }

        let procs_path = target_cgroup.join("cgroup.procs");
        let weight_path = target_cgroup.join("cpu.weight");

        if let Err(e) = std::fs::write(&weight_path, cpu_weight.to_string()) {
            return Err(format!("Failed to write cpu.weight to {:?}: {}", weight_path, e));
        }
        if let Err(e) = std::fs::write(&procs_path, pid.to_string()) {
            return Err(format!("Failed to attach PID {} to cgroup procs {:?}: {}", pid, procs_path, e));
        }

        Ok(())
    }
}

