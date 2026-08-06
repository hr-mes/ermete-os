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
#[allow(dead_code)]
pub async fn install_app(app_id: &str, progress_tx: Sender<f64>) -> Result<(), String> {
    let mut child = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg("flathub")
        .arg(app_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn flatpak install command: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture flatpak stdout pipe".to_string())?;

    let mut reader = BufReader::new(stdout).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        if let Some(percentage) = parse_percentage(&line) {
            let _ = progress_tx.send(percentage).await;
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed waiting for flatpak process to finish: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Flatpak installation of '{}' failed with status {}",
            app_id, status
        ));
    }

    Ok(())
}

/// Helper function to parse percentage values (e.g., "45%", " 50.5%") from a stdout line.
#[allow(dead_code)]
fn parse_percentage(line: &str) -> Option<f64> {
    if let Some(percent_idx) = line.find('%') {
        let prefix = &line[..percent_idx];
        let number_str: String = prefix
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        let number_str = number_str.replace(',', ".");
        if let Ok(pct) = number_str.parse::<f64>() {
            return Some(pct);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("Installing 1/1 ... 45%"), Some(45.0));
        assert_eq!(parse_percentage("  50.5%"), Some(50.5));
        assert_eq!(parse_percentage("Downloading: 100%"), Some(100.0));
        assert_eq!(parse_percentage("No percentage here"), None);
        assert_eq!(parse_percentage("%"), None);
    }
}
