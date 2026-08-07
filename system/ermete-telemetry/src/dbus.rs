use crate::ai_engine::AnomalyReport;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use zbus::interface;

pub struct TelemetryMetrics {
    pub total_records_parsed: AtomicU64,
    pub total_batches_processed: AtomicU64,
    pub total_anomalies_detected: AtomicU64,
}

impl TelemetryMetrics {
    pub fn new() -> Self {
        Self {
            total_records_parsed: AtomicU64::new(0),
            total_batches_processed: AtomicU64::new(0),
            total_anomalies_detected: AtomicU64::new(0),
        }
    }
}

pub struct TelemetryDbusInterface {
    metrics: Arc<TelemetryMetrics>,
    anomaly_trigger_sender: mpsc::Sender<AnomalyReport>,
}

impl TelemetryDbusInterface {
    pub fn new(
        metrics: Arc<TelemetryMetrics>,
        anomaly_trigger_sender: mpsc::Sender<AnomalyReport>,
    ) -> Self {
        Self {
            metrics,
            anomaly_trigger_sender,
        }
    }
}

#[interface(name = "org.ermete.Telemetry")]
impl TelemetryDbusInterface {
    async fn status(&self) -> String {
        serde_json::json!({
            "daemon": "ermete-telemetry",
            "version": "1.0.0",
            "status": "ONLINE",
            "architecture": "AI Predictive Log-Aggregator & Self-Healing",
            "records_parsed": self.metrics.total_records_parsed.load(Ordering::Relaxed),
            "batches_processed": self.metrics.total_batches_processed.load(Ordering::Relaxed),
            "anomalies_detected": self.metrics.total_anomalies_detected.load(Ordering::Relaxed),
        })
        .to_string()
    }

    async fn get_telemetry_metrics(&self) -> String {
        serde_json::json!({
            "total_records_parsed": self.metrics.total_records_parsed.load(Ordering::Relaxed),
            "total_batches_processed": self.metrics.total_batches_processed.load(Ordering::Relaxed),
            "total_anomalies_detected": self.metrics.total_anomalies_detected.load(Ordering::Relaxed),
        })
        .to_string()
    }

    async fn trigger_anomaly_check(&self, unit_name: String) -> String {
        let report = AnomalyReport {
            anomaly_score: 0.99,
            target_unit: unit_name.clone(),
            predicted_failure_mode: "MANUAL_TEST_TRIGGER".to_string(),
            suggested_intent: format!("RESTART_UNIT: {}", unit_name),
            confidence: 1.0,
            embedding_vector: vec![0.5; 16],
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.metrics.total_anomalies_detected.fetch_add(1, Ordering::Relaxed);

        if self.anomaly_trigger_sender.send(report).await.is_ok() {
            serde_json::json!({
                "success": true,
                "message": format!("Manual predictive anomaly check triggered for unit '{}'", unit_name)
            })
            .to_string()
        } else {
            serde_json::json!({
                "success": false,
                "error": "Failed to route manual anomaly trigger to Init Oracle channel"
            })
            .to_string()
        }
    }
}
