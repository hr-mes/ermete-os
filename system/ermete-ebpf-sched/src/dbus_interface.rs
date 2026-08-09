use crate::sched_ext::{SchedClass, SchedExtController, TaskSchedPolicy};
use std::sync::Arc;
use zbus::interface;

pub struct SchedExtDbusInterface {
    controller: Arc<SchedExtController>,
}

impl SchedExtDbusInterface {
    pub fn new(controller: Arc<SchedExtController>) -> Self {
        Self { controller }
    }
}

#[interface(name = "os.ermete.SchedExt")]
impl SchedExtDbusInterface {
    /// Remote interface allowing external daemons to update `AI_SCHED_MAP`
    async fn update_sched_map(
        &self,
        pid: u32,
        cpu_weight: u32,
        slice_us: u64,
        sched_class: u32,
        latency_target_us: u64,
    ) -> String {
        let policy = TaskSchedPolicy {
            pid,
            class: SchedClass::from(sched_class),
            cpu_weight,
            slice_us,
            latency_target_us,
        };

        match self.controller.apply_task_policy(&policy).await {
            Ok(_) => serde_json::json!({
                "success": true,
                "pid": pid,
                "bpf_active": self.controller.sched_map().is_bpf_active().await,
                "message": format!("Successfully updated AI_SCHED_MAP for PID {}", pid)
            })
            .to_string(),
            Err(err) => serde_json::json!({
                "success": false,
                "pid": pid,
                "error": err
            })
            .to_string(),
        }
    }

    /// Query policy for a specific PID from `AI_SCHED_MAP`
    async fn get_sched_map(&self, pid: u32) -> String {
        match self.controller.sched_map().get_policy(pid).await {
            Some(val) => serde_json::json!({
                "found": true,
                "pid": val.pid,
                "cpu_weight": val.cpu_weight,
                "slice_us": val.slice_us,
                "sched_class": val.sched_class,
                "latency_target_us": val.latency_target_us,
                "flags": val.flags
            })
            .to_string(),
            None => serde_json::json!({
                "found": false,
                "pid": pid
            })
            .to_string(),
        }
    }

    /// List all policies registered in `AI_SCHED_MAP`
    async fn list_sched_map(&self) -> String {
        let policies = self.controller.sched_map().list_policies().await;
        let serialized: Vec<_> = policies
            .into_iter()
            .map(|(pid, val)| {
                serde_json::json!({
                    "pid": pid,
                    "cpu_weight": val.cpu_weight,
                    "slice_us": val.slice_us,
                    "sched_class": val.sched_class,
                    "latency_target_us": val.latency_target_us
                })
            })
            .collect();

        serde_json::json!({
            "count": serialized.len(),
            "policies": serialized,
            "bpf_active": self.controller.sched_map().is_bpf_active().await
        })
        .to_string()
    }

    /// Status endpoint
    async fn status(&self) -> String {
        serde_json::json!({
            "daemon": "ermete-ebpf-sched",
            "version": "0.1.0",
            "sched_ext_enabled": self.controller.is_sched_ext_enabled(),
            "bpf_map_active": self.controller.sched_map().is_bpf_active().await,
            "status": "ONLINE"
        })
        .to_string()
    }
}
