use anyhow::{Context, Result};
use tracing::{info, warn, error};
use tokio::process::Command;
use serde_json::Value;

#[derive(Debug)]
pub enum UpdateStatus {
    NoUpdates,
    Layer1AppliedLive,
    Layer0RebootRequired,
    Error(String),
}

pub struct UpdateEngine;

impl UpdateEngine {
    pub async fn new() -> Result<Self> {
        info!("Initializing Ermete OTA Dual-Layer Update Engine...");
        Ok(Self {})
    }

    pub async fn check_and_apply(&mut self) -> Result<UpdateStatus> {
        info!("Fetching latest OCI OS Image via bootc...");
        
        // Use bootc to pull and stage the latest OCI container image
        let upgrade_out = Command::new("bootc")
            .arg("upgrade")
            .output()
            .await
            .context("Failed to execute bootc upgrade")?;

        let upgrade_text = String::from_utf8_lossy(&upgrade_out.stdout);
        let upgrade_err = String::from_utf8_lossy(&upgrade_out.stderr);
        
        // If bootc says no updates or nothing changed
        if upgrade_text.contains("No updates available") || upgrade_err.contains("No updates available") || upgrade_text.contains("No changes") {
            info!("System is already up to date.");
            return Ok(UpdateStatus::NoUpdates);
        }

        if !upgrade_out.status.success() {
            error!("bootc upgrade failed: {}", upgrade_err);
            return Ok(UpdateStatus::Error("Upgrade command failed".to_string()));
        }

        info!("New OS Layer staged successfully. Analyzing Tier impact...");

        // Determine if Layer 0 packages were touched
        // We can check rpm-ostree pending diff
        let status_out = Command::new("rpm-ostree")
            .arg("status")
            .arg("--json")
            .output()
            .await?;

        if !status_out.status.success() {
            warn!("Could not get rpm-ostree status, assuming Reboot Required just in case.");
            return Ok(UpdateStatus::Layer0RebootRequired);
        }

        let mut layer0_touched = false;
        
        if let Ok(json) = serde_json::from_slice::<Value>(&status_out.stdout) {
            if let Some(deployments) = json.get("deployments").and_then(|d| d.as_array()) {
                // Find pending and booted deployments to run a diff
                let booted = deployments.iter().find(|d| d.get("booted").and_then(|b| b.as_bool()) == Some(true));
                let pending = deployments.iter().find(|d| d.get("staged").and_then(|b| b.as_bool()) == Some(true) || d.get("pending").and_then(|b| b.as_bool()) == Some(true));
                
                if let (Some(b), Some(p)) = (booted, pending) {
                    if let (Some(b_csum), Some(p_csum)) = (b.get("checksum").and_then(|v| v.as_str()), p.get("checksum").and_then(|v| v.as_str())) {
                        info!("Comparing diff between {} and {}", b_csum, p_csum);
                        if let Ok(diff_out) = Command::new("rpm-ostree").arg("db").arg("diff").arg(b_csum).arg(p_csum).output().await {
                            let diff_text = String::from_utf8_lossy(&diff_out.stdout).to_lowercase();
                            
                            // Define Layer 0 packages that strictly require a reboot
                            let layer0_keywords = ["kernel", "nvidia", "dracut", "selinux", "systemd", "glibc", "dbus", "bootc", "kmod", "fwupd"];
                            
                            for kw in layer0_keywords.iter() {
                                if diff_text.contains(kw) {
                                    warn!("Layer 0 package '{}' was modified. A reboot is strictly required.", kw);
                                    layer0_touched = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            warn!("Failed to parse JSON, defaulting to Reboot Required for safety.");
            layer0_touched = true;
        }

        if layer0_touched {
            info!("Transaction contains Tier 0 modifications. Reboot pending.");
            return Ok(UpdateStatus::Layer0RebootRequired);
        } else {
            info!("Transaction contains ONLY Tier 1-3 modifications (UI/Userspace). Applying LIVE...");
            
            let apply_out = Command::new("rpm-ostree")
                .arg("apply-live")
                .output()
                .await?;
                
            if apply_out.status.success() {
                info!("Live update applied successfully without reboot.");
                return Ok(UpdateStatus::Layer1AppliedLive);
            } else {
                let err = String::from_utf8_lossy(&apply_out.stderr);
                warn!("Live update failed (maybe some services couldn't be restarted?): {}. Falling back to Reboot.", err);
                return Ok(UpdateStatus::Layer0RebootRequired);
            }
        }
    }
}
