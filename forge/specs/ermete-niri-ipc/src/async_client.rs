//! Asynchronous Niri IPC & Configuration Client for GTK4/Relm4
//! 
//! Provides non-blocking Tokio-based IPC communication over Unix Sockets
//! and async filesystem mutations for Niri compositor settings.

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

/// Return the path to the Niri IPC socket from the environment.
pub fn get_niri_socket_path() -> Option<String> {
    env::var("NIRI_SOCKET").ok()
}

/// Low-level helper to send a raw JSON string request to the Niri socket asynchronously.
/// Enforces non-blocking I/O with timeouts to guarantee zero main-thread blocking.
pub async fn send_socket_request(json_req: &str) -> Option<String> {
    let socket_path = get_niri_socket_path()?;
    let stream = timeout(Duration::from_millis(1000), UnixStream::connect(socket_path))
        .await
        .ok()?
        .ok()?;

    let mut reader = BufReader::new(stream);

    let write_future = async {
        reader.write_all(json_req.as_bytes()).await?;
        if !json_req.ends_with('\n') {
            reader.write_all(b"\n").await?;
        }
        reader.flush().await?;
        Ok::<(), std::io::Error>(())
    };
    timeout(Duration::from_millis(1000), write_future).await.ok()?.ok()?;

    let mut line = String::new();
    let read_future = reader.read_line(&mut line);
    let bytes_read = timeout(Duration::from_millis(1000), read_future).await.ok()?.ok()?;

    if bytes_read > 0 {
        Some(line.trim().to_string())
    } else {
        None
    }
}

/// Send a request string (e.g. "Workspaces") and parse JSON response asynchronously.
pub async fn niri_request(req_str: &str) -> Option<Value> {
    let formatted_req = format!("\"{}\"\n", req_str);
    let resp = send_socket_request(&formatted_req).await?;
    serde_json::from_str::<Value>(&resp).ok()
}

/// Send an action JSON object to Niri socket asynchronously.
pub async fn niri_action(action_value: Value) -> Option<Value> {
    let req_str = serde_json::to_string(&action_value).ok()?;
    let resp = send_socket_request(&req_str).await?;
    serde_json::from_str::<Value>(&resp).ok()
}

/// Helper to fetch and deserialize typed data from Niri asynchronously.
pub async fn fetch_niri_data<T: DeserializeOwned>(req: &str, inner_key: &str) -> Option<T> {
    let resp = niri_request(req).await?;
    let ok_val = resp.get("Ok")?;
    let data_val = ok_val.get(inner_key)?;
    serde_json::from_value::<T>(data_val.clone()).ok()
}

/// Fetch list of connected output display names asynchronously.
pub async fn get_outputs() -> Vec<String> {
    let mut outputs = Vec::new();
    if let Some(resp) = send_socket_request("\"Outputs\"\n").await {
        if let Ok(json) = serde_json::from_str::<Value>(&resp) {
            if let Some(ok_obj) = json.get("Ok").and_then(|o| o.as_object()) {
                if let Some(outs_obj) = ok_obj.get("Outputs").and_then(|o| o.as_object()) {
                    for (name, _) in outs_obj {
                        outputs.push(name.clone());
                    }
                }
            }
        }
    }
    outputs.sort();
    outputs.dedup();
    if outputs.is_empty() {
        outputs.push("eDP-1".to_string());
    }
    outputs
}

