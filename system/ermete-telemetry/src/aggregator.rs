
use crate::collector::LogRecord;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogBatch {
    pub batch_id: String,
    pub created_at: String,
    pub records: Vec<LogRecord>,
    pub has_critical_severity: bool,
}

pub struct BatchAggregator {
    receiver: mpsc::Receiver<LogRecord>,
    output_sender: mpsc::Sender<LogBatch>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl BatchAggregator {
    pub fn new(
        receiver: mpsc::Receiver<LogRecord>,
        output_sender: mpsc::Sender<LogBatch>,
        max_batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        Self {
            receiver,
            output_sender,
            max_batch_size,
            flush_interval,
        }
    }

    pub async fn run_loop(mut self) {
        info!(
            "📦 Batch Aggregator running (max size: {}, flush window: {:?})",
            self.max_batch_size, self.flush_interval
        );

        let mut buffer = Vec::with_capacity(self.max_batch_size);
        let mut has_critical = false;
        let mut timer = tokio::time::interval(self.flush_interval);

        loop {
            tokio::select! {
                record_opt = self.receiver.recv() => {
                    match record_opt {
                        Some(record) => {
                            if matches!(record.priority.as_str(), "EMERG" | "ALERT" | "CRIT" | "ERR") {
                                has_critical = true;
                            }
                            buffer.push(record);

                            // Flush immediately if buffer full or if a critical error arrived
                            if buffer.len() >= self.max_batch_size || has_critical {
                                self.flush(&mut buffer, &mut has_critical).await;
                            }
                        }
                        None => {
                            info!("Aggregator input channel closed. Flushing remaining buffer.");
                            if !buffer.is_empty() {
                                self.flush(&mut buffer, &mut has_critical).await;
                            }
                            break;
                        }
                    }
                }
                _ = timer.tick() => {
                    if !buffer.is_empty() {
                        debug!("Flush timer triggered batch export ({} records)", buffer.len());
                        self.flush(&mut buffer, &mut has_critical).await;
                    }
                }
            }
        }
    }

    async fn flush(&self, buffer: &mut Vec<LogRecord>, has_critical: &mut bool) {
        let batch_id = format!("batch-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let batch = LogBatch {
            batch_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            records: std::mem::take(buffer),
            has_critical_severity: *has_critical,
        };

        *has_critical = false;

        if self.output_sender.send(batch).await.is_err() {
            tracing::warn!("Output channel closed. Failed to dispatch log batch.");
        }
    }
}
