use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

const PRIMARY_SYSTEMD_DIR: &str = "/etc/systemd/system";
const FALLBACK_SYSTEMD_DIR: &str = "/tmp/systemd/system";

fn is_dir_writable(path: &Path) -> bool {
    if !path.exists() {
        if fs::create_dir_all(path).is_err() {
            return false;
        }
    }
    let probe_file = path.join(".ermete_init_oracle_probe");
    if fs::write(&probe_file, b"probe").is_ok() {
        let _ = fs::remove_file(probe_file);
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedServiceRecord {
    pub service_name: String,
    pub unit_name: String,
    pub unit_path: PathBuf,
    pub primary_exec: String,
    pub fallback_exec: Option<String>,
    pub is_fallback_active: bool,
    pub status: String,
    pub created_at_secs: u64,
}

#[derive(Clone)]
pub struct SystemdManager {
    target_dir: PathBuf,
    records: Arc<Mutex<HashMap<String, ManagedServiceRecord>>>,
}

impl SystemdManager {
    pub fn new() -> Self {
        let primary_path = Path::new(PRIMARY_SYSTEMD_DIR);
        let target_dir = if is_dir_writable(primary_path) {
            primary_path.to_path_buf()
        } else {
            let fb = PathBuf::from(FALLBACK_SYSTEMD_DIR);
            let _ = fs::create_dir_all(&fb);
            warn!(
                "Primary systemd directory {} not writable, using fallback location {:?}",
                PRIMARY_SYSTEMD_DIR, fb
            );
            fb
        };

        info!("SystemdManager initialized with target unit directory: {:?}", target_dir);

        Self {
            target_dir,
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_target_dir(&self) -> &Path {
        &self.target_dir
    }



    pub async fn reload_daemon(&self) -> Result<()> {
        info!("Executing systemctl daemon-reload...");
        let is_user_mode = self.target_dir != Path::new(PRIMARY_SYSTEMD_DIR);
        let mut cmd = tokio::process::Command::new("systemctl");
        cmd.arg("--no-ask-password");
        if is_user_mode {
            cmd.arg("--user");
        }
        cmd.arg("daemon-reload");

        match cmd.output().await {
            Ok(out) if out.status.success() => {
                info!("systemctl daemon-reload succeeded.");
                Ok(())
            }
            Ok(out) => {
                let err_msg = String::from_utf8_lossy(&out.stderr);
                anyhow::bail!("systemctl daemon-reload returned non-zero exit code: {}", err_msg);
            }
            Err(e) => {
                anyhow::bail!("systemctl command not available or failed: {}", e);
            }
        }
    }

    pub async fn start_service(&self, unit_name: &str) -> Result<()> {
        info!("Starting systemd unit '{}'...", unit_name);
        let is_user_mode = self.target_dir != Path::new(PRIMARY_SYSTEMD_DIR);
        let mut cmd = tokio::process::Command::new("systemctl");
        cmd.arg("--no-ask-password");
        if is_user_mode {
            cmd.arg("--user");
        }
        cmd.args(["start", unit_name]);

        match cmd.output().await {
            Ok(out) if out.status.success() => {
                info!("Unit '{}' started successfully.", unit_name);
                Ok(())
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                anyhow::bail!("systemctl start error for unit '{}': {}", unit_name, err);
            }
            Err(e) => {
                anyhow::bail!("systemctl start command failed for unit '{}': {}", unit_name, e);
            }
        }
    }

    pub async fn stop_service(&self, unit_name: &str) -> Result<()> {
        info!("Stopping systemd unit '{}'...", unit_name);
        let is_user_mode = self.target_dir != Path::new(PRIMARY_SYSTEMD_DIR);
        let mut cmd = tokio::process::Command::new("systemctl");
        cmd.arg("--no-ask-password");
        if is_user_mode {
            cmd.arg("--user");
        }
        cmd.args(["stop", unit_name]);

        match cmd.output().await {
            Ok(out) if out.status.success() => {
                info!("Unit '{}' stopped successfully.", unit_name);
                Ok(())
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                anyhow::bail!("systemctl stop error for unit '{}': {}", unit_name, err);
            }
            Err(e) => {
                anyhow::bail!("systemctl stop command failed for unit '{}': {}", unit_name, e);
            }
        }
    }

    pub async fn check_service_status(&self, unit_name: &str) -> Result<String> {
        let is_user_mode = self.target_dir != Path::new(PRIMARY_SYSTEMD_DIR);
        let mut args = vec!["--no-ask-password"];
        if is_user_mode {
            args.push("--user");
        }
        args.extend_from_slice(&["is-active", unit_name]);
        let output = tokio::process::Command::new("systemctl").args(&args).output().await?;

        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !status_str.is_empty() {
            Ok(status_str)
        } else {
            let err_str = String::from_utf8_lossy(&output.stderr).trim().to_string();
            anyhow::bail!("systemctl is-active returned empty output for unit '{}'. Stderr: {}", unit_name, err_str);
        }
    }

    pub async fn list_services(&self) -> Vec<ManagedServiceRecord> {
        let lock = self.records.lock().await;
        lock.values().cloned().collect()
    }

    pub async fn revert_service(&self, service_name: &str) -> Result<String> {
        let mut lock = self.records.lock().await;
        if let Some(record) = lock.remove(service_name) {
            self.stop_service(&record.unit_name).await?;
            if record.unit_path.exists() {
                tokio::fs::remove_file(&record.unit_path).await?;
            }
            self.reload_daemon().await?;
            Ok(format!("Service '{}' reverted and unit file {:?} removed.", service_name, record.unit_path))
        } else {
            anyhow::bail!("Service '{}' is not currently managed by Init Oracle", service_name)
        }
    }

    pub async fn run_health_audit_cycle(&self) -> Result<()> {
        let services = self.list_services().await;
        for record in services {
            let current_status = self.check_service_status(&record.unit_name).await?;
            if current_status == "failed" || current_status == "inactive" {
                warn!(
                    "Audit detected service '{}' in state '{}'. Triggering autonomous recovery...",
                    record.service_name, current_status
                );
                self.start_service(&record.unit_name).await?;
            }
        }
        Ok(())
    }

}




