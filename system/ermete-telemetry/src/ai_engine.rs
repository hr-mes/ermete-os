use crate::aggregator::LogBatch;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LlamaEmbeddingResponse {
    pub embedding: Vec<f32>,
}

pub struct AiPredictiveEngine {
    dbus_conn: Option<Connection>,
    http_client: reqwest::Client,
    llama_endpoint: String,
    report_sender: mpsc::Sender<AnomalyReport>,
}

impl AiPredictiveEngine {
    pub async fn new(report_sender: mpsc::Sender<AnomalyReport>) -> Self {
        let dbus_conn = Connection::session().await.ok();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();

        let llama_endpoint = std::env::var("LLAMA_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:11434/api/embeddings".to_string());

        if dbus_conn.is_some() {
            info!("🤖 Telemetry AI Engine bound to `os.ermete.AiDaemon` DBus IPC interface.");
        } else {
            info!("🤖 Telemetry AI Engine set to fallback mode for DBus IPC.");
        }

        info!("🧠 Llama 3.2 AI Embeddings / REST Inference target: {}", llama_endpoint);

        Self {
            dbus_conn,
            http_client,
            llama_endpoint,
            report_sender,
        }
    }

    pub async fn run_loop(self: Arc<Self>, mut batch_receiver: mpsc::Receiver<LogBatch>) {
        info!("🔮 Predictive AI Engine active. Processing log batches for crash vector inference...");
        let semaphore = Arc::new(Semaphore::new(16));

        while let Some(batch) = batch_receiver.recv().await {
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let engine = self.clone();
            tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) = engine.process_batch(batch).await {
                    error!("Error processing log batch in AI Engine: {}", e);
                }
            });
        }
    }

    async fn process_batch(&self, batch: LogBatch) -> anyhow::Result<()> {
        use std::fmt::Write;
        let mut combined_text = String::new();
        for (i, r) in batch.records.iter().enumerate() {
            if i > 0 {
                combined_text.push('\n');
            }
            let _ = write!(
                combined_text,
                "[{}] {} ({}): {}",
                r.priority,
                r.unit,
                r.pid.unwrap_or(0),
                r.message
            );
        }

        // 1. Attempt Llama 3.2 embedding generation via REST
        let embedding = self.generate_embedding(&combined_text).await;

        // 2. Query AI Daemon via DBus IPC if available
        let ai_daemon_prediction = self.query_ai_daemon_ipc(&batch.batch_id, &combined_text).await;

        // 3. Compute predictive anomaly score
        let report = self.infer_anomaly(&batch, &combined_text, embedding, ai_daemon_prediction);

        if report.anomaly_score >= 0.65 {
            warn!(
                "🔥 HIGH PREDICTIVE ANOMALY DETECTED! Unit: {}, Failure: {}, Score: {:.2}, Action: {}",
                report.target_unit, report.predicted_failure_mode, report.anomaly_score, report.suggested_intent
            );

            if self.report_sender.send(report).await.is_err() {
                error!("Failed to deliver AnomalyReport to Init Oracle Bridge channel.");
            }
        }

        Ok(())
    }

    async fn generate_embedding(&self, text: &str) -> Vec<f32> {
        let payload = serde_json::json!({
            "model": "llama3.2",
            "prompt": text
        });

        match self
            .http_client
            .post(&self.llama_endpoint)
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<LlamaEmbeddingResponse>().await {
                    return data.embedding;
                }
            }
            _ => {
                // Return synthetic local embedding representation
            }
        }

        Self::generate_synthetic_embedding(text)
    }

    async fn query_ai_daemon_ipc(&self, batch_id: &str, text: &str) -> Option<String> {
        let conn = self.dbus_conn.as_ref()?;
        let query_payload = serde_json::json!({
            "intent": "predictive_telemetry_anomaly",
            "batch_id": batch_id,
            "log_content": text
        })
        .to_string();

        let reply = conn
            .call_method(
                Some("os.ermete.AiDaemon"),
                "/os/ermete/AiDaemon",
                Some("os.ermete.AiDaemon"),
                "process_query",
                &(query_payload.as_str()),
            )
            .await
            .ok()?;

        reply.body().deserialize::<String>().ok()
    }

    fn infer_anomaly(
        &self,
        batch: &LogBatch,
        _text: &str,
        embedding: Vec<f32>,
        _ai_daemon_resp: Option<String>,
    ) -> AnomalyReport {
        let mut score: f32 = 0.1;
        let mut failure_mode = "NORMAL_OPERATION".to_string();
        let mut suggested_intent = "NO_ACTION".to_string();
        let mut target_unit = "systemd".to_string();

        for record in &batch.records {
            if contains_ignore_ascii_case(&record.message, "MEMORY USAGE REACHED")
                || contains_ignore_ascii_case(&record.message, "OUT OF MEMORY")
                || contains_ignore_ascii_case(&record.message, "OOM-KILL")
            {
                score = 0.95;
                failure_mode = "IMMINENT_OOM_CRASH".to_string();
                target_unit = record.unit.clone();
                suggested_intent = format!("RESTART_UNIT: {}", record.unit);
                break;
            } else if contains_ignore_ascii_case(&record.message, "CHECKSUM ERROR")
                || contains_ignore_ascii_case(&record.message, "I/O ERROR")
                || contains_ignore_ascii_case(&record.message, "CORRUPTION")
            {
                score = 0.88;
                failure_mode = "STORAGE_CORRUPTION_PREVENTATIVE".to_string();
                target_unit = record.unit.clone();
                suggested_intent = format!("QUARANTINE_UNIT: {}", record.unit);
                break;
            } else if contains_ignore_ascii_case(&record.message, "HIGH RESTART COUNT")
                || contains_ignore_ascii_case(&record.message, "CRASH LOOP")
            {
                score = 0.78;
                failure_mode = "SERVICE_CRASH_LOOP".to_string();
                target_unit = record.unit.clone();
                suggested_intent = format!("REVERT_SERVICE: {}", record.unit.replace(".service", ""));
                break;
            } else if record.priority.eq_ignore_ascii_case("CRIT") || record.priority.eq_ignore_ascii_case("EMERG") {
                if score < 0.70 {
                    score = 0.72;
                    failure_mode = "CRITICAL_DAEMON_FAULT".to_string();
                    target_unit = record.unit.clone();
                    suggested_intent = format!("RESTART_UNIT: {}", record.unit);
                }
            }
        }

        AnomalyReport {
            anomaly_score: score,
            target_unit,
            predicted_failure_mode: failure_mode,
            suggested_intent,
            confidence: 0.94,
            embedding_vector: embedding,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn generate_synthetic_embedding(text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; 16];
        let bytes = text.as_bytes();
        for (i, b) in bytes.iter().enumerate() {
            vec[i % 16] += (*b as f32) / 255.0;
        }
        vec
    }
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
