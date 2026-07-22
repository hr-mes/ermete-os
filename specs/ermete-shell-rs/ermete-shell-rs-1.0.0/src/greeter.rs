use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Entry, Label, Orientation, Align};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use greetd_ipc::{Request, Response};

const GREETER_CSS: &str = r#"
window.background {
    background-color: transparent;
}

.greeter-backdrop {
    background-color: rgba(10, 12, 18, 0.45);
}

.greeter-topbar-title {
    font-family: 'Inter', 'SF Pro Display', sans-serif;
    font-size: 14px;
    font-weight: 800;
    letter-spacing: 4px;
    color: rgba(255, 255, 255, 0.90);
    text-shadow: 0 2px 8px rgba(0,0,0,0.5);
}

.greeter-status-pill {
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 13px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.95);
    background-color: rgba(255, 255, 255, 0.15);
    padding: 8px 16px;
    border-radius: 999px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    transition: background-color 0.3s ease;
}

.greeter-status-pill:hover {
    background-color: rgba(255, 255, 255, 0.25);
}

.greeter-clock-time {
    font-family: 'Inter', 'SF Pro Display', sans-serif;
    font-size: 84px;
    font-weight: 300;
    color: #ffffff;
    letter-spacing: -3px;
    text-shadow: 0 8px 24px rgba(0,0,0,0.4);
    margin-bottom: -10px;
}

.greeter-clock-date {
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 20px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.90);
    margin-bottom: 24px;
    text-shadow: 0 4px 12px rgba(0,0,0,0.4);
}

.greeter-card {
    background-color: rgba(24, 27, 36, 0.55);
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 36px;
    padding: 42px 52px;
    min-width: 400px;
    box-shadow: 0 32px 84px rgba(0, 0, 0, 0.8), inset 0 1px 1px rgba(255, 255, 255, 0.15);
    transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.greeter-avatar-frame {
    border: 2px solid rgba(255, 255, 255, 0.4);
    border-radius: 999px;
    min-width: 96px;
    min-height: 96px;
    background-color: rgba(255, 255, 255, 0.15);
    font-size: 40px;
    color: #ffffff;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.6);
    transition: all 0.3s ease;
}

.greeter-user-name {
    font-family: 'Inter', 'SF Pro Display', sans-serif;
    font-size: 26px;
    font-weight: 700;
    color: #ffffff;
    margin-top: 18px;
    text-shadow: 0 4px 16px rgba(0,0,0,0.5);
}

.greeter-badge {
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 2px;
    color: rgba(255, 255, 255, 0.70);
    margin-top: 6px;
    margin-bottom: 22px;
}

.greeter-caps-pill, .greeter-biometric-pill {
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 12px;
    font-weight: 700;
    border-radius: 999px;
    padding: 6px 14px;
    margin-bottom: 12px;
    letter-spacing: 1px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    transition: all 0.3s ease;
}

.greeter-caps-pill {
    color: #ffd166;
    background-color: rgba(255, 209, 102, 0.20);
    border: 1px solid rgba(255, 209, 102, 0.40);
}

.greeter-biometric-pill {
    color: #38bdf8;
    background-color: rgba(56, 189, 248, 0.20);
    border: 1px solid rgba(56, 189, 248, 0.40);
}

.greeter-entry-box {
    background-color: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 20px;
    padding: 6px 10px;
    box-shadow: inset 0 2px 8px rgba(0,0,0,0.3);
    transition: all 0.3s ease;
}

.greeter-entry-box:focus-within {
    border-color: rgba(255, 255, 255, 0.6);
    background-color: rgba(0, 0, 0, 0.35);
    box-shadow: inset 0 2px 8px rgba(0,0,0,0.4), 0 0 12px rgba(255,255,255,0.2);
}

.greeter-entry {
    background: transparent;
    border: none;
    color: #ffffff;
    caret-color: #6ea8fe;
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 16px;
    min-height: 44px;
    box-shadow: none;
}

.greeter-icon-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.75);
    font-size: 18px;
    padding: 8px 12px;
    border-radius: 12px;
    transition: all 0.2s ease;
    box-shadow: none;
}

.greeter-icon-btn:hover {
    background-color: rgba(255, 255, 255, 0.15);
    color: #ffffff;
}

.greeter-error {
    color: #ff6b6b;
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 14px;
    font-weight: 600;
    margin-top: 12px;
    transition: opacity 0.3s ease;
}

.greeter-status-msg {
    color: #6ea8fe;
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 14px;
    font-weight: 600;
    margin-top: 12px;
    transition: opacity 0.3s ease;
}

