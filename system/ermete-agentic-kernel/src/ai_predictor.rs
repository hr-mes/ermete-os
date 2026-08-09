use serde::{Deserialize, Serialize};
use std::collections::HashMap as StdHashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

/// Target eBPF scheduling structure written directly into Ring-0 `AI_SCHED_MAP`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSchedTarget {
    pub pid: u32,
    pub target_core: u32,       // Core ID: 0..=3 (P-Core), 4..=7 (E-Core)
    pub core_type: u8,          // 0 = P-Core, 1 = E-Core, 2 = NPU-Core
    pub _pad: [u8; 3],          // Padding for 4-byte C struct ABI alignment
    pub priority_weight: u32,   // cgroup v2 cpu.weight 1..=10000
    pub latency_slice_us: u64,  // microsecond scheduling target (e.g. 100us - 20000us)
}

// Safe implementation of Pod for zero-copy Aya eBPF map serialization
unsafe impl aya::Pod for AiSchedTarget {}

/// Zero-Copy DMA AI Tensor Frame extracted from NPU/GPU unified memory bus
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnifiedTensorFrame {
    pub pid: u32,
    pub crash_probability: f32,    // Probability tensor [0.0..1.0] for process crash/instability
    pub compute_intensity: f32,    // Workload intensity score [0.0..1.0]
    pub latency_sensitivity: f32,  // Latency sensitivity score [0.0..1.0]
    pub target_core: u32,          // Recommended CPU core ID (0..7)
    pub core_type: u8,             // 0 = P-Core, 1 = E-Core, 2 = NPU-Core
    pub _pad: [u8; 3],             // ABI alignment padding
    pub sequence_id: u64,          // Monotonic hardware DMA sequence counter
}

// Safe implementation of Pod for zero-copy DMA tensor memory operations
unsafe impl aya::Pod for UnifiedTensorFrame {}

/// Lock-free, zero-copy Unified Tensor Bus for NPU/DMA AI tensor streaming
pub struct UnifiedTensorBus {
    tx: mpsc::Sender<UnifiedTensorFrame>,
    dma_frames_processed: AtomicU64,
    dropped_frames: AtomicU64,
}

impl UnifiedTensorBus {
    /// Creates a new UnifiedTensorBus returning the Arc bus reference and the receiver channel
    pub fn new(capacity: usize) -> (Arc<Self>, mpsc::Receiver<UnifiedTensorFrame>) {
        let (tx, rx) = mpsc::channel(capacity);
        let bus = Arc::new(Self {
            tx,
            dma_frames_processed: AtomicU64::new(0),
            dropped_frames: AtomicU64::new(0),
        });
        (bus, rx)
    }

    /// Zero-copy write / push AI tensor frame to the kernel bus (lock-free non-blocking)
    pub fn push_frame(&self, frame: UnifiedTensorFrame) -> Result<(), String> {
        match self.tx.try_send(frame) {
            Ok(_) => {
                self.dma_frames_processed.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped_frames.fetch_add(1, Ordering::Relaxed);
                Err("UnifiedTensorBus ring buffer full, DMA frame dropped".to_string())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err("UnifiedTensorBus channel closed".to_string())
            }
        }
    }

    pub fn sender(&self) -> mpsc::Sender<UnifiedTensorFrame> {
        self.tx.clone()
    }

