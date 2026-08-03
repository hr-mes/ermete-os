use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
struct NiriWorkspaceState {
    active_workspace: String,
    windows: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum SyncMessage {
    Clipboard(String),
    NiriState(NiriWorkspaceState),
}

/// Mock for WireGuard Connection
struct WireguardP2pConnection {
    peer_ip: String,
    is_connected: bool,
}

impl WireguardP2pConnection {
    fn new(peer_ip: &str) -> Self {
        Self {
            peer_ip: peer_ip.to_string(),
            is_connected: false,
        }
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Establishing Zero-Trust WireGuard P2P connection to {}", self.peer_ip);
        // Mock connection delay
        sleep(Duration::from_millis(500)).await;
        self.is_connected = true;
        info!("WireGuard P2P connection established. E2E Encryption active.");
        Ok(())
    }

    async fn send_message(&self, message: &SyncMessage) -> Result<()> {
        if !self.is_connected {
            warn!("Cannot send message, peer disconnected.");
            return Ok(());
        }
        let payload = serde_json::to_string(message)?;
        info!("(Encrypted) Sending payload to {}: {}", self.peer_ip, payload);
        Ok(())
    }
}

async fn get_clipboard_content() -> Result<String> {
    let output = Command::new("wl-paste")
        .stdout(Stdio::piped())
        .output()?;
    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(content)
}

async fn get_niri_state() -> Result<NiriWorkspaceState> {
    // Mocking Niri state retrieval
    Ok(NiriWorkspaceState {
        active_workspace: "Workspace 1".to_string(),
        windows: vec!["terminal".to_string(), "browser".to_string()],
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Ermete Mesh Sync Zero-Trust Daemon...");

    let mut wg = WireguardP2pConnection::new("10.0.0.2");
    wg.connect().await?;

    let mut last_clipboard = String::new();

    loop {
        // Sync clipboard
        if let Ok(clipboard) = get_clipboard_content().await {
            if clipboard != last_clipboard && !clipboard.is_empty() {
                info!("Clipboard change detected, syncing...");
                wg.send_message(&SyncMessage::Clipboard(clipboard.clone())).await?;
                last_clipboard = clipboard;
            }
        }

        // Sync Niri State
        if let Ok(state) = get_niri_state().await {
            wg.send_message(&SyncMessage::NiriState(state)).await?;
        }

        sleep(Duration::from_secs(5)).await;
    }
}