.greeter-power-btn {
    background-color: rgba(255, 255, 255, 0.15);
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 999px;
    color: #ffffff;
    font-family: 'Inter', 'SF Pro Text', sans-serif;
    font-size: 14px;
    font-weight: 600;
    padding: 12px 24px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.3);
    transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.greeter-power-btn:hover {
    background-color: rgba(255, 255, 255, 0.25);
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0,0,0,0.4);
}
"#;

fn format_italian_date(now: &chrono::DateTime<chrono::Local>) -> String {
    use chrono::Datelike;
    let weekday = match now.weekday() {
        chrono::Weekday::Mon => "Lunedì",
        chrono::Weekday::Tue => "Martedì",
        chrono::Weekday::Wed => "Mercoledì",
        chrono::Weekday::Thu => "Giovedì",
        chrono::Weekday::Fri => "Venerdì",
        chrono::Weekday::Sat => "Sabato",
        chrono::Weekday::Sun => "Domenica",
    };
    let month = match now.month() {
        1 => "gennaio",
        2 => "febbraio",
        3 => "marzo",
        4 => "aprile",
        5 => "maggio",
        6 => "giugno",
        7 => "luglio",
        8 => "agosto",
        9 => "settembre",
        10 => "ottobre",
        11 => "novembre",
        12 => "dicembre",
        _ => "",
    };
    format!("{}, {} {}", weekday, now.day(), month)
}

fn send_request(stream: &mut UnixStream, req: &Request) -> Result<Response, String> {
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub real_name: String,
    pub avatar_path: Option<String>,
}