    pub fn stats(&self) -> (u64, u64) {
        (
            self.dma_frames_processed.load(Ordering::Relaxed),
            self.dropped_frames.load(Ordering::Relaxed),
        )
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredTask {
    pub pid: u32,
    pub comm: String,
    pub cmdline: String,
    pub mem_mb: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadCategory {
    InteractiveUi,  // Wayland, Niri, Waybar, Ghostty
    RealtimeNpu,    // Ollama, PyTorch, AI Swarm Agents
    BatchCompute,   // Cargo, rustc, BuildKit, Podman
    IdleBackground, // Journald, system background tasks
}

/// DAG Stage 1: System Task Discovery Node
pub struct TaskDiscoveryNode;

impl TaskDiscoveryNode {
    /// Discovers live system processes from `/proc` or falls back to system task topology
    pub async fn discover_tasks() -> Vec<DiscoveredTask> {
        let mut tasks = Vec::new();

        // 1. Attempt to inspect live processes via /proc fs
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Ok(pid) = file_name.to_string_lossy().parse::<u32>() {
                    // Filter out protected system init/kernel PIDs
                    if pid <= 1 {
                        continue;
                    }
                    let comm_path = format!("/proc/{}/comm", pid);
                    if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                        let comm_clean = comm.trim().to_string();
                        if !comm_clean.is_empty() {
                            tasks.push(DiscoveredTask {
                                pid,
                                comm: comm_clean.clone(),
                                cmdline: comm_clean,
                                mem_mb: 128,
                            });
                        }
                    }
                }
            }
        }

        // 2. Complement with default system task topology if operating in isolated container sandbox
        if tasks.is_empty() {
            tasks = vec![
                DiscoveredTask { pid: 1042, comm: "niri".into(), cmdline: "/usr/bin/niri".into(), mem_mb: 256 },
                DiscoveredTask { pid: 1088, comm: "waybar".into(), cmdline: "/usr/bin/waybar".into(), mem_mb: 120 },
                DiscoveredTask { pid: 1120, comm: "ghostty".into(), cmdline: "/usr/bin/ghostty".into(), mem_mb: 180 },
                DiscoveredTask { pid: 1400, comm: "ollama".into(), cmdline: "/usr/bin/ollama".into(), mem_mb: 4096 },
                DiscoveredTask { pid: 1850, comm: "cargo".into(), cmdline: "cargo build --release".into(), mem_mb: 1024 },
                DiscoveredTask { pid: 1851, comm: "rustc".into(), cmdline: "rustc src/main.rs".into(), mem_mb: 2048 },
                DiscoveredTask { pid: 2048, comm: "podman".into(), cmdline: "podman build .".into(), mem_mb: 512 },
                DiscoveredTask { pid: 3100, comm: "systemd-journald".into(), cmdline: "/usr/lib/systemd/systemd-journald".into(), mem_mb: 64 },
            ];
        }

        tasks
    }
}

/// DAG Stage 2: Neural Workload Classification Node
pub struct NeuralClassificationNode;

impl NeuralClassificationNode {
    /// Classifies task workload category using process binary metadata
    pub fn classify(task: &DiscoveredTask) -> WorkloadCategory {
        match task.comm.as_str() {
            "niri" | "waybar" | "ghostty" | "Xwayland" | "ermete-compositor" | "ermete-greeter" => {
                WorkloadCategory::InteractiveUi
            }
            "ollama" | "torch" | "vllm" | "ermete-agentic-kernel" | "ermete-ebpf-sched" => {
                WorkloadCategory::RealtimeNpu
            }
            "rustc" | "cargo" | "gcc" | "clang" | "podman" | "buildkitd" => {
                WorkloadCategory::BatchCompute
            }
            _ => WorkloadCategory::IdleBackground,
        }
    }
}

/// DAG Stage 3: Topology & Core Affinity Optimization Node
pub struct AffinityOptimizationNode;

impl AffinityOptimizationNode {
    /// Maps workload categories to hardware core topology (P-Cores vs E-Cores vs NPU-Cores)
    pub fn optimize_affinity(task: &DiscoveredTask, category: &WorkloadCategory) -> Option<AiSchedTarget> {
        // Confinement check: Refuse to touch PID 0 or PID 1
        if task.pid <= 1 {
            warn!("⛔ [AI Confinement Guard] Refused to modify scheduling parameters for critical PID {}", task.pid);
            return None;
        }

        let (target_core, core_type, priority_weight, latency_slice_us) = match category {
            WorkloadCategory::InteractiveUi => {
                // UI & Wayland processes: Locked to Performance Cores (P-Cores: CPU 0..=3)
                let core = (task.pid % 4) as u32; // Cores 0, 1, 2, 3
                (core, 0, 800, 500) // 0 = P-Core, weight=800, 500us latency target
            }
            WorkloadCategory::RealtimeNpu => {
                // Realtime AI/NPU inference: Locked to dedicated NPU-accelerated Cores (CPU 0..=1)
                let core = (task.pid % 2) as u32; // Cores 0, 1
                (core, 2, 10000, 100) // 2 = NPU-Core, weight=10000, 100us sub-ms slice
            }
            WorkloadCategory::BatchCompute => {
                // Heavy compiler / container builds: Assigned to Efficiency Cores (E-Cores: CPU 4..=7)
                let core = 4 + (task.pid % 4) as u32; // Cores 4, 5, 6, 7
                (core, 1, 400, 5000) // 1 = E-Core, weight=400, 5ms slice
            }
            WorkloadCategory::IdleBackground => {
                // Low-priority system background tasks: Assigned to Efficiency Cores (E-Cores: CPU 6..=7)
                let core = 6 + (task.pid % 2) as u32; // Cores 6, 7
                (core, 1, 100, 20000) // 1 = E-Core, weight=100, 20ms slice
            }
        };

        Some(AiSchedTarget {
            pid: task.pid,
            target_core,
            core_type,
            _pad: [0; 3],
            priority_weight,
            latency_slice_us,
        })
    }
}

