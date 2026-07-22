use crate::github::GitHubReporter;
use std::process::Command;
use tracing::{info, warn};
use zbus::interface;

pub struct TelemetryIface;

#[interface(name = "os.ermete.Telemetry")]
impl TelemetryIface {
    /// Submits a crash report to the Forge. Requires Polkit auth if we want to ensure only the console user can do it.
    async fn submit_crash(
        &self,
        crash_id: String,
    ) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to submit crash: {}", crash_id);

        let reporter = GitHubReporter::new();

        // Fetch the crash dump using `coredumpctl info <crash_id>`.
        // NOTE: The raw output may contain sensitive data (env vars, memory maps, etc.).
        // GitHubReporter::report_crash() internally calls extract_safe_metadata() to
        // strip everything except PID, Signal, Timestamp, and binary name before posting.
        let output = Command::new("coredumpctl")
            .arg("info")
            .arg(&crash_id)
            .output();

        let dump_data = match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
            Ok(out) => {
                warn!(
                    "coredumpctl failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
                format!(
                    "Failed to retrieve full dump for {}. (Exit code: {})",
                    crash_id, out.status
                )
            }
            Err(e) => {
                warn!("Failed to execute coredumpctl: {}", e);
                format!("Failed to execute coredumpctl for {}", crash_id)
            }
        };

        if let Err(e) = reporter.report_crash(&dump_data).await {
            return Ok(format!("Failed to submit: {}", e));
        }

        Ok("Crash report successfully submitted.".into())
    }
}
