use greetd_ipc::{Request, Response};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

pub fn send_request(stream: &mut UnixStream, req: &Request) -> Result<Response, String> {
    let json = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let len = (json.len() as u32).to_ne_bytes();
    stream.write_all(&len).map_err(|e| e.to_string())?;
    stream.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let reply_len = u32::from_ne_bytes(len_buf);

    let mut reply_buf = vec![0u8; reply_len as usize];
    stream.read_exact(&mut reply_buf).map_err(|e| e.to_string())?;

    serde_json::from_slice(&reply_buf).map_err(|e| e.to_string())
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub real_name: String,
    pub avatar_path: Option<String>,
}

pub fn discover_target_user() -> UserInfo {
    let env_user = std::env::var("USER").unwrap_or_default();
    let target_user = if env_user == "greeter" || env_user.is_empty() {
        std::env::var("ERMETE_LOGIN_USER").unwrap_or_else(|_| {
            if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 7 {
                        if let Ok(uid) = parts[2].parse::<u32>() {
                            if uid >= 1000 && uid < 65534 && (parts[6].ends_with("bash") || parts[6].ends_with("zsh") || parts[6].ends_with("fish")) {
                                return parts[0].to_string();
                            }
                        }
                    }
                }
            }
            "ermete".to_string()
        })
    } else {
        env_user
    };

    let mut real_name = capitalize_first(&target_user);
    let mut home_dir = format!("/home/{}", target_user);
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 6 && parts[0] == target_user {
                let gecos = parts[4].split(',').next().unwrap_or(parts[0]);
                if !gecos.trim().is_empty() {
                    real_name = gecos.trim().to_string();
                }
                home_dir = parts[5].to_string();
                break;
            }
        }
    }

    let face_path = format!("{}/.face", home_dir);
    let acc_path = format!("/var/lib/AccountsService/icons/{}", target_user);
    let avatar_path = if std::path::Path::new(&face_path).exists() {
        Some(face_path)
    } else if std::path::Path::new(&acc_path).exists() {
        Some(acc_path)
    } else {
        None
    };

    UserInfo {
        username: target_user,
        real_name,
        avatar_path,
    }
}

pub fn unlock_keyring_automatic(password: &str, username: &str) {
    println!("[Ermete Greeter] Keyring unlock requested for user: {}", username);
    if let Ok(conn) = zbus::blocking::Connection::session() {
        if let Ok(proxy) = crate::core::system_proxies::SecretEnrollerProxyBlocking::new(&conn) {
            if password.is_empty() {
                if let Ok(decrypted_secret) = proxy.decrypt_secret(username) {
                    let _ = proxy.unlock_keyring(username, &decrypted_secret);
                }
            } else {
                let _ = proxy.enroll_secret(username, password);
                let _ = proxy.unlock_keyring(username, password);
            }
        }
    }
}

pub fn authenticate_interactive<F>(password: &str, is_lockscreen: bool, status_cb: &F) -> Result<(), String>
where
    F: Fn(&str),
{
    let path = std::env::var("GREETD_SOCK").unwrap_or_else(|_| "/run/greetd.sock".to_string());
    if !std::path::Path::new(&path).exists() {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if password.is_empty() {
            return Err("Inserisci la password o usa l'impronta digitale".to_string());
        }
        return Ok(());
    }

    let mut stream = UnixStream::connect(path).map_err(|e| e.to_string())?;
    let username = discover_target_user().username;

    let session_cmd = if std::path::Path::new("/usr/bin/ermete-session").exists() {
        "/usr/bin/ermete-session".to_string()
    } else if std::path::Path::new("/etc/greetd/ermete-session").exists() {
        "/etc/greetd/ermete-session".to_string()
    } else if std::path::Path::new("/usr/local/bin/ermete-session").exists() {
        "/usr/local/bin/ermete-session".to_string()
    } else {
        "ermete-session".to_string()
    };

    let req = Request::CreateSession { username: username.clone() };
    let mut resp = send_request(&mut stream, &req)?;

    let mut iterations = 0;
    while iterations < 15 {
        iterations += 1;
        match resp {
            Response::AuthMessage { auth_message_type, auth_message } => {
                let msg_lower = auth_message.to_lowercase();
                if msg_lower.contains("finger") || msg_lower.contains("impronta") || msg_lower.contains("touch") || matches!(auth_message_type, greetd_ipc::AuthMessageType::Info) {
                    status_cb(&auth_message);
                    let req = Request::PostAuthMessageResponse { response: Some("".to_string()) };
                    resp = send_request(&mut stream, &req)?;
                } else {
                    status_cb("Verifica credenziali in corso...");
                    let req = Request::PostAuthMessageResponse { response: Some(password.to_string()) };
                    resp = send_request(&mut stream, &req)?;
                }
            }
            Response::Success => {
                if is_lockscreen {
                    unlock_keyring_automatic(password, &username);
                    return Ok(());
                } else {
                    unlock_keyring_automatic(password, &username);
                    let req = Request::StartSession {
                        cmd: vec![session_cmd],
                        env: vec![
                            "XDG_SESSION_TYPE=wayland".to_string(),
                            "XDG_CURRENT_DESKTOP=niri".to_string(),
                        ],
                    };
                    let start_resp = send_request(&mut stream, &req)?;
                    match start_resp {
                        Response::Success => return Ok(()),
                        Response::Error { description, .. } => return Err(description),
                        _ => return Err("Risposta inattesa dal comando StartSession".to_string()),
                    }
                }
            }
            Response::Error { description, .. } => return Err(description),
        }
    }
    Err("Timeout conversazione PAM (troppi passaggi di autenticazione)".to_string())
}

pub fn authenticate_or_simulate(password: &str, is_lockscreen: bool) -> Result<(), String> {
    authenticate_interactive(password, is_lockscreen, &|_| {})
}
