use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub timestamp: String,
    pub unit: String,
    pub priority: String,
    pub message: String,
    pub pid: Option<u32>,
    pub sys_facility: Option<String>,
}

pub struct JournalCollector {
    sender: mpsc::Sender<LogRecord>,
}

impl JournalCollector {
    pub fn new(sender: mpsc::Sender<LogRecord>) -> Self {
        Self { sender }
    }

    /// Spawns the async journald / syslog stream reader task
    pub async fn run_loop(self) -> Result<()> {
        info!("🌀 Initializing async journald stream subscriber...");

        let mut child = Command::new("journalctl")
            .args(["-f", "-o", "json", "-n", "20"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        let stdout = match child.as_mut().ok().and_then(|c| c.stdout.take()) {
            Some(out) => out,
            None => {
                warn!("⚠️ Could not attach to systemd journalctl pipe. Falling back to synthetic syslog stream...");
                return self.run_fallback_syslog_stream().await;
            }
        };

        let mut reader = BufReader::new(stdout).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(record) = Self::parse_journal_entry(&line) {
                if self.sender.send(record).await.is_err() {
                    warn!("Log record receiver dropped. Shutting down collector.");
                    break;
                }
            }
        }

        info!("Journal collector stream closed.");
        Ok(())
    }

    fn parse_journal_entry(json_str: &str) -> Result<LogRecord> {
        let v: serde_json::Value = serde_json::from_str(json_str)?;

        let message = v["MESSAGE"]
            .as_str()
            .unwrap_or_else(|| v["message"].as_str().unwrap_or(""))
            .to_string();

        let unit = v["_SYSTEMD_UNIT"]
            .as_str()
            .unwrap_or_else(|| v["SYSLOG_IDENTIFIER"].as_str().unwrap_or("kernel"))
            .to_string();

        let priority_num = v["PRIORITY"]
            .as_str()
            .and_then(|p| p.parse::<u8>().ok())
            .unwrap_or(6);

        let priority = match priority_num {
            0 => "EMERG",
            1 => "ALERT",
            2 => "CRIT",
            3 => "ERR",
            4 => "WARNING",
            5 => "NOTICE",
            6 => "INFO",
            _ => "DEBUG",
        }
        .to_string();

        let timestamp = v["__REALTIME_TIMESTAMP"]
            .as_str()
            .unwrap_or("0")
            .to_string();

        let pid = v["_PID"]
            .as_str()
            .and_then(|p| p.parse::<u32>().ok());

        Ok(LogRecord {
            timestamp,
            unit,
            priority,
            message,
            pid,
            sys_facility: v["SYSLOG_FACILITY"].as_str().map(|s| s.to_string()),
        })
    }

    /// Fallback handler when systemd journalctl is not available
    async fn run_fallback_syslog_stream(self) -> Result<()> {
        anyhow::bail!("Systemd journalctl log stream unavailable")
    }
}
