use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::fs;
use tracing::{info, warn};

pub struct GitHubReporter {
    client: Client,
}

impl GitHubReporter {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Ermete-Telemetry-Daemon")
                .build()
                .expect("Failed to build HTTP client — this should never fail"),
        }
    }

    /// Extract only safe metadata from crash data for the issue body.
    /// Never posts raw coredump data.
    pub fn extract_safe_metadata(raw: &str) -> String {
        let mut metadata = Vec::new();
        if let Some(line) = raw.lines().find(|l| l.contains("PID:")) {
            metadata.push(line.trim().to_string());
        }
        if let Some(line) = raw.lines().find(|l| l.contains("Signal:")) {
            metadata.push(line.trim().to_string());
        }
        if let Some(line) = raw.lines().find(|l| l.contains("Timestamp:")) {
            metadata.push(line.trim().to_string());
        }
        if let Some(line) = raw.lines().find(|l| l.contains("Command line:")) {
            let cmd = line
                .trim()
                .strip_prefix("Command line:")
                .unwrap_or("")
                .trim();
            let binary = cmd.split_whitespace().next().unwrap_or("unknown");
            metadata.push(format!("Executable: {}", binary));
        }
        if metadata.is_empty() {
            "No extractable metadata from crash data.".to_string()
        } else {
            metadata.join("\n")
        }
    }

    pub async fn report_crash(&self, crash_data: &str) -> Result<()> {
        info!("Preparing crash report to GitHub Issues...");

        let token = match std::env::var("GITHUB_TOKEN").or_else(|_| fs::read_to_string("/etc/ermete/telemetry.conf")) {
            Ok(t) => t.trim().to_string(),
            Err(_) => {
                warn!("No GitHub token configured. Crash report aborted.");
                return Err(anyhow::anyhow!("Opt-in GitHub token missing."));
            }
        };

        // NEVER post raw coredump data. Extract safe metadata only.
        let safe_metadata = Self::extract_safe_metadata(crash_data);

        let body = json!({
            "title": "Ermete OS Crash Report [metadata-only]",
            "body": format!(
                "### Crash Report (Metadata Only)\n\n\
                 ```\n{}\n```\n\n\
                 _This report contains only non-sensitive metadata. \
                 Raw crash data is never transmitted._\n\n\
                 _Reported automatically by Ermete OS Telemetry_",
                safe_metadata
            ),
            "labels": ["bug", "crash", "automated"]
        });

        // Use PRIVATE repository, not public
        let repo = std::env::var("ERMETE_TELEMETRY_REPO")
            .unwrap_or_else(|_| "patapem/ermete-forge-private".to_string());
        let url = format!("https://api.github.com/repos/{}/issues", repo);

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("token {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&body)
            .send()
            .await
            .context("Failed to connect to GitHub API")?;

        if res.status().is_success() {
            info!("Crash report submitted (metadata only).");
            Ok(())
        } else {
            let err_text = res.text().await?;
            Err(anyhow::anyhow!("GitHub API Error: {}", err_text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_safe_metadata() {
        let raw = "PID: 5678\nSignal: SIGSEGV\nTimestamp: 2024-01-15\nCommand line: /usr/bin/firefox --profile /home/user\nMemory map:\n7f000000 /usr/lib/libc.so\n";
        let metadata = GitHubReporter::extract_safe_metadata(raw);
        assert!(metadata.contains("PID: 5678"));
        assert!(metadata.contains("Executable: /usr/bin/firefox"));
        assert!(!metadata.contains("Memory map"));
        assert!(!metadata.contains("/home/user"));
    }
}