/// Set scale factor for a specific output asynchronously.
pub async fn set_output_scale(output_name: &str, scale: f64) {
    let req = serde_json::json!({
        "Output": {
            "output": output_name,
            "action": {
                "Scale": {
                    "scale": {
                        "Specific": scale
                    }
                }
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str).await;
    }
}

/// Set VRR (Variable Refresh Rate) mode for an output asynchronously.
pub async fn set_output_vrr(output_name: &str, enabled: bool) {
    let req = serde_json::json!({
        "Output": {
            "output": output_name,
            "action": {
                "SetVRR": enabled
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str).await;
    }
}

/// Set HDR mode for an output asynchronously.
pub async fn set_output_hdr(output_name: &str, enabled: bool) {
    let req = serde_json::json!({
        "Output": {
            "output": output_name,
            "action": {
                "SetHDR": enabled
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str).await;
    }
}

/// Set resolution/mode for an output asynchronously.
pub async fn set_output_mode(output_name: &str, mode: &str) {
    let req = serde_json::json!({
        "Output": {
            "output": output_name,
            "action": {
                "SetMode": {
                    "mode": mode
                }
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str).await;
    }
}

/// Switch keyboard layout by index asynchronously.
pub async fn set_keyboard_layout_by_index(idx: usize) {
    let req = serde_json::json!({
        "Action": {
            "SwitchLayout": {
                "layout": {
                    "Index": idx
                }
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str).await;
    }
}

/// Native Rust KDL setting updater using non-blocking tokio::fs without subprocesses or thread locks.
pub async fn update_niri_kdl_setting(setting_key: &str, val: &str) {
    let mut path = match env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => PathBuf::from("/tmp"),
    };
    path.push(".config/niri/config.kdl");

    if let Ok(content) = fs::read_to_string(&path).await {
        let mut new_lines = Vec::new();
        let mut found = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(setting_key)
                && (trimmed.len() == setting_key.len()
                    || trimmed.chars().nth(setting_key.len()) == Some(' ')
                    || trimmed.chars().nth(setting_key.len()) == Some('\t'))
            {
                let leading_spaces: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                new_lines.push(format!(
                    "{}{}{}{}",
                    leading_spaces,
                    setting_key,
                    if val.is_empty() { "" } else { " " },
                    val
                ));
                found = true;
            } else {
                new_lines.push(line.to_string());
            }
        }
        if !found {
            new_lines.push(format!("{} {}", setting_key, val));
        }
        let content_str = new_lines.join("\n");
            if let Err(e) = async {
                let mut file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(&path)
                    .await?;
                tokio::io::AsyncWriteExt::write_all(&mut file, content_str.as_bytes()).await
            }.await {
                tracing::error!("Failed to update config at {:?}: {:?}", path, e);
            }
    }
}

/// Enable or disable DRM/KMS Direct Scanout compositor bypass asynchronously.
pub async fn set_direct_scanout(enabled: bool) {
    let val = if enabled { "enable-direct-scanout" } else { "// enable-direct-scanout" };
    update_niri_kdl_setting("enable-direct-scanout", val).await;
}

/// Enable or disable prefer-no-vsync for tearing control and zero-latency graphics asynchronously.
pub async fn set_prefer_no_vsync(enabled: bool) {
    let val = if enabled { "prefer-no-vsync" } else { "// prefer-no-vsync" };
    update_niri_kdl_setting("prefer-no-vsync", val).await;
}


/// Focus window by window ID asynchronously.
pub async fn focus_window(win_id: u64) {
    let _ = niri_action(serde_json::json!({
        "Action": {
            "FocusWindow": { "id": win_id }
        }
    })).await;
}

/// Close currently focused window asynchronously.
pub async fn close_window() {
    let _ = niri_action(serde_json::json!({
        "Action": { "CloseWindow": {} }
    })).await;
}

/// Close specific window by window ID asynchronously.
pub async fn close_window_by_id(win_id: u64) {
    let _ = niri_action(serde_json::json!({
        "Action": { "CloseWindow": { "id": win_id } }
    })).await;
}

/// Focus workspace down asynchronously.
pub async fn focus_workspace_down() {
    let _ = niri_action(serde_json::json!({
        "Action": { "FocusWorkspaceDown": {} }
    })).await;
}

/// Focus workspace up asynchronously.
pub async fn focus_workspace_up() {
    let _ = niri_action(serde_json::json!({
        "Action": { "FocusWorkspaceUp": {} }
    })).await;
}

/// Focus workspace by ID asynchronously.
pub async fn focus_workspace_by_id(ws_id: u64) {
    let _ = niri_action(serde_json::json!({
        "Action": {
            "FocusWorkspace": { "reference": { "Id": ws_id } }
        }
    })).await;
}

/// Quit Niri compositor asynchronously.
pub async fn quit_niri() {
    let _ = niri_action(serde_json::json!({
        "Action": { "Quit": {} }
    })).await;
}

/// Take screenshot asynchronously.
pub async fn screenshot() {
    let _ = niri_action(serde_json::json!({
        "Action": { "Screenshot": {} }
    })).await;
}

/// Power off monitors asynchronously.
pub async fn power_off_monitors() {
    let _ = niri_action(serde_json::json!({
        "Action": { "PowerOffMonitors": {} }
    })).await;
}

/// Connect directly to Niri EventStream socket asynchronously using Tokio background task.
pub fn watch_niri_event_stream<F>(mut callback: F) -> tokio::task::JoinHandle<()>
where
    F: FnMut(&str) + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            if let Some(socket_path) = get_niri_socket_path() {
                match UnixStream::connect(&socket_path).await {
                    Ok(mut stream) => {
                        let (read_half, mut write_half) = stream.split();
                        if write_half.write_all(b"\"EventStream\"\n").await.is_ok()
                            && write_half.flush().await.is_ok()
                        {
                            let mut reader = BufReader::new(read_half);
                            let mut line = String::new();
                            // First line is handshake {"Ok":"Handled"}
                            if reader.read_line(&mut line).await.is_ok() {
                                line.clear();
                                loop {
                                    match reader.read_line(&mut line).await {
                                        Ok(0) => break,
                                        Ok(_) => {
                                            callback(&line);
                                            line.clear();
                                        }
                                        Err(_) => break,
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to connect to NIRI_SOCKET ({:?}). Retrying in 2s...",
                            e
                        );
                    }
                }
            } else {
                eprintln!("Warning: NIRI_SOCKET environment variable not found. Retrying in 2s...");
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    })
}
