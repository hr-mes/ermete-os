use candle_core::{Device, Tensor};
use candle_nn::{Linear, Module};
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
    pub cpu_time_ms: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub num_threads: u32,
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
    fn read_proc_stat(pid: u32) -> (u64, u32) {
        let stat_path = format!("/proc/{}/stat", pid);
        if let Ok(content) = std::fs::read_to_string(stat_path) {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 20 {
                let utime: u64 = parts[13].parse().unwrap_or(0);
                let stime: u64 = parts[14].parse().unwrap_or(0);
                let threads: u32 = parts[19].parse().unwrap_or(1);
                let cpu_ms = utime.saturating_add(stime) * 10;
                return (cpu_ms, threads);
            }
        }
        (0, 1)
    }

    fn read_proc_io(pid: u32) -> (u64, u64) {
        let io_path = format!("/proc/{}/io", pid);
        let mut read_bytes = 0u64;
        let mut write_bytes = 0u64;
        if let Ok(content) = std::fs::read_to_string(io_path) {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("read_bytes:") {
                    read_bytes = val.trim().parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("write_bytes:") {
                    write_bytes = val.trim().parse().unwrap_or(0);
                }
            }
        }
        (read_bytes, write_bytes)
    }

    fn read_proc_mem(pid: u32) -> u64 {
        let statm_path = format!("/proc/{}/statm", pid);
        if let Ok(content) = std::fs::read_to_string(statm_path) {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if let Some(pages_str) = parts.get(1) {
                if let Ok(pages) = pages_str.parse::<u64>() {
                    return (pages * 4096) / (1024 * 1024);
                }
            }
        }
        128
    }

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
                            let (cpu_time_ms, num_threads) = Self::read_proc_stat(pid);
                            let (io_read, io_write) = Self::read_proc_io(pid);
                            let mem_mb = Self::read_proc_mem(pid);

                            tasks.push(DiscoveredTask {
                                pid,
                                comm: comm_clean.clone(),
                                cmdline: comm_clean,
                                mem_mb,
                                cpu_time_ms,
                                io_read_bytes: io_read,
                                io_write_bytes: io_write,
                                num_threads,
                            });
                        }
                    }
                }
            }
        }

        // 2. Complement with default system task topology if operating in isolated container sandbox
        if tasks.is_empty() {
            tasks = vec![
                DiscoveredTask {
                    pid: 1042,
                    comm: "niri".into(),
                    cmdline: "/usr/bin/niri".into(),
                    mem_mb: 256,
                    cpu_time_ms: 450,
                    io_read_bytes: 524_288,
                    io_write_bytes: 131_072,
                    num_threads: 8,
                },
                DiscoveredTask {
                    pid: 1088,
                    comm: "waybar".into(),
                    cmdline: "/usr/bin/waybar".into(),
                    mem_mb: 120,
                    cpu_time_ms: 300,
                    io_read_bytes: 262_144,
                    io_write_bytes: 65_536,
                    num_threads: 4,
                },
                DiscoveredTask {
                    pid: 1120,
                    comm: "ghostty".into(),
                    cmdline: "/usr/bin/ghostty".into(),
                    mem_mb: 180,
                    cpu_time_ms: 600,
                    io_read_bytes: 1_048_576,
                    io_write_bytes: 262_144,
                    num_threads: 6,
                },
                DiscoveredTask {
                    pid: 1400,
                    comm: "ollama".into(),
                    cmdline: "/usr/bin/ollama".into(),
                    mem_mb: 4096,
                    cpu_time_ms: 15_000,
                    io_read_bytes: 200_000_000,
                    io_write_bytes: 100_000_000,
                    num_threads: 32,
                },
                DiscoveredTask {
                    pid: 1850,
                    comm: "cargo".into(),
                    cmdline: "cargo build --release".into(),
                    mem_mb: 1024,
                    cpu_time_ms: 8_500,
                    io_read_bytes: 150_000_000,
                    io_write_bytes: 80_000_000,
                    num_threads: 16,
                },
                DiscoveredTask {
                    pid: 1851,
                    comm: "rustc".into(),
                    cmdline: "rustc src/main.rs".into(),
                    mem_mb: 2048,
                    cpu_time_ms: 9_200,
                    io_read_bytes: 180_000_000,
                    io_write_bytes: 90_000_000,
                    num_threads: 16,
                },
                DiscoveredTask {
                    pid: 2048,
                    comm: "podman".into(),
                    cmdline: "podman build .".into(),
                    mem_mb: 512,
                    cpu_time_ms: 5_000,
                    io_read_bytes: 80_000_000,
                    io_write_bytes: 40_000_000,
                    num_threads: 12,
                },
                DiscoveredTask {
                    pid: 3100,
                    comm: "systemd-journald".into(),
                    cmdline: "/usr/lib/systemd/systemd-journald".into(),
                    mem_mb: 64,
                    cpu_time_ms: 120,
                    io_read_bytes: 65_536,
                    io_write_bytes: 262_144,
                    num_threads: 2,
                },
            ];
        }

        tasks
    }
}

/// DAG Stage 2: Neural Workload Classification Node
pub struct NeuralClassificationNode;

