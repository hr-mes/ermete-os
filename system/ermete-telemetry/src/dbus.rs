//! D-Bus surface for `org.ermete.Telemetry`: status/metrics queries and a manual
//! anomaly-check trigger. The polkit gate comes from `ermete_bus_api::polkit` (the shared
//! copy that replaced the per-daemon duplicates, CQ-01).

use crate::oracle_bridge::AnomalyReport;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use ermete_bus_api::polkit::check_polkit_auth_zbus;
use zbus::interface;

/// Process-lifetime counters shared (via `Arc`) between the D-Bus interface
/// and the daemon's background tasks, exposed through `status`/
/// `get_telemetry_metrics`.
pub struct TelemetryMetrics {
    pub total_records_parsed: AtomicU64,
    pub total_batches_processed: AtomicU64,
    pub total_anomalies_detected: AtomicU64,
}

impl TelemetryMetrics {
    /// Creates a zeroed counter set.
    pub fn new() -> Self {
        Self {
            total_records_parsed: AtomicU64::new(0),
            total_batches_processed: AtomicU64::new(0),
            total_anomalies_detected: AtomicU64::new(0),
        }
    }
}

/// D-Bus object implementing `org.ermete.Telemetry`, serving live metrics and
/// a manual anomaly-trigger method that feeds into the same
/// [`AnomalyReport`] channel the log-analyzer task uses.
pub struct TelemetryDbusInterface {
    metrics: Arc<TelemetryMetrics>,
    anomaly_trigger_sender: mpsc::Sender<AnomalyReport>,
}

impl TelemetryDbusInterface {
    /// Builds the interface object, sharing `metrics` with the rest of the
    /// daemon and routing triggered anomaly checks through
    /// `anomaly_trigger_sender`.
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
    /// D-Bus method: returns a JSON status blob (daemon name/version, an
    /// architecture label, and the current metric counters). No
    /// authorization is required to call this — it's read-only.
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

    /// D-Bus method: returns just the raw metric counters as JSON (a subset
    /// of [`Self::status`]'s payload). Also unauthenticated/read-only.
    async fn get_telemetry_metrics(&self) -> String {
        serde_json::json!({
            "total_records_parsed": self.metrics.total_records_parsed.load(Ordering::Relaxed),
            "total_batches_processed": self.metrics.total_batches_processed.load(Ordering::Relaxed),
            "total_anomalies_detected": self.metrics.total_anomalies_detected.load(Ordering::Relaxed),
        })
        .to_string()
    }

    /// D-Bus method: manually enqueues a synthetic, maximum-confidence
    /// [`AnomalyReport`] for `unit_name` (`predicted_failure_mode =
    /// "MANUAL_TEST_TRIGGER"`), routed through the same channel the
    /// automated log-analyzer and [`crate::oracle_bridge`] use — useful for
    /// testing the self-healing pipeline without waiting for a real anomaly.
    /// Requires Polkit authorization for `org.ermete.telemetry.trigger`.
    ///
    /// # Errors
    /// Returns `AccessDenied` if there's no D-Bus sender or the Polkit check
    /// fails/denies, and `Failed` if the internal channel send fails (e.g. the
    /// receiving task has already shut down).
    async fn trigger_anomaly_check(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
        unit_name: String,
    ) -> zbus::fdo::Result<String> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.telemetry.trigger", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for trigger_anomaly_check".into()));
        }

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
            Ok(serde_json::json!({
                "success": true,
                "message": format!("Manual predictive anomaly check triggered for unit '{}'", unit_name)
            })
            .to_string())
        } else {
            Err(zbus::fdo::Error::Failed("Failed to route manual anomaly trigger to Init Oracle channel".into()))
        }
    }
}
