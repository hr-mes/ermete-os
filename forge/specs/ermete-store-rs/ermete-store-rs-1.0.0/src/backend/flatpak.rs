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
