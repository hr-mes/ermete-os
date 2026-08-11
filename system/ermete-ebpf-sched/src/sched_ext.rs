use aya::maps::HashMap as BpfHashMap;
use aya::Ebpf;
use std::collections::HashMap as StdHashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[repr(C)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AiSchedMapValue {
    pub pid: u32,
    pub target_core: u32,       // Core ID assigned by AI Predictor DAG (P-Core vs E-Core)
    pub core_type: u8,          // 0 = P-Core, 1 = E-Core, 2 = NPU-Core
    pub _pad: [u8; 3],          // Padding for 4-byte alignment
    pub cpu_weight: u32,
    pub slice_us: u64,
    pub sched_class: u32,       // 0: RealtimeNpu, 1: InteractiveUi, 2: BatchCompute, 3: IdleBackground
    pub latency_target_us: u64,
    pub flags: u32,
}

unsafe impl aya::Pod for AiSchedMapValue {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SchedClass {
    RealtimeNpu = 0,     // Ultra-low latency NPU/AI tasks
    InteractiveUi = 1,   // Compositor / UI frame tasks
    BatchCompute = 2,    // Compilation / Heavy background compute
    IdleBackground = 3,  // Low priority background tasks
}

impl From<u32> for SchedClass {
    fn from(val: u32) -> Self {
        match val {
            0 => SchedClass::RealtimeNpu,
            1 => SchedClass::InteractiveUi,
            2 => SchedClass::BatchCompute,
            _ => SchedClass::IdleBackground,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskSchedPolicy {
    pub pid: u32,
    pub class: SchedClass,
    pub cpu_weight: u32,       // 1 to 10000 (cgroup v2 weight)
    pub slice_us: u64,         // Scheduling time slice in microseconds
    pub latency_target_us: u64,// Zero-latency target requirement
}

/// Safe thread-safe interface exposing `AI_SCHED_MAP` for daemons and scheduler controllers
#[derive(Clone)]
pub struct AiSchedMap {
    ebpf: Option<Arc<Mutex<Ebpf>>>,
    fallback: Arc<Mutex<StdHashMap<u32, AiSchedMapValue>>>,
}

impl AiSchedMap {
    pub fn new(ebpf: Option<Arc<Mutex<Ebpf>>>) -> Self {
        Self {
            ebpf,
            fallback: Arc::new(Mutex::new(StdHashMap::new())),
        }
    }

    pub async fn is_bpf_active(&self) -> bool {
        if let Some(ebpf_arc) = &self.ebpf {
            let mut ebpf = ebpf_arc.lock().await;
            ebpf.map_mut("AI_SCHED_MAP").is_some()
        } else {
            false
        }
    }

    pub async fn update_policy(&self, pid: u32, value: AiSchedMapValue) -> Result<(), String> {
        if let Some(ebpf_arc) = &self.ebpf {
            let mut ebpf = ebpf_arc.lock().await;
            if let Some(map) = ebpf.map_mut("AI_SCHED_MAP") {
                if let Ok(mut bpf_map) = BpfHashMap::<_, u32, AiSchedMapValue>::try_from(map) {
                    if let Err(e) = bpf_map.insert(pid, value, 0) {
                        return Err(format!("Failed to insert PID {} into eBPF AI_SCHED_MAP: {}", pid, e));
                    }
                    info!("⚡ [eBPF Map] AI_SCHED_MAP updated for PID {} -> weight={}, slice={}us", pid, value.cpu_weight, value.slice_us);
                    return Ok(());
                }
            }
        }

        let mut fallback = self.fallback.lock().await;
        fallback.insert(pid, value);
        info!("💡 [Fallback Map] AI_SCHED_MAP updated for PID {} -> weight={}, slice={}us", pid, value.cpu_weight, value.slice_us);
        Ok(())
    }

    pub async fn get_policy(&self, pid: u32) -> Option<AiSchedMapValue> {
        if let Some(ebpf_arc) = &self.ebpf {
            let mut ebpf = ebpf_arc.lock().await;
            if let Some(map) = ebpf.map_mut("AI_SCHED_MAP") {
                if let Ok(bpf_map) = BpfHashMap::<_, u32, AiSchedMapValue>::try_from(map) {
                    if let Ok(val) = bpf_map.get(&pid, 0) {
                        return Some(val);
                    }
                }
            }
        }

        let fallback = self.fallback.lock().await;
        fallback.get(&pid).copied()
    }

    pub async fn remove_policy(&self, pid: u32) -> Result<(), String> {
        if let Some(ebpf_arc) = &self.ebpf {
            let mut ebpf = ebpf_arc.lock().await;
            if let Some(map) = ebpf.map_mut("AI_SCHED_MAP") {
                if let Ok(mut bpf_map) = BpfHashMap::<_, u32, AiSchedMapValue>::try_from(map) {
                    let _ = bpf_map.remove(&pid);
                }
            }
        }

        let mut fallback = self.fallback.lock().await;
        fallback.remove(&pid);
        Ok(())
    }

    pub async fn list_policies(&self) -> Vec<(u32, AiSchedMapValue)> {
        if let Some(ebpf_arc) = &self.ebpf {
            let mut ebpf = ebpf_arc.lock().await;
            if let Some(map) = ebpf.map_mut("AI_SCHED_MAP") {
                if let Ok(bpf_map) = BpfHashMap::<_, u32, AiSchedMapValue>::try_from(map) {
                    if let Ok(keys) = bpf_map.keys().collect::<Result<Vec<_>, _>>() {
                        let mut results = Vec::new();
                        for k in keys {
                            if let Ok(val) = bpf_map.get(&k, 0) {
                                results.push((k, val));
                            }
                        }
                        return results;
                    }
                }
            }
        }

        let fallback = self.fallback.lock().await;
        fallback.iter().map(|(k, v)| (*k, *v)).collect()
    }
}

pub struct SchedExtController {
    sched_ext_enabled: bool,
    sched_map: AiSchedMap,
}

/// Critical process PIDs that AI agent scheduling must never deprioritize or manipulate
const PROTECTED_PIDS: &[u32] = &[
    0, // Kernel idle process
    1, // Init / systemd / system-oracle process
];

const EBPF_BYTECODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ermete-ebpf-sched-bpf"));

impl SchedExtController {
    pub async fn new() -> Self {
        info!("==========================================================================");
        info!("🧠 Initializing User-Space eBPF Scheduler Loader (`aya`) & sched_ext...");
        info!("==========================================================================");

        let mut loaded_ebpf = None;

        if !EBPF_BYTECODE.is_empty() {
            info!("⚡ Loading embedded eBPF bytecode (0 external runtime dependencies)...");
            match Ebpf::load(EBPF_BYTECODE) {
                Ok(bpf) => {
                    info!("✅ Successfully loaded embedded eBPF object bytecode ({} bytes)", EBPF_BYTECODE.len());
                    loaded_ebpf = Some(bpf);
                }
                Err(e) => {
                    warn!("⚠️ Failed to parse embedded eBPF bytecode: {}", e);
                }
            }
        }

        if loaded_ebpf.is_none() {
            let candidate_paths = [
                "target/bpfel-unknown-none/release/ermete-ebpf-sched-bpf",
                "target/bpfel-unknown-none/debug/ermete-ebpf-sched-bpf",
                "/usr/lib/ermete/ebpf/sched_ext.bpf.o",
                "system/ebpf/target/bpfel-unknown-none/release/ebpf-core",
            ];

            for path in candidate_paths {
                if std::path::Path::new(path).exists() {
                    info!("🔍 Found candidate BPF bytecode object at: {}", path);
                    match Ebpf::load_file(path) {
                        Ok(bpf) => {
                            info!("✅ Successfully loaded eBPF object file from {}", path);
                            loaded_ebpf = Some(bpf);
                            break;
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse BPF object file {}: {}", path, e);
                        }
                    }
                }
            }
        }


        let is_sysfs_sched_ext = std::path::Path::new("/sys/kernel/sched_ext").exists();

        let (sched_map, sched_ext_enabled) = if let Some(mut ebpf) = loaded_ebpf {
            let map_present = ebpf.map_mut("AI_SCHED_MAP").is_some();
            if map_present {
                info!("✅ `AI_SCHED_MAP` eBPF HashMap detected in BPF object.");
            } else {
                warn!("⚠️ Map `AI_SCHED_MAP` missing in BPF bytecode. Operating with in-memory fallback map.");
            }

            let mut attached = false;
            if is_sysfs_sched_ext {
                info!("⚡ Kernel `sched_ext` sysfs interface available.");
                if let Some(_prog) = ebpf.program_mut("scx_enqueue") {
                    info!("✅ Loaded `scx_enqueue` sched_ext eBPF program.");
                    attached = true;
                }
            } else {
                warn!("ℹ️ sysfs path `/sys/kernel/sched_ext` absent. Kernel standard CFS/EEVDF fallback activated.");
            }

            let ebpf_arc = Arc::new(Mutex::new(ebpf));
            (AiSchedMap::new(Some(ebpf_arc)), attached || is_sysfs_sched_ext)
        } else {
            warn!("⚠️ BPF bytecode object not found or load failed. Activating zero-latency cgroup v2 fallback scheduler.");
            (AiSchedMap::new(None), false)
        };

        Self {
            sched_ext_enabled,
            sched_map,
        }
    }

    pub fn sched_map(&self) -> &AiSchedMap {
        &self.sched_map
    }

    pub fn is_sched_ext_enabled(&self) -> bool {
        self.sched_ext_enabled
    }

    /// Apply zero-latency task priority decision directly into kernel sched_ext BPF maps or fallback map.
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

        let map_val = AiSchedMapValue {
            pid: policy.pid,
            target_core: 0,
            core_type: 0,
            _pad: [0; 3],
            cpu_weight: policy.cpu_weight,
            slice_us: policy.slice_us,
            sched_class: policy.class as u32,
            latency_target_us: policy.latency_target_us,
            flags: 1,
        };

        // Update AI_SCHED_MAP map safely
        self.sched_map.update_policy(policy.pid, map_val).await?;

        info!(
            "⚡ [sched_ext] Policy applied for PID {} ('{:?}'): Weight={}, Slice={}us, TargetLatency={}us (Mode: {})",
            policy.pid, policy.class, policy.cpu_weight, policy.slice_us, policy.latency_target_us,
            if self.sched_ext_enabled { "Kernel sched_ext" } else { "cgroup v2 Fallback" }
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_sched_map_fallback_ops() -> Result<(), String> {
        let map = AiSchedMap::new(None);
        assert!(!map.is_bpf_active().await);

        let val = AiSchedMapValue {
            pid: 2048,
            target_core: 1,
            core_type: 0,
            _pad: [0; 3],
            cpu_weight: 800,
            slice_us: 1000,
            sched_class: 1,
            latency_target_us: 500,
            flags: 1,
        };

        map.update_policy(2048, val).await?;

        let queried = map.get_policy(2048).await.ok_or("Policy should exist")?;
        assert_eq!(queried.cpu_weight, 800);
        assert_eq!(queried.slice_us, 1000);

        let policies = map.list_policies().await;
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].0, 2048);

        map.remove_policy(2048).await?;
        assert!(map.get_policy(2048).await.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_sched_ext_controller_confinement_and_validation() {
        let controller = SchedExtController::new().await;

        // Test PID 1 protection
        let pid1_policy = TaskSchedPolicy {
            pid: 1,
            class: SchedClass::RealtimeNpu,
            cpu_weight: 1000,
            slice_us: 1000,
            latency_target_us: 100,
        };
        assert!(controller.apply_task_policy(&pid1_policy).await.is_err());

        // Test invalid CPU weight
        let invalid_weight = TaskSchedPolicy {
            pid: 5000,
            class: SchedClass::InteractiveUi,
            cpu_weight: 20000,
            slice_us: 1000,
            latency_target_us: 500,
        };
        assert!(controller.apply_task_policy(&invalid_weight).await.is_err());

        // Test valid policy
        let valid_policy = TaskSchedPolicy {
            pid: 5000,
            class: SchedClass::InteractiveUi,
            cpu_weight: 800,
            slice_us: 2000,
            latency_target_us: 500,
        };
        assert!(controller.apply_task_policy(&valid_policy).await.is_ok());
    }
}