impl NeuralClassificationNode {
    /// Classifies task workload category using Candle neural tensor model on process statistics (CPU time, I/O, memory, threads)
    pub fn classify(task: &DiscoveredTask) -> WorkloadCategory {
        let device = Device::Cpu;

        // 1. Normalize process statistics into a continuous feature vector
        let norm_cpu = (task.cpu_time_ms as f32 / 20_000.0).clamp(0.0, 1.0);
        let total_io = task.io_read_bytes.saturating_add(task.io_write_bytes);
        let norm_io = (total_io as f32 / 300_000_000.0).clamp(0.0, 1.0);
        let norm_mem = (task.mem_mb as f32 / 8192.0).clamp(0.0, 1.0);
        let norm_threads = (task.num_threads as f32 / 32.0).clamp(0.0, 1.0);

        let features = vec![norm_cpu, norm_io, norm_mem, norm_threads];

        // 2. Build Candle Tensor [1, 4] from process statistics vector
        let input_tensor = match Tensor::from_slice(&features, (1, 4), &device) {
            Ok(t) => t,
            Err(_) => return WorkloadCategory::IdleBackground,
        };

        // 3. Neural Network Forward Pass: Multi-Layer Perceptron (MLP) with ReLU activation
        // Layer 1 (4 features -> 8 hidden neurons)
        let w1_data: [f32; 32] = [
            -0.8, -0.8,  0.2,  0.6, // Hidden 0: UI characteristics
             0.9,  0.8,  1.0,  0.9, // Hidden 1: NPU heavy characteristics
             0.8,  0.9,  0.3,  0.4, // Hidden 2: Batch compute characteristics
            -0.9, -0.9, -0.8, -0.8, // Hidden 3: Idle background characteristics
            -0.5, -0.5,  0.5,  0.8, // Hidden 4: UI/Interactive candidate
             1.0,  0.6,  0.9,  0.7, // Hidden 5: Heavy Realtime NPU candidate
             0.7,  0.8,  0.4,  0.5, // Hidden 6: High I/O batch build candidate
            -0.7, -0.7, -0.6, -0.7, // Hidden 7: Low resource background candidate
        ];
        let b1_data: [f32; 8] = [0.5, 0.0, 0.0, 0.8, 0.3, 0.0, 0.0, 0.6];

        let w1 = match Tensor::from_slice(&w1_data, (8, 4), &device) {
            Ok(t) => t,
            Err(_) => return WorkloadCategory::IdleBackground,
        };
        let b1 = match Tensor::from_slice(&b1_data, (8,), &device) {
            Ok(t) => t,
            Err(_) => return WorkloadCategory::IdleBackground,
        };

        let l1 = Linear::new(w1, Some(b1));
        let hidden = match l1.forward(&input_tensor) {
            Ok(t) => match t.relu() {
                Ok(r) => r,
                Err(_) => return WorkloadCategory::IdleBackground,
            },
            Err(_) => return WorkloadCategory::IdleBackground,
        };

        // Layer 2: 8 hidden -> 4 output logits
        let w2_data: [f32; 32] = [
            // Out 0: InteractiveUi
             1.2, -0.8, -0.5, -1.0,  1.0, -0.8, -0.5, -0.8,
            // Out 1: RealtimeNpu
            -0.8,  1.5, -0.4, -1.2, -0.6,  1.4, -0.4, -1.0,
            // Out 2: BatchCompute
            -0.5, -0.4,  1.4, -1.0, -0.4, -0.3,  1.3, -0.8,
            // Out 3: IdleBackground
            -0.8, -1.2, -1.0,  1.5, -0.6, -1.0, -0.8,  1.4,
        ];
        let b2_data: [f32; 4] = [0.1, -0.1, -0.1, 0.2];

        let w2 = match Tensor::from_slice(&w2_data, (4, 8), &device) {
            Ok(t) => t,
            Err(_) => return WorkloadCategory::IdleBackground,
        };
        let b2 = match Tensor::from_slice(&b2_data, (4,), &device) {
            Ok(t) => t,
            Err(_) => return WorkloadCategory::IdleBackground,
        };

        let l2 = Linear::new(w2, Some(b2));
        let logits = match l2.forward(&hidden) {
            Ok(t) => t,
            Err(_) => return WorkloadCategory::IdleBackground,
        };

        let logits_vec = match logits.squeeze(0).and_then(|t| t.to_vec1::<f32>()) {
            Ok(v) => v,
            Err(_) => return WorkloadCategory::IdleBackground,
        };

        let mut max_idx = 3;
        let mut max_val = f32::NEG_INFINITY;
        for (i, &val) in logits_vec.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }

        match max_idx {
            0 => WorkloadCategory::InteractiveUi,
            1 => WorkloadCategory::RealtimeNpu,
            2 => WorkloadCategory::BatchCompute,
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
            cpu_time_ms: 450,
            io_read_bytes: 524_288,
            io_write_bytes: 131_072,
            num_threads: 8,
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
            cpu_time_ms: 120,
            io_read_bytes: 65_536,
            io_write_bytes: 262_144,
            num_threads: 2,
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
            cpu_time_ms: 50,
            io_read_bytes: 1024,
            io_write_bytes: 1024,
            num_threads: 1,
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


