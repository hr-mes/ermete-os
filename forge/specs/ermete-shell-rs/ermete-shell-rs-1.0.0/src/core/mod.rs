pub mod dock_config;
pub mod dock_data;
pub mod dock_watcher;
pub mod network;
pub mod mpris;
pub mod niri_state;
pub mod live_state;
pub mod niri_client;
pub mod spring;
pub mod system_proxies;
pub mod battery;
use gtk4::CssProvider;
use chrono::Local;
use serde::Deserialize;
use zbus::interface;

#[derive(Deserialize, Debug, Clone)]
pub struct NiriWorkspace {
    pub id: u64,
    pub idx: u64,
    pub name: Option<String>,
    pub output: String,
    pub is_active: bool,
    pub is_focused: bool,
}

pub fn spawn_niri_workspace_watcher(sender: glib::Sender<Vec<NiriWorkspace>>) {
    if let Some(workspaces) = niri_client::fetch_niri_data::<Vec<NiriWorkspace>>("Workspaces", "Workspaces") {
        let _ = sender.send(workspaces);
    }

    niri_client::watch_niri_event_stream(move |line| {
        if line.contains("Workspace") {
            if let Some(workspaces) = niri_client::fetch_niri_data::<Vec<NiriWorkspace>>("Workspaces", "Workspaces") {
                let _ = sender.send(workspaces);
            }
        }
    });
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NotificationData {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    #[serde(default = "default_timestamp")]
    pub timestamp: String,
    #[serde(default)]
    pub actions: Vec<(String, String)>,
    #[serde(default)]
    pub has_inline_reply: bool,
}

fn default_timestamp() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

pub fn get_notifications_file_path() -> std::path::PathBuf {
    let mut path = dirs_next_or_home();
    path.push(".local/share/ermete");
    let _ = std::fs::create_dir_all(&path);
    path.push("notifications.json");
    path
}

fn dirs_next_or_home() -> std::path::PathBuf {
    std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
}

pub fn save_notification_history() {
    NOTIFICATIONS.with(|n| {
        let list = n.borrow();
        if let Ok(json) = serde_json::to_string_pretty(&*list) {
            let _ = std::fs::write(get_notifications_file_path(), json);
        }
    });
}

pub fn load_notification_history() {
    let path = get_notifications_file_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(list) = serde_json::from_str::<Vec<NotificationData>>(&content) {
            NOTIFICATIONS.with(|n| {
                *n.borrow_mut() = list;
            });
        }
    }
}

pub static DND_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

thread_local! {
    pub static NOTIFICATIONS: std::cell::RefCell<Vec<NotificationData>> = std::cell::RefCell::new(Vec::new());
    pub static CSS_PROVIDER: std::cell::RefCell<Option<CssProvider>> = std::cell::RefCell::new(None);
}

pub struct NotificationServer {
    pub sender: glib::Sender<NotificationData>,
    pub counter: std::sync::atomic::AtomicU32,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        _app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<&str>,
        _hints: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
        _expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id == 0 {
            self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        } else {
            replaces_id
        };

        let mut parsed_actions = Vec::new();
        let mut has_inline = false;
        let mut i = 0;
        while i + 1 < _actions.len() {
            let key = _actions[i].to_string();
            let label = _actions[i + 1].to_string();
            if key == "inline-reply" || key.contains("reply") {
                has_inline = true;
            }
            parsed_actions.push((key, label));
            i += 2;
        }
        let app_lower = app_name.to_lowercase();
        if app_lower.contains("telegram") || app_lower.contains("slack") || app_lower.contains("whatsapp") || app_lower.contains("discord") || app_lower.contains("matrix") || app_lower.contains("element") || app_lower.contains("mail") {
            has_inline = true;
        }

        let notif = NotificationData {
            id,
            app_name: app_name.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            timestamp: default_timestamp(),
            actions: parsed_actions,
            has_inline_reply: has_inline,
        };

        let _ = self.sender.send(notif);
        id
    }

    async fn get_capabilities(&self) -> Vec<&str> {
        vec!["body", "actions", "inline-reply", "persistence"]
    }

    async fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("Ermete Notifications", "Ermete OS", "1.0", "1.2")
    }

    async fn close_notification(&self, _id: u32) {}
}

// Orologio macOS: "sab 11 lug 14:47"
pub fn macos_clock_string() -> String {
    Local::now().format("%a %d %b %H:%M").to_string()
}

pub fn get_ram_info() -> (String, f64) {
    let mut total_kb = 0.0;
    let mut avail_kb = 0.0;
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    total_kb = val.parse::<f64>().unwrap_or(0.0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    avail_kb = val.parse::<f64>().unwrap_or(0.0);
                }
            }
        }
    }
    if total_kb > 0.0 {
        let used_kb = total_kb - avail_kb;
        let frac = (used_kb / total_kb).clamp(0.0, 1.0);
        let used_gb = used_kb / 1048576.0;
        let total_gb = total_kb / 1048576.0;
        (
            format!("{:.1} GB / {:.1} GB ({:.0}%)", used_gb, total_gb, frac * 100.0),
            frac,
        )
    } else {
        ("N/D".to_string(), 0.0)
    }
}


pub fn get_cpu_load() -> (String, f64) {
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load_str) = content.split_whitespace().next() {
            let load = load_str.parse::<f64>().unwrap_or(0.0);
            let frac = (load / 4.0).clamp(0.0, 1.0);
            return (format!("Carico 1m: {}", load_str), frac);
        }
    }
    ("N/D".to_string(), 0.0)
}


pub fn get_network_status() -> (String, String, String) {
    crate::core::system_proxies::get_global_controller().get_cached_network_status()
}

pub fn speak_text(text: String) {
    glib::MainContext::default().spawn_local(async move {
        if let Ok(connection) = zbus::Connection::session().await {
            let _ = connection.call_method(
                Some("os.ermete.VoiceOver"),
                "/os/ermete/VoiceOver",
                Some("os.ermete.VoiceOver"),
                "Speak",
                &(text,)
            ).await;
        }
    });
}

pub fn attach_voiceover_hover<W: gtk4::prelude::IsA<gtk4::Widget>>(widget: &W, text: &str) {
    let ctrl = gtk4::EventControllerMotion::new();
    let text_clone = text.to_string();
    ctrl.connect_enter(move |_, _, _| {
        speak_text(text_clone.clone());
    });
    gtk4::prelude::WidgetExt::add_controller(widget, ctrl);
}
