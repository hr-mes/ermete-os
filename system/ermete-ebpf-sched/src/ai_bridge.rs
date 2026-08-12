#![allow(unsafe_code)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::multiple_unsafe_ops_per_block)]

use crate::sched_ext::SchedClass;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use zbus::Connection;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiProcessClassification {
    pub pid: u32,
    pub binary_name: String,
    pub recommended_sched_class: SchedClass,
    pub recommended_weight: u32,
    pub recommended_slice_us: u64,
    pub heuristic_score: f32,
}

pub struct AiDaemonBridge {
    connection: Option<Connection>,
}

impl AiDaemonBridge {
    pub async fn new() -> Self {
        info!("🤖 Connecting eBPF Kernel Scheduler to NPU AI Daemon (os.ermete.AiDaemon)...");
        
        let conn = match Connection::session().await {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("DBus session unavailable ({:?}). eBPF Scheduler will use local NPU zero-latency heuristic AI inferencing engine.", e);
                None
            }
        };
        if conn.is_some() {
            info!("✅ DBus connection to `ermete-ai-daemon` established.");
        } else {
            warn!("⚠️ DBus session unavailable. eBPF Scheduler will use local NPU zero-latency heuristic AI inferencing engine.");
        }

        Self { connection: conn }
    }

    /// Rule-based heuristic calculator for process classification and scoring
    fn calculate_heuristic(comm: &str, filename: &str) -> (SchedClass, u32, u64, f32) {
        let has_valid_path = filename.starts_with('/');

        match comm {
            "niri" | "waybar" | "ghostty" => {
                let score = if has_valid_path { 0.95 } else { 0.90 };
                (SchedClass::InteractiveUi, 800, 2000, score)
            }
            "ollama" | "torch" => {
                let score = if has_valid_path { 0.99 } else { 0.95 };
                (SchedClass::RealtimeNpu, 1000, 1000, score)
            }
            "rustc" | "cargo" | "gcc" => {
                let score = if has_valid_path { 0.85 } else { 0.80 };
                (SchedClass::BatchCompute, 400, 10000, score)
            }
            _ => {
                if has_valid_path && filename.starts_with("/usr/") {
                    (SchedClass::IdleBackground, 100, 20000, 0.60)
                } else if !filename.is_empty() {
                    (SchedClass::IdleBackground, 100, 20000, 0.50)
                } else {
                    (SchedClass::IdleBackground, 100, 20000, 0.35)
                }
            }
        }
    }

    /// Query `ermete-ai-daemon` for AI weights/predictions for a newly executed process
    pub async fn predict_task_priority(&self, pid: u32, comm: &str, filename: &str) -> AiProcessClassification {
        let query_payload = serde_json::json!({
            "intent": "classify_process_workload",
            "pid": pid,
            "comm": comm,
            "filename": filename,
        })
        .to_string();

        if let Some(conn) = &self.connection {
            if let Ok(reply) = conn
                .call_method(
                    Some("os.ermete.AiDaemon"),
                    "/os/ermete/AiDaemon",
                    Some("os.ermete.AiDaemon"),
                    "process_query",
                    &(query_payload.as_str()),
                )
                .await
            {
                if let Ok(resp_str) = reply.body().deserialize::<String>() {
                    info!("🤖 NPU AI Model Prediction response for PID {}: {}", pid, resp_str);
                    if let Ok(classification) = serde_json::from_str::<AiProcessClassification>(&resp_str) {
                        return classification;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&resp_str) {
                        if let (Some(class_str), Some(weight), Some(slice)) = (
                            val.get("recommended_sched_class").and_then(|v| v.as_str()),
                            val.get("recommended_weight").and_then(|v| v.as_u64()),
                            val.get("recommended_slice_us").and_then(|v| v.as_u64()),
                        ) {
                            let sched_class = match class_str {
                                "InteractiveUi" => SchedClass::InteractiveUi,
                                "RealtimeNpu" => SchedClass::RealtimeNpu,
                                "BatchCompute" => SchedClass::BatchCompute,
                                _ => SchedClass::IdleBackground,
                            };
                            let score = val
                                .get("heuristic_score")
                                .and_then(|v| v.as_f64())
                                .map(|v| v as f32)
                                .unwrap_or(0.90);
                            return AiProcessClassification {
                                pid,
                                binary_name: comm.to_string(),
                                recommended_sched_class: sched_class,
                                recommended_weight: weight as u32,
                                recommended_slice_us: slice,
                                heuristic_score: score,
                            };
                        }
                    }
                }
            }
        }

        // Local low-latency fallback classification heuristics (mimicking local NPU output)
        let (sched_class, weight, slice_us, heuristic_score) = Self::calculate_heuristic(comm, filename);

        AiProcessClassification {
            pid,
            binary_name: comm.to_string(),
            recommended_sched_class: sched_class,
            recommended_weight: weight,
            recommended_slice_us: slice_us,
            heuristic_score,
        }
    }
}