#[allow(dead_code)]
pub fn discover_target_user() -> UserInfo {
    let env_user = std::env::var("USER").unwrap_or_default();
    let target_user = if env_user == "greeter" || env_user.is_empty() {
        std::env::var("ERMETE_LOGIN_USER").unwrap_or_else(|_| {
            // Scan /etc/passwd for first normal user with UID >= 1000 and valid shell
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

    // Parse real name from GECOS in /etc/passwd
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

    // Discover avatar path
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

fn resolve_target_username() -> String {
    discover_target_user().username
}

pub fn unlock_keyring_automatic(password: &str, username: &str) {
    println!("[Ermete Greeter] Keyring unlock requested for user: {}", username);
    if let Ok(conn) = zbus::blocking::Connection::session() {
        if let Ok(proxy) = crate::core::system_proxies::SecretEnrollerProxyBlocking::new(&conn) {
            if password.is_empty() {
                // Biometric / FIDO2 login without password: try decrypting TPM 2.0 sealed token
                if let Ok(decrypted_secret) = proxy.decrypt_secret(username) {
                    let _ = proxy.unlock_keyring(username, &decrypted_secret);
                }
            } else {
                // Interactive login/enrollment
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
        // Dry-run / standalone mode for previewing greeter/lock UI outside greetd
        std::thread::sleep(std::time::Duration::from_millis(500));
        if password.is_empty() {
            return Err("Inserisci la password o usa l'impronta digitale".to_string());
        }
        return Ok(());
    }

    let mut stream = UnixStream::connect(path).map_err(|e| e.to_string())?;
    let username = resolve_target_username();

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

    // PAM / Biometric Multi-step interactive conversation loop (pam_fprintd / pam_u2f / pam_unix)
    let mut iterations = 0;
    while iterations < 15 {
        iterations += 1;
        match resp {
            Response::AuthMessage { auth_message_type, auth_message } => {
                let msg_lower = auth_message.to_lowercase();
                if msg_lower.contains("finger") || msg_lower.contains("impronta") || msg_lower.contains("touch") || matches!(auth_message_type, greetd_ipc::AuthMessageType::Info) {
                    status_cb(&auth_message);
                    // Send empty response to acknowledge Info / Biometric prompt and wait for next PAM message
                    let req = Request::PostAuthMessageResponse { response: Some("".to_string()) };
                    resp = send_request(&mut stream, &req)?;
                } else {
                    // Password or secret requested by PAM
                    status_cb("Verifica credenziali in corso...");
                    let req = Request::PostAuthMessageResponse { response: Some(password.to_string()) };
                    resp = send_request(&mut stream, &req)?;
                }
            }
            Response::Success => {
                if is_lockscreen {
                    unlock_keyring_automatic(password, &username);
                    // In lockscreen mode, session is already active; unlock directly
                    return Ok(());
                } else {
                    unlock_keyring_automatic(password, &username);
                    let req = Request::StartSession {
                        cmd: vec![session_cmd],
                        env: vec![],
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

#[allow(dead_code)]
pub fn authenticate_or_simulate(password: &str, is_lockscreen: bool) -> Result<(), String> {
    authenticate_interactive(password, is_lockscreen, &|_| {})
}

pub fn build_ui(app: &Application, is_lockscreen: bool) {
    let title = if is_lockscreen { "Ermete Lockscreen" } else { "Ermete Greeter" };
    let window = ApplicationWindow::builder()
        .application(app)
        .title(title)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.set_namespace(if is_lockscreen { "lockscreen" } else { "greeter" });

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    if let Some(display) = gtk4::gdk::Display::default() {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(GREETER_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let root_vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["greeter-backdrop"])
        .hexpand(true)
        .vexpand(true)
        .build();

    // Zone 1: Top Bar
    let topbar = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(20)
        .margin_start(28)
        .margin_end(28)
        .build();

    let os_title = Label::builder()
        .label("ERMETE OS")
        .css_classes(["greeter-topbar-title"])
        .build();

    let spacer = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    let theme_toggle = Button::builder()
        .label("🎨 Theme")
        .css_classes(["greeter-status-pill"])
        .build();

    let status_pill = Label::builder()
        .label("󰤨   󰁹   IT")
        .css_classes(["greeter-status-pill"])
        .build();

    let right_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    right_box.append(&theme_toggle);
    right_box.append(&status_pill);

    topbar.append(&os_title);
    topbar.append(&spacer);
    topbar.append(&right_box);

    // Zone 2: Center Clock + Card
    let center_box = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .spacing(24)
        .build();

    let clock_box = Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Center)
        .spacing(4)
        .build();

    let time_label = Label::builder()
        .css_classes(["greeter-clock-time"])
        .build();

    let date_label = Label::builder()
        .css_classes(["greeter-clock-date"])
        .build();

    let now = chrono::Local::now();
    time_label.set_text(&now.format("%H:%M").to_string());
    date_label.set_text(&format_italian_date(&now));

    let time_label_clone = time_label.clone();
    let date_label_clone = date_label.clone();
    glib::timeout_add_seconds_local(1, move || {
        let now = chrono::Local::now();
        time_label_clone.set_text(&now.format("%H:%M").to_string());
        date_label_clone.set_text(&format_italian_date(&now));
        glib::ControlFlow::Continue
    });

    clock_box.append(&time_label);
    clock_box.append(&date_label);

    let card_box = Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Center)
        .css_classes(["greeter-card"])
        .build();

    let user_info = discover_target_user();

    // Avatar rendering
    let avatar_widget: gtk4::Widget = if let Some(path) = &user_info.avatar_path {
        let picture = gtk4::Picture::for_filename(path);
        picture.set_can_shrink(true);
        picture.set_size_request(88, 88);
        picture.add_css_class("greeter-avatar-frame");
        picture.upcast()
    } else {
        let lbl = Label::builder()
            .label("")
            .css_classes(["greeter-avatar-frame"])
            .halign(Align::Center)
            .build();
        lbl.upcast()
    };
    avatar_widget.set_halign(Align::Center);

    let user_label = Label::builder()
        .label(&user_info.real_name)
        .halign(Align::Center)
        .css_classes(["greeter-user-name"])
        .build();

    let badge_text = if is_lockscreen { "BLOCCO SCHERMO • WAYLAND" } else { "WAYLAND • NIRI" };
    let badge_label = Label::builder()
        .label(badge_text)
        .halign(Align::Center)
        .css_classes(["greeter-badge"])
        .build();

    let biometric_pill = Label::builder()
        .label("󰈆 BIOMETRIA (TPM 2.0 / FPRINTD) & KEYRING UNLOCK ATTIVI")
        .halign(Align::Center)
        .css_classes(["greeter-biometric-pill"])
        .visible(std::path::Path::new("/var/run/dbus/system_bus_socket").exists())
        .build();

    let caps_label = Label::builder()
        .label("󰪛 MAIUSC ATTIVO")
        .halign(Align::Center)
        .css_classes(["greeter-caps-pill"])
        .visible(false)
        .build();

    // Password Entry Row
    let entry_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["greeter-entry-box"])
        .hexpand(true)
        .build();

    let password_entry = Entry::builder()
        .placeholder_text("Password di accesso...")
        .visibility(false)
        .hexpand(true)
        .css_classes(["greeter-entry"])
        .build();

    let reveal_btn = Button::builder()
        .label("󰈈")
        .css_classes(["greeter-icon-btn"])
        .build();

    let entry_reveal_clone = password_entry.clone();
    let reveal_btn_clone = reveal_btn.clone();
    reveal_btn.connect_clicked(move |_| {
        let vis = gtk4::prelude::EntryExt::is_visible(&entry_reveal_clone);
        entry_reveal_clone.set_visibility(!vis);
        reveal_btn_clone.set_label(if !vis { "󰈉" } else { "󰈈" });
    });

    let submit_btn = Button::builder()
        .label("➔")
        .css_classes(["greeter-icon-btn"])
        .build();

    entry_row.append(&password_entry);
    entry_row.append(&reveal_btn);
    entry_row.append(&submit_btn);

    // Caps Lock detection on key presses
    let key_ctrl = gtk4::EventControllerKey::new();
    let caps_clone = caps_label.clone();
    key_ctrl.connect_key_pressed(move |_, _keyval, _keycode, state| {
        let is_caps = state.contains(gtk4::gdk::ModifierType::LOCK_MASK);
        caps_clone.set_visible(is_caps);
        glib::Propagation::Proceed
    });
    password_entry.add_controller(key_ctrl);

    let error_label = Label::builder()
        .label("")
        .css_classes(["greeter-error"])
        .visible(false)
        .wrap(true)
        .build();

    let status_label = Label::builder()
        .label("")
        .css_classes(["greeter-status-msg"])
        .visible(false)
        .wrap(true)
        .build();

    let err_clear = error_label.clone();
    let status_clear = status_label.clone();
    password_entry.connect_changed(move |_| {
        err_clear.set_visible(false);
        status_clear.set_visible(false);
    });

    let app_ref = app.clone();
    let submit_login = std::rc::Rc::new({
        let entry = password_entry.clone();
        let err_label = error_label.clone();
        let status_label = status_label.clone();
        let submit_btn = submit_btn.clone();
        move || {
            let password = entry.text().to_string();
            entry.set_sensitive(false);
            submit_btn.set_sensitive(false);
            err_label.set_visible(false);
            status_label.set_text("Accesso in corso...");
            status_label.set_visible(true);

            let (sender, receiver) = glib::MainContext::channel::<Result<(), String>>(glib::Priority::DEFAULT);
            std::thread::spawn(move || {
                let res = authenticate_or_simulate(&password, is_lockscreen);
                let _ = sender.send(res);
            });

            let entry_clone = entry.clone();
            let err_clone = err_label.clone();
            let status_clone = status_label.clone();
            let submit_clone = submit_btn.clone();
            let app_quit = app_ref.clone();
            receiver.attach(None, move |res| {
                match res {
                    Ok(_) => {
                        std::process::exit(0);
                        app_quit.quit();
                    }
                    Err(e) => {
                        status_clone.set_visible(false);
                        err_clone.set_text(&format!("Accesso non riuscito: {}", e));
                        err_clone.set_visible(true);
                        entry_clone.set_text("");
                        entry_clone.set_sensitive(true);
                        submit_clone.set_sensitive(true);
                        entry_clone.grab_focus();
                    }
                }
                glib::ControlFlow::Break
            });
        }
    });

    let sl_clone = submit_login.clone();
    password_entry.connect_activate(move |_| sl_clone());
    submit_btn.connect_clicked(move |_| submit_login());

    card_box.append(&avatar_widget);
    card_box.append(&user_label);
    card_box.append(&badge_label);
    card_box.append(&biometric_pill);
    card_box.append(&caps_label);
    card_box.append(&entry_row);
    card_box.append(&error_label);
    card_box.append(&status_label);

    center_box.append(&clock_box);
    center_box.append(&card_box);

    // Zone 3: Bottom Power Buttons
    let bottom_bar = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .margin_bottom(32)
        .spacing(16)
        .build();

    let suspend_btn = Button::builder()
        .label("Sospendi")
        .css_classes(["greeter-power-btn"])
        .build();
    suspend_btn.connect_clicked(|_| {
        if let Err(e) = std::process::Command::new("systemctl").arg("suspend").spawn() {
            eprintln!("Failed to spawn systemctl suspend: {}", e);
        }
    });

    let reboot_btn = Button::builder()
        .label("Riavvia")
        .css_classes(["greeter-power-btn"])
        .build();
    reboot_btn.connect_clicked(|_| {
        if let Err(e) = std::process::Command::new("systemctl").arg("reboot").spawn() {
            eprintln!("Failed to spawn systemctl reboot: {}", e);
        }
    });

    let poweroff_btn = Button::builder()
        .label("Spegni")
        .css_classes(["greeter-power-btn"])
        .build();
    poweroff_btn.connect_clicked(|_| {
        if let Err(e) = std::process::Command::new("systemctl").arg("poweroff").spawn() {
            eprintln!("Failed to spawn systemctl poweroff: {}", e);
        }
    });

    bottom_bar.append(&suspend_btn);
    bottom_bar.append(&reboot_btn);
    bottom_bar.append(&poweroff_btn);

    root_vbox.append(&topbar);
    root_vbox.append(&center_box);
    root_vbox.append(&bottom_bar);

    window.set_child(Some(&root_vbox));
    window.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlock_keyring_automatic_empty_password() {
        unlock_keyring_automatic("", "testuser");
    }

    #[test]
    fn test_unlock_keyring_automatic_with_password() {
        unlock_keyring_automatic("secret123", "testuser");
    }
}

