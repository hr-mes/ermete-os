use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::fdo;
use zbus::interface;

async fn check_polkit_auth() -> bool {
    // Fictional check
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub color_scheme: String,  // "prefer-dark" or "default" (light)
    pub accent_color: String,  // hex e.g. "#89b4fa"
    pub wallpaper: String,     // e.g. "/usr/share/backgrounds/ermete-default.png"
    pub dock_pinned: Vec<String>,
    pub true_tone_enabled: bool,
    pub true_tone_temperature: u32,
    pub voiceover_enabled: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            color_scheme: "prefer-dark".to_string(),
            accent_color: "#89b4fa".to_string(),
            wallpaper: "/usr/share/backgrounds/ermete-default.png".to_string(),
            dock_pinned: vec![
                "org.gnome.Terminal.desktop".to_string(),
                "org.mozilla.firefox.desktop".to_string(),
                "os.ermete.Settings.desktop".to_string(),
            ],
            true_tone_enabled: false,
            true_tone_temperature: 4500,
            voiceover_enabled: false,
        }
    }
}

#[derive(Clone)]
pub struct SettingsStateStore {
    pub state: Arc<Mutex<SettingsState>>,
}

impl SettingsStateStore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Self::load())),
        }
    }

    pub fn config_path() -> PathBuf {
        let mut path = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else if let Ok(home) = std::env::var("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".config");
            p
        } else {
            PathBuf::from("/var/lib/ermete")
        };
        path.push("ermete");
        let _ = std::fs::create_dir_all(&path);
        path.push("settings.json");
        path
    }

    pub fn load() -> SettingsState {
        let path = Self::config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
        SettingsState::default()
    }

    pub fn save(state: &SettingsState) -> std::io::Result<()> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(state)?;
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &content)?;
        std::fs::rename(&temp_path, &path)?;
        Ok(())
    }

    pub async fn save_async(state: &SettingsState) -> std::io::Result<()> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(state)?;
        let temp_path = path.with_extension("json.tmp");
        tokio::fs::write(&temp_path, &content).await?;
        tokio::fs::rename(&temp_path, &path).await?;
        Ok(())
    }
}

use tokio::sync::{mpsc, oneshot};

pub enum SettingsCommand {
    GetColorScheme(oneshot::Sender<String>),
    SetColorScheme(String, oneshot::Sender<fdo::Result<()>>),
    GetAccentColor(oneshot::Sender<String>),
    SetAccentColor(String, oneshot::Sender<fdo::Result<()>>),
    GetWallpaper(oneshot::Sender<String>),
    SetWallpaper(String, oneshot::Sender<fdo::Result<()>>),
    GetTrueToneEnabled(oneshot::Sender<bool>),
    SetTrueToneEnabled(bool, oneshot::Sender<fdo::Result<()>>),
    GetTrueToneTemperature(oneshot::Sender<u32>),
    SetTrueToneTemperature(u32, oneshot::Sender<fdo::Result<()>>),
    GetVoiceoverEnabled(oneshot::Sender<bool>),
    SetVoiceoverEnabled(bool, oneshot::Sender<fdo::Result<()>>),
}

#[derive(Clone)]
pub struct SettingsService {
    tx: mpsc::Sender<SettingsCommand>,
}

impl SettingsService {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<SettingsCommand>(32);
        
