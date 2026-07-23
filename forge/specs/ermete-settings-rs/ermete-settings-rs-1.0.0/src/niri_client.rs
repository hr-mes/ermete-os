use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::env;

fn send_socket_request(json_req: &str) -> Option<String> {
    let socket_path = env::var("NIRI_SOCKET").ok()?;
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream.write_all(json_req.as_bytes()).ok()?;
    if !json_req.ends_with('\n') {
        stream.write_all(b"\n").ok()?;
    }
    stream.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() {
        Some(line.trim().to_string())
    } else {
        None
    }
}

pub fn get_outputs() -> Vec<String> {
    let mut outputs = Vec::new();
    if let Some(resp) = send_socket_request("\"Outputs\"\n") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
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

pub fn set_output_scale(output_name: &str, scale: f64) {
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
        let _ = send_socket_request(&req_str);
    }
}

pub fn set_output_vrr(output_name: &str, enabled: bool) {
    let req = serde_json::json!({
        "Output": {
            "output": output_name,
            "action": {
                "SetVRR": enabled
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str);
    }
}

pub fn set_output_hdr(output_name: &str, enabled: bool) {
    let req = serde_json::json!({
        "Output": {
            "output": output_name,
            "action": {
                "SetHDR": enabled
            }
        }
    });
    if let Ok(req_str) = serde_json::to_string(&req) {
        let _ = send_socket_request(&req_str);
    }
}

pub fn set_output_mode(output_name: &str, mode: &str) {
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
        let _ = send_socket_request(&req_str);
    }
}

pub fn set_keyboard_layout_by_index(idx: usize) {
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
        let _ = send_socket_request(&req_str);
    }
}

/// Native Rust KDL setting updater without CLI subprocess calls (`sh -c sed`).
pub fn update_niri_kdl_setting(setting_key: &str, val: &str) {
    let mut path = match env::var("HOME") {
        Ok(h) => std::path::PathBuf::from(h),
        Err(_) => std::path::PathBuf::from("/tmp"),
    };
    path.push(".config/niri/config.kdl");

    if let Ok(content) = std::fs::read_to_string(&path) {
        let mut new_lines = Vec::new();
        let mut found = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(setting_key) && (trimmed.len() == setting_key.len() || trimmed.chars().nth(setting_key.len()) == Some(' ') || trimmed.chars().nth(setting_key.len()) == Some('\t')) {
                let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                new_lines.push(format!("{}{}{}{}", leading_spaces, setting_key, if val.is_empty() { "" } else { " " }, val));
                found = true;
            } else {
                new_lines.push(line.to_string());
            }
        }
        if !found {
            // Append at end if not found
            new_lines.push(format!("{} {}", setting_key, val));
        }
        let _ = std::fs::write(&path, new_lines.join("\n"));
    }
}
