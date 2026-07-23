use serde::de::DeserializeOwned;
use serde_json::Value;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

pub fn get_niri_socket_path() -> Option<String> {
    env::var("NIRI_SOCKET").ok()
}

/// Send a synchronous request over Niri UNIX socket without spawning subprocesses
pub fn niri_request(req_str: &str) -> Option<Value> {
    let socket_path = get_niri_socket_path()?;
    let mut stream = UnixStream::connect(socket_path).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let msg = format!("\"{}\"\n", req_str);
    stream.write_all(msg.as_bytes()).ok()?;
    stream.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str::<Value>(&line).ok()
}

/// Helper to fetch and deserialize data from Niri (e.g., Windows or Workspaces)
pub fn fetch_niri_data<T: DeserializeOwned>(req: &str, inner_key: &str) -> Option<T> {
    let resp = niri_request(req)?;
    let ok_val = resp.get("Ok")?;
    let data_val = ok_val.get(inner_key)?;
    serde_json::from_value::<T>(data_val.clone()).ok()
}

/// Connect directly to Niri EventStream socket, yielding lines to the callback
pub fn watch_niri_event_stream<F>(mut callback: F)
where
    F: FnMut(&str) + Send + 'static,
{
    std::thread::spawn(move || {
        loop {
            if let Some(socket_path) = get_niri_socket_path() {
                match UnixStream::connect(&socket_path) {
                    Ok(mut stream) => {
                        if stream.write_all(b"\"EventStream\"\n").is_ok() && stream.flush().is_ok() {
                            let mut reader = BufReader::new(stream);
                            let mut line = String::new();
                            // First line is handshake {"Ok":"Handled"}
                            if reader.read_line(&mut line).is_ok() {
                                std::thread::sleep(Duration::from_secs(2));
                                continue;
                            }
                            line.clear();

                            while reader.read_line(&mut line).is_ok() {
                                callback(&line);
                                line.clear();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to connect to NIRI_SOCKET ({:?}). Retrying in 2s...", e);
                    }
                }
            } else {
                eprintln!("Warning: NIRI_SOCKET environment variable not found. Retrying in 2s...");
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

/// Send an action request over Niri UNIX socket without spawning subprocesses
pub fn niri_action(action_value: Value) -> Option<Value> {
    let socket_path = get_niri_socket_path()?;
    let mut stream = UnixStream::connect(socket_path).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let msg = format!("{}\n", serde_json::to_string(&action_value).ok()?);
    stream.write_all(msg.as_bytes()).ok()?;
    stream.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str::<Value>(&line).ok()
}

pub fn focus_window(win_id: u64) {
    let _ = niri_action(serde_json::json!({
        "Action": {
            "FocusWindow": { "id": win_id }
        }
    }));
}

pub fn close_window() {
    let _ = niri_action(serde_json::json!({
        "Action": { "CloseWindow": {} }
    }));
}

pub fn close_window_by_id(win_id: u64) {
    let _ = niri_action(serde_json::json!({
        "Action": { "CloseWindow": { "id": win_id } }
    }));
}

pub fn focus_workspace_down() {
    let _ = niri_action(serde_json::json!({
        "Action": { "FocusWorkspaceDown": {} }
    }));
}

pub fn focus_workspace_up() {
    let _ = niri_action(serde_json::json!({
        "Action": { "FocusWorkspaceUp": {} }
    }));
}

pub fn quit_niri() {
    let _ = niri_action(serde_json::json!({
        "Action": { "Quit": {} }
    }));
}

pub fn focus_workspace_by_id(ws_id: u64) {
    let _ = niri_action(serde_json::json!({
        "Action": {
            "FocusWorkspace": { "reference": { "Id": ws_id } }
        }
    }));
}

pub fn screenshot() {
    let _ = niri_action(serde_json::json!({
        "Action": { "Screenshot": {} }
    }));
}

pub fn power_off_monitors() {
    let _ = niri_action(serde_json::json!({
        "Action": { "PowerOffMonitors": {} }
    }));
}