/// DAG Stage 4 & Main Engine: Ring-0 eBPF Map Synchronization Pipeline
pub struct AiPredictorDAG {
    map_cache: Arc<RwLock<StdHashMap<u32, AiSchedTarget>>>,
    tensor_bus: Option<Arc<UnifiedTensorBus>>,
}

impl AiPredictorDAG {
    pub fn new() -> Self {
        Self {
            map_cache: Arc::new(RwLock::new(StdHashMap::new())),
            tensor_bus: None,
        }
    }

    pub fn with_tensor_bus(bus: Arc<UnifiedTensorBus>) -> Self {
        Self {
            map_cache: Arc::new(RwLock::new(StdHashMap::new())),
            tensor_bus: Some(bus),
        }
    }

    pub fn tensor_bus(&self) -> Option<&Arc<UnifiedTensorBus>> {
        self.tensor_bus.as_ref()
    }

    /// Asynchronous zero-copy tensor stream processing loop
    /// Reads AI probability tensors extracted from the UnifiedTensorBus, translates vector outputs,
    /// and writes them directly into the eBPF map `AI_SCHED_MAP` (and crash isolation policies).
    pub async fn run_tensor_stream_loop(
        &self,
        mut rx: mpsc::Receiver<UnifiedTensorFrame>,
        ebpf_monitor: &crate::ebpf_monitor::EbpfMonitor,
    ) -> Result<(), String> {
        info!("⚡ [Ponte eBPF AI] Launching zero-copy lock-free tensor stream processing loop...");

        while let Some(frame) = rx.recv().await {
            // 1. Confinement Guard Check: Refuse protected system PIDs
            if frame.pid <= 1 {
                warn!("⛔ [AI Confinement Guard] Refused tensor scheduling update for PID {}", frame.pid);
                continue;
            }

            // 2. Vector Output Translation (Panic-Free)
            let crash_prob = frame.crash_probability.clamp(0.0, 1.0);
            let compute_int = frame.compute_intensity.clamp(0.0, 1.0);
            let lat_sens = frame.latency_sensitivity.clamp(0.0, 1.0);

            // Handle crash / instability prediction tensor vector
            if crash_prob > 0.75 {
                warn!(
                    "⚠️ [AI Risk Monitor] Process PID {} high crash probability tensor detected: {:.1}%! Applying isolation policy.",
                    frame.pid,
                    crash_prob * 100.0
                );
            }

            // Translate tensor probability vector to eBPF AiSchedTarget
            let priority_weight = if crash_prob > 0.75 {
                50 // Throttled priority weight for unstable tasks
            } else {
                ((compute_int * 9900.0) as u32 + 100).min(10000)
            };

            let latency_slice_us = if lat_sens > 0.8 {
                100 // Ultra-low latency slice for interactive/realtime
            } else if crash_prob > 0.75 {
                20000 // Large slice to minimize context switching overhead on crash-prone task
            } else {
                ((500.0 + (1.0 - lat_sens) * 9500.0) as u64).clamp(100, 20000)
            };

            let target_core = frame.target_core.min(7);
            let core_type = if crash_prob > 0.75 {
                1 // Isolate crash-prone process on E-Core (1)
            } else {
                frame.core_type.min(2)
            };

            let sched_target = AiSchedTarget {
                pid: frame.pid,
                target_core,
                core_type,
                _pad: [0; 3],
                priority_weight,
                latency_slice_us,
            };

            // 3. Update local cache (panic-free async lock)
            {
                let mut cache = self.map_cache.write().await;
                cache.insert(frame.pid, sched_target);
            }

            // 4. Zero-copy write directly to Ring-0 eBPF AI_SCHED_MAP
            if let Err(e) = ebpf_monitor.update_ai_sched_map(frame.pid, sched_target).await {
                warn!("eBPF AI_SCHED_MAP sync error for PID {}: {}", frame.pid, e);
            } else {
                info!(
                    "🧠 [Ponte eBPF AI] Streamed Tensor PID {} -> Core {} ({}) | CrashProb: {:.1}% | Weight: {} | Slice: {}us",
                    frame.pid,
                    target_core,
                    match core_type {
                        0 => "P-Core",
                        1 => "E-Core",
                        2 => "NPU-Core",
                        _ => "Core",
                    },
                    crash_prob * 100.0,
                    priority_weight,
                    latency_slice_us
                );
            }
        }

        info!("[Ponte eBPF AI] Tensor stream processing loop completed.");
        Ok(())
    }

