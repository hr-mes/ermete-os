use crate::ai_engine::AnomalyReport;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use zbus::interface;
use zbus::zvariant::{OwnedValue, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitSubject {
    pub kind: String,
    pub details: HashMap<String, OwnedValue>,
}

impl PolkitSubject {
    pub fn system_bus_name(name: impl Into<String>) -> Self {
        let mut details = HashMap::new();
        let val: Value = Value::from(name.into());
        if let Ok(owned) = val.try_into() {
            details.insert("name".to_string(), owned);
        }
        Self {
            kind: "system-bus-name".to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitAuthorizationResult {
    pub is_authorized: bool,
    pub is_challenge: bool,
    pub details: HashMap<String, String>,
}

#[zbus::proxy(
    interface = "org.freedesktop.PolicyKit1.Authority",
    default_service = "org.freedesktop.PolicyKit1",
    default_path = "/org/freedesktop/PolicyKit1/Authority"
)]
pub trait PolicyKitAuthority {
    fn check_authorization(
        &self,
        subject: &PolkitSubject,
        action_id: &str,
        details: &HashMap<&str, &str>,
        flags: u32,
        cancellation_id: &str,
    ) -> zbus::Result<PolkitAuthorizationResult>;
}

pub async fn check_polkit_auth_zbus(
    conn: &zbus::Connection,
    sender: &str,
    action_id: &str,
    allow_user_interaction: bool,
) -> Result<bool, zbus::Error> {
    if let Ok(creds) = conn.peer_credentials().await {
        if creds.uid() == Some(0) {
            return Ok(true);
        }
    }

    let proxy = PolicyKitAuthorityProxy::new(conn).await?;
    let subject = PolkitSubject::system_bus_name(sender);
    let details = HashMap::<&str, &str>::new();
    let flags = if allow_user_interaction { 1u32 } else { 0u32 };

    let result = proxy
        .check_authorization(&subject, action_id, &details, flags, "")
        .await?;

    Ok(result.is_authorized)
}

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
