use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

/// Installs a Flatpak application from Flathub non-blockingly, reporting progress percentages
/// through the provided `progress_tx` channel.
///
/// # Command
/// Executes `flatpak install -y flathub {app_id}` with stdout piped to capture percentage progress.
///
/// # Errors
/// Returns `Err(String)` if spawning the process fails, capturing stdout fails, waiting for completion fails,
/// or if flatpak returns a non-zero exit code.
pub async fn install_app(app_id: &str, progress_tx: Sender<u32>) -> Result<(), String> {
    if app_id.starts_with('-') {
        return Err("Invalid app_id".to_string());
    }
    let mut child = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg("flathub")
        .arg(app_id)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Some(pct_str) = line.split('%').next().and_then(|s| s.split_whitespace().last()) {
                if let Ok(pct) = pct_str.parse::<u32>() {
                    let _ = progress_tx.send(pct).await;
                }
            }
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("flatpak exited with status: {}", status))
    }
}