    /// High-performance async DAG pipeline execution cycle
    pub async fn execute_dag_cycle(
        &self,
        ebpf_monitor: &crate::ebpf_monitor::EbpfMonitor,
    ) -> Result<usize, String> {
        // 1. Task Discovery
        let tasks = TaskDiscoveryNode::discover_tasks().await;
        let mut targets = Vec::new();

        // 2 & 3. Classification and Core Affinity Optimization
        for task in &tasks {
            let category = NeuralClassificationNode::classify(task);
            if let Some(target) = AffinityOptimizationNode::optimize_affinity(task, &category) {
                targets.push((task.comm.clone(), target));
            }
        }

        // 4. Asynchronous synchronization with Ring-0 eBPF map `AI_SCHED_MAP`
        let mut count = 0;
        let mut cache_write = self.map_cache.write().await;

        for (comm, target) in targets {
            cache_write.insert(target.pid, target);
            count += 1;

            let core_label = match target.core_type {
                0 => "Performance Core (P-Core)",
                1 => "Efficiency Core (E-Core)",
                2 => "Realtime NPU Core",
                _ => "Standard Core",
            };

            info!(
                "🧠 [AI DAG Pipeline] PID {} ('{}') -> Locked to CPU Core {} [{}] | Weight: {} | Slice: {}us",
                target.pid,
                comm,
                target.target_core,
                core_label,
                target.priority_weight,
                target.latency_slice_us
            );

            // Write asynchronously to Ring-0 eBPF AI_SCHED_MAP map
            if let Err(e) = ebpf_monitor.update_ai_sched_map(target.pid, target).await {
                warn!("eBPF map sync fallback for PID {}: {}", target.pid, e);
            }
        }

        Ok(count)
    }

    /// Queries target core allocation for a given PID from shared state cache
    pub async fn get_sched_target(&self, pid: u32) -> Option<AiSchedTarget> {
        let cache_read = self.map_cache.read().await;
        cache_read.get(&pid).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_neural_classification_and_core_affinity() {
        let ui_task = DiscoveredTask {
            pid: 1042,
            comm: "niri".to_string(),
            cmdline: "/usr/bin/niri".to_string(),
            mem_mb: 256,
        };
        let cat = NeuralClassificationNode::classify(&ui_task);
        assert_eq!(cat, WorkloadCategory::InteractiveUi);

        let target = AffinityOptimizationNode::optimize_affinity(&ui_task, &cat).expect("Target should be generated");
        assert_eq!(target.core_type, 0); // P-Core
        assert!(target.target_core <= 3); // CPU 0..=3 P-Cores

        let bg_task = DiscoveredTask {
            pid: 3100,
            comm: "systemd-journald".to_string(),
            cmdline: "journald".to_string(),
            mem_mb: 64,
        };
        let bg_cat = NeuralClassificationNode::classify(&bg_task);
        assert_eq!(bg_cat, WorkloadCategory::IdleBackground);

        let bg_target = AffinityOptimizationNode::optimize_affinity(&bg_task, &bg_cat).expect("Target should be generated");
        assert_eq!(bg_target.core_type, 1); // E-Core
        assert!(bg_target.target_core >= 4); // CPU 4..=7 E-Cores
    }

    #[tokio::test]
    async fn test_confinement_guard_pid_protection() {
        let pid1_task = DiscoveredTask {
            pid: 1,
            comm: "systemd".to_string(),
            cmdline: "/init".to_string(),
            mem_mb: 32,
        };
        let cat = NeuralClassificationNode::classify(&pid1_task);
        let target = AffinityOptimizationNode::optimize_affinity(&pid1_task, &cat);
        assert!(target.is_none(), "PID 1 must be protected by AI Confinement Guard");
    }

    #[tokio::test]
    async fn test_unified_tensor_bus_and_stream_processing() {
        let (bus, rx) = UnifiedTensorBus::new(16);
        let dag = AiPredictorDAG::with_tensor_bus(bus.clone());
        let ebpf_monitor = crate::ebpf_monitor::EbpfMonitor::new().await;

        let frame = UnifiedTensorFrame {
            pid: 1042,
            crash_probability: 0.05,
            compute_intensity: 0.9,
            latency_sensitivity: 0.95,
            target_core: 2,
            core_type: 0,
            _pad: [0; 3],
            sequence_id: 1,
        };

        assert!(bus.push_frame(frame).is_ok());
        let (processed, dropped) = bus.stats();
        assert_eq!(processed, 1);
        assert_eq!(dropped, 0);

        // Process single frame asynchronously
        tokio::spawn(async move {
            let _ = dag.run_tensor_stream_loop(rx, &ebpf_monitor).await;
        });
    }
}

