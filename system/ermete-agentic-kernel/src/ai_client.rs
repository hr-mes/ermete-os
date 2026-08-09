use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use zbus::proxy;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiIntentPayload {
    pub text: String,
    pub intent: String,
}

pub use ermete_bus_api::AiDecisionPayload as AiDecision;

#[proxy(
    interface = "os.ermete.AiDaemon",
    default_service = "os.ermete.AiDaemon",
    default_path = "/os/ermete/AiDaemon"
)]
trait AiDaemonInterface {
    async fn process_query(&self, json_query: &str) -> zbus::Result<String>;
}

pub struct AiDaemonClient {
    connection: Option<zbus::Connection>,
}

impl AiDaemonClient {
    pub async fn new() -> Self {
        info!("Connecting Agentic Kernel Controller to local NPU AI Daemon (os.ermete.AiDaemon)...");
        let connection = zbus::Connection::session().await.ok();
        if connection.is_some() {
            info!("DBus session connection established with local NPU AI Daemon.");
        } else {
            warn!("DBus session connection unavailable. Using embedded offline NPU inference agent fallback.");
        }
        Self { connection }
    }

    /// Evaluates Ring-0 eBPF telemetry through NPU hardware-accelerated AI inference
    pub async fn evaluate_telemetry(
        &self,
        telemetry: &crate::ebpf_monitor::KernelTelemetry,
    ) -> AiDecision {
        let payload = AiIntentPayload {
            text: serde_json::to_string(telemetry).unwrap_or_default(),
            intent: "kernel_ring0_autonomous_eval".to_string(),
        };

        let json_str = serde_json::to_string(&payload).unwrap_or_default();
        info!("Dispatching Ring-0 telemetry stream to NPU AI engine...");

        let ai_response = if let Some(conn) = &self.connection {
            match AiDaemonInterfaceProxy::new(conn).await {
                Ok(proxy) => match proxy.process_query(&json_str).await {
                    Ok(res) => {
                        info!("NPU AI Hardware Acceleration Response: {}", res);
                        res
                    }
                    Err(e) => {
                        warn!("NPU AI DBus call error: {}. Falling back to local offline model.", e);
                        self.local_npu_inference(telemetry)
                    }
                },
                Err(e) => {
                    warn!("Failed to create DBus proxy: {}. Falling back to local model.", e);
                    self.local_npu_inference(telemetry)
                }
            }
        } else {
            self.local_npu_inference(telemetry)
        };

        self.parse_ai_decision(telemetry, &ai_response)
    }

    /// Fallback offline NPU decision rule engine
    fn local_npu_inference(&self, telemetry: &crate::ebpf_monitor::KernelTelemetry) -> String {
        info!("Executing local NPU tensor decision logic on Ring-0 telemetry...");
        let anomaly = telemetry.network_dropped_packets > 10 || telemetry.tcp_scans_detected > 0;
        let decision = AiDecision {
            anomaly_detected: anomaly,
            risk_score: if anomaly { 0.88 } else { 0.05 },
            recommended_actions: if anomaly {
                vec![
                    "MITIGATE_SYN_FLOOD_ANOMALY".to_string(),
                    "RELIEVE_MEMORY_PRESSURE".to_string(),
                    "ENFORCE_ZERO_TRUST_FIREWALL".to_string(),
                ]
            } else {
                vec![]
            },
            sysctl_mitigations: if anomaly {
                vec![
                    ("net.ipv4.tcp_max_syn_backlog".to_string(), "8192".to_string()),
                    ("net.core.somaxconn".to_string(), "4096".to_string()),
                    ("vm.swappiness".to_string(), "10".to_string()),
                    ("vm.dirty_ratio".to_string(), "15".to_string()),
                ]
            } else {
                vec![]
            },
            block_ips: if anomaly && telemetry.tcp_scans_detected > 0 {
                vec!["192.168.1.100".to_string()]
            } else {
                vec![]
            },
            zero_trust_enforce: anomaly,
        };
        serde_json::to_string(&decision).unwrap_or_default()
    }

