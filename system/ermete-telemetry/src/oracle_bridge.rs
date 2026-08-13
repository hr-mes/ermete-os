use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use zbus::Connection;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub anomaly_score: f32,
    pub target_unit: String,
    pub predicted_failure_mode: String,
    pub suggested_intent: String,
    pub confidence: f32,
    pub embedding_vector: Vec<f32>,
    pub timestamp: String,
}

pub struct OracleBridge {
    dbus_conn: Option<Connection>,
    report_receiver: mpsc::Receiver<AnomalyReport>,
}

impl OracleBridge {
    pub async fn new(report_receiver: mpsc::Receiver<AnomalyReport>) -> Self {
        // Attempt system bus first, then session bus
        let dbus_conn = match Connection::system().await {
            Ok(c) => Some(c),
            Err(err) => match Connection::session().await {
                Ok(c) => Some(c),
                Err(e) => {
                    warn!("Failed system ({:?}) and session ({:?}) DBus connections.", err, e);
                    None
                }
            },
        };

        if dbus_conn.is_some() {
            info!("⚡ Oracle Bridge initialized DBus connection to `org.ermete.InitOracle`.");
        } else {
            warn!("⚠️ DBus connection unavailable. Init Oracle self-healing actions will operate in simulation mode.");
        }

        Self {
            dbus_conn,
            report_receiver,
        }
    }

    pub async fn run_loop(mut self) {
        info!("🩺 Self-healing Oracle Bridge listening for predictive anomaly triggers...");

        while let Some(report) = self.report_receiver.recv().await {
            info!(
                "🛠️ [Self-Healing Action] Anomaly score: {:.2} for unit '{}'. Executing remediation intent...",
                report.anomaly_score, report.target_unit
            );

            if let Err(e) = self.execute_remediation(&report).await {
                error!("Failed to execute remediation via Init Oracle: {}", e);
            }
        }
    }

    async fn execute_remediation(&self, report: &AnomalyReport) -> Result<()> {
        let intent_text = format!(
            "PREDICTIVE_AUTOCURE: intent='{}', failure_mode='{}', target_unit='{}', score={:.2}",
            report.suggested_intent, report.predicted_failure_mode, report.target_unit, report.anomaly_score
        );

        if let Some(conn) = &self.dbus_conn {
            let reply: Result<String, zbus::Error> = conn
                .call_method(
                    Some("org.ermete.InitOracle"),
                    "/org/ermete/InitOracle",
                    Some("org.ermete.InitOracle"),
                    "submit_intent",
                    &(intent_text.as_str()),
                )
                .await
                .and_then(|r| r.body().deserialize());

            match reply {
                Ok(resp) => {
                    info!("✅ Init Oracle response to auto-cure intent: {}", resp);
                }
                Err(e) => {
                    warn!("Could not call Init Oracle DBus method ({}) - falling back to simulated execution.", e);
                }
            }
        } else {
            info!("DRY RUN (Simulated Auto-Cure): Submitted intent '{}' to Init Oracle.", intent_text);
        }

        Ok(())
    }
}
