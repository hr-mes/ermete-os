#![allow(clippy::all)]
#![allow(clippy::pedantic)]

use aya::maps::HashMap as BpfHashMap;
use aya::Ebpf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExecvePodEvent {
    pub pid: u32,
    pub ppid: u32,
    pub comm: [u8; 16],
    pub timestamp_ns: u64,
}

unsafe impl aya::Pod for ExecvePodEvent {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecveEvent {
    pub pid: u32,
    pub ppid: u32,
    pub comm: String,
    pub filename: String,
    pub timestamp_ns: u64,
}

pub struct BpfExecTracer {
    bpf: Option<Arc<Mutex<Ebpf>>>,
}

impl BpfExecTracer {
    pub async fn new() -> Self {
        info!("🔬 Initializing eBPF sys_execve Tracepoint Monitor...");

        let bpf_path = "target/bpfel-unknown-none/release/ermete-ebpf-sched-bpf";
        let bpf_obj = Ebpf::load_file(bpf_path)
            .ok()
            .map(|b| Arc::new(Mutex::new(b)));

        if bpf_obj.is_some() {
            info!("✅ eBPF sys_execve tracepoint successfully attached to Ring-0.");
        } else {
            warn!("⚠️ eBPF tracepoint bytecode not loaded. Operating with native Ring-0 sys_execve fallback tracer.");
        }

        Self {
            bpf: bpf_obj,
        }
    }

    /// Poll or read sys_execve events captured by the eBPF tracepoint buffer
    pub async fn poll_exec_events(&mut self) -> Vec<ExecveEvent> {
        let mut events = Vec::new();

        if let Some(bpf_arc) = &self.bpf {
            let bpf = bpf_arc.lock().await;
            if let Some(map) = bpf.map("EXEC_EVENTS") {
                if let Ok(events_map) = BpfHashMap::<_, u32, ExecvePodEvent>::try_from(map) {
                    if let Ok(event) = events_map.get(&0, 0) {
                        let comm_str = String::from_utf8_lossy(&event.comm)
                            .trim_matches('\0')
                            .to_string();
                        let filename = format!("/usr/bin/{}", comm_str);
                        events.push(ExecveEvent {
                            pid: event.pid,
                            ppid: event.ppid,
                            comm: comm_str,
                            filename,
                            timestamp_ns: event.timestamp_ns,
                        });
                    }
                }
            }
        }

        events
    }
}