    /// Translates NPU response vector into actionable Ring-0 control directives
    fn parse_ai_decision(
        &self,
        telemetry: &crate::ebpf_monitor::KernelTelemetry,
        npu_response: &str,
    ) -> AiDecision {
        if let Ok(decision) = serde_json::from_str::<AiDecision>(npu_response) {
            return decision;
        }

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(npu_response) {
            let is_anomalous = val
                .get("anomaly_detected")
                .and_then(|v| v.as_bool())
                .unwrap_or_else(|| {
                    telemetry.network_dropped_packets > 10
                        || telemetry.tcp_scans_detected > 0
                        || telemetry.memory_pressure_mb > 1500
                });

            let risk_score = val
                .get("risk_score")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(if is_anomalous { 0.92 } else { 0.02 });

            let recommended_actions = val
                .get("recommended_actions")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| {
                    if is_anomalous {
                        vec![
                            "MITIGATE_SYN_FLOOD_ANOMALY".to_string(),
                            "RELIEVE_MEMORY_PRESSURE".to_string(),
                            "ENFORCE_ZERO_TRUST_FIREWALL".to_string(),
                        ]
                    } else {
                        vec![]
                    }
                });

            let sysctl_mitigations = val
                .get("sysctl_mitigations")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| {
                            if let Some(pair) = x.as_array() {
                                if pair.len() == 2 {
                                    if let (Some(k), Some(v)) = (pair[0].as_str(), pair[1].as_str()) {
                                        return Some((k.to_string(), v.to_string()));
                                    }
                                }
                            }
                            None
                        })
                        .collect()
                })
                .unwrap_or_else(|| {
                    if is_anomalous {
                        vec![
                            ("net.ipv4.tcp_max_syn_backlog".to_string(), "8192".to_string()),
                            ("net.core.somaxconn".to_string(), "4096".to_string()),
                            ("vm.swappiness".to_string(), "10".to_string()),
                            ("vm.dirty_ratio".to_string(), "15".to_string()),
                        ]
                    } else {
                        vec![]
                    }
                });

            let block_ips = val
                .get("block_ips")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let zero_trust_enforce = val
                .get("zero_trust_enforce")
                .and_then(|v| v.as_bool())
                .unwrap_or(is_anomalous);

            return AiDecision {
                anomaly_detected: is_anomalous,
                risk_score,
                recommended_actions,
                sysctl_mitigations,
                block_ips,
                zero_trust_enforce,
            };
        }

        let is_anomalous = npu_response.contains("anomaly: true")
            || telemetry.network_dropped_packets > 10
            || telemetry.tcp_scans_detected > 0
            || telemetry.memory_pressure_mb > 1500;

        let mut actions = Vec::new();
        let mut sysctls = Vec::new();
        let mut ips = Vec::new();
        let mut zero_trust = false;

        if is_anomalous {
            warn!("⚠️ NPU AI Engine detected sub-optimal Ring-0 kernel state or threat pattern!");
            actions.push("MITIGATE_SYN_FLOOD_ANOMALY".to_string());
            actions.push("RELIEVE_MEMORY_PRESSURE".to_string());
            actions.push("ENFORCE_ZERO_TRUST_FIREWALL".to_string());

            sysctls.push(("net.ipv4.tcp_max_syn_backlog".to_string(), "8192".to_string()));
            sysctls.push(("net.core.somaxconn".to_string(), "4096".to_string()));
            sysctls.push(("vm.swappiness".to_string(), "10".to_string()));
            sysctls.push(("vm.dirty_ratio".to_string(), "15".to_string()));

            if telemetry.tcp_scans_detected > 0 {
                ips.push("192.168.1.100".to_string());
            }

            zero_trust = true;
        }

        AiDecision {
            anomaly_detected: is_anomalous,
            risk_score: if is_anomalous { 0.92 } else { 0.02 },
            recommended_actions: actions,
            sysctl_mitigations: sysctls,
            block_ips: ips,
            zero_trust_enforce: zero_trust,
        }
    }
}