        tokio::spawn(async move {
            let mut state = SettingsStateStore::load();
            
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    SettingsCommand::GetColorScheme(reply) => {
                        let _ = reply.send(state.color_scheme.clone());
                    }
                    SettingsCommand::SetColorScheme(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        state.color_scheme = val.clone();
                        let res = SettingsStateStore::save_async(&state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = tokio::process::Command::new("dconf")
                                .args(["write", "/org/gnome/desktop/interface/color-scheme", &format!("'{}'", val)])
                                .output()
                                .await;
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetAccentColor(reply) => {
                        let _ = reply.send(state.accent_color.clone());
                    }
                    SettingsCommand::SetAccentColor(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        state.accent_color = val.clone();
                        let res = SettingsStateStore::save_async(&state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = tokio::process::Command::new("matugen")
                                .args(["color", "hex", &val])
                                .output()
                                .await;
                            let _ = tokio::process::Command::new("niri")
                                .args(["msg", "action", "do-screen-transition"])
                                .output()
                                .await;
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetWallpaper(reply) => {
                        let _ = reply.send(state.wallpaper.clone());
                    }
                    SettingsCommand::SetWallpaper(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        state.wallpaper = val.clone();
                        let res = SettingsStateStore::save_async(&state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = tokio::process::Command::new("swww")
                                .args(["img", &val, "--transition-type", "grow", "--transition-pos", "0.5,0.5"])
                                .output()
                                .await;
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetTrueToneEnabled(reply) => {
                        let _ = reply.send(state.true_tone_enabled);
                    }
                    SettingsCommand::SetTrueToneEnabled(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        state.true_tone_enabled = val;
                        let res = SettingsStateStore::save_async(&state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            apply_true_tone(state.true_tone_enabled, state.true_tone_temperature).await;
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetTrueToneTemperature(reply) => {
                        let _ = reply.send(state.true_tone_temperature);
                    }
                    SettingsCommand::SetTrueToneTemperature(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        state.true_tone_temperature = val;
                        let res = SettingsStateStore::save_async(&state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            apply_true_tone(state.true_tone_enabled, state.true_tone_temperature).await;
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetVoiceoverEnabled(reply) => {
                        let _ = reply.send(state.voiceover_enabled);
                    }
                    SettingsCommand::SetVoiceoverEnabled(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        state.voiceover_enabled = val;
                        let res = SettingsStateStore::save_async(&state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() && val {
                            let _ = tokio::process::Command::new("spd-say")
                                .arg("Voice Over attivato. Accessibilità sistema pronta.")
                                .spawn();
                        }
                        let _ = reply.send(res);
                    }
                }
            }
        });

        Self { tx }
    }
}

#[interface(name = "org.ermete.Settings")]
impl SettingsService {
    #[zbus(property, name = "ColorScheme")]
    async fn color_scheme(&self) -> String {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::GetColorScheme(reply)).await;
        rx.await.unwrap_or_default()
    }

    #[zbus(property, name = "ColorScheme")]
    async fn set_color_scheme(&self, val: String) -> fdo::Result<()> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetColorScheme(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }

    #[zbus(property, name = "AccentColor")]
    async fn accent_color(&self) -> String {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::GetAccentColor(reply)).await;
        rx.await.unwrap_or_default()
    }

    #[zbus(property, name = "AccentColor")]
    async fn set_accent_color(&self, val: String) -> fdo::Result<()> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetAccentColor(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }

    #[zbus(property, name = "Wallpaper")]
    async fn wallpaper(&self) -> String {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::GetWallpaper(reply)).await;
        rx.await.unwrap_or_default()
    }

    #[zbus(property, name = "Wallpaper")]
    async fn set_wallpaper(&self, val: String) -> fdo::Result<()> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetWallpaper(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }

    #[zbus(property, name = "TrueToneEnabled")]
    async fn true_tone_enabled(&self) -> bool {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::GetTrueToneEnabled(reply)).await;
        rx.await.unwrap_or_default()
    }

    #[zbus(property, name = "TrueToneEnabled")]
    async fn set_true_tone_enabled(&self, val: bool) -> fdo::Result<()> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetTrueToneEnabled(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }

    #[zbus(property, name = "TrueToneTemperature")]
    async fn true_tone_temperature(&self) -> u32 {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::GetTrueToneTemperature(reply)).await;
        rx.await.unwrap_or_default()
    }

    #[zbus(property, name = "TrueToneTemperature")]
    async fn set_true_tone_temperature(&self, val: u32) -> fdo::Result<()> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetTrueToneTemperature(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }

    #[zbus(property, name = "VoiceOverEnabled")]
    async fn voiceover_enabled(&self) -> bool {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::GetVoiceoverEnabled(reply)).await;
        rx.await.unwrap_or_default()
    }

    #[zbus(property, name = "VoiceOverEnabled")]
    async fn set_voiceover_enabled(&self, val: bool) -> fdo::Result<()> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetVoiceoverEnabled(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }
}

async fn apply_true_tone(enabled: bool, temp: u32) {
    // Kill existing wlsunset instances
    let _ = tokio::process::Command::new("killall")
        .arg("wlsunset")
        .output()
        .await;

    if enabled {
        // Spawn wlsunset with target temperature
        let _ = tokio::process::Command::new("wlsunset")
            .arg("-T")
            .arg(temp.to_string())
            .arg("-t")
            .arg(temp.to_string()) // Force fixed temp
            .spawn();
    }
}
