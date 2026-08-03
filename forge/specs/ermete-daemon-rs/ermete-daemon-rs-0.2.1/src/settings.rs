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

#[derive(Clone)]
pub struct SettingsService {
    pub store: SettingsStateStore,
}

impl SettingsService {
    pub fn new() -> Self {
        Self {
            store: SettingsStateStore::new(),
        }
    }
}

#[interface(name = "org.ermete.Settings")]
impl SettingsService {
    #[zbus(property, name = "ColorScheme")]
    async fn color_scheme(&self) -> String {
        self.store.state.lock().await.color_scheme.clone()
    }

    #[zbus(property, name = "ColorScheme")]
    async fn set_color_scheme(&self, val: String) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        {
            let mut st = self.store.state.lock().await;
            st.color_scheme = val.clone();
            SettingsStateStore::save_async(&st).await.map_err(|e| fdo::Error::Failed(format!("Failed to save state: {}", e)))?;
        }
        let _ = tokio::process::Command::new("dconf")
            .args(["write", "/org/gnome/desktop/interface/color-scheme", &format!("'{}'", val)])
            .output()
            .await;
        Ok(())
    }

    #[zbus(property, name = "AccentColor")]
    async fn accent_color(&self) -> String {
        self.store.state.lock().await.accent_color.clone()
    }

    #[zbus(property, name = "AccentColor")]
    async fn set_accent_color(&self, val: String) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        {
            let mut st = self.store.state.lock().await;
            st.accent_color = val.clone();
            SettingsStateStore::save_async(&st).await.map_err(|e| fdo::Error::Failed(format!("Failed to save state: {}", e)))?;
        }
        let _ = tokio::process::Command::new("matugen")
            .args(["color", "hex", &val])
            .output()
            .await;
            
        let _ = tokio::process::Command::new("niri")
            .args(["msg", "action", "do-screen-transition"])
            .output()
            .await;
            
        Ok(())
    }

    #[zbus(property, name = "Wallpaper")]
    async fn wallpaper(&self) -> String {
        self.store.state.lock().await.wallpaper.clone()
    }

    #[zbus(property, name = "Wallpaper")]
    async fn set_wallpaper(&self, val: String) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        {
            let mut st = self.store.state.lock().await;
            st.wallpaper = val.clone();
            SettingsStateStore::save_async(&st).await.map_err(|e| fdo::Error::Failed(format!("Failed to save state: {}", e)))?;
        }
        let _ = tokio::process::Command::new("swww")
            .args(["img", &val, "--transition-type", "grow", "--transition-pos", "0.5,0.5"])
            .output()
            .await;
        Ok(())
    }

    #[zbus(property, name = "TrueToneEnabled")]
    async fn true_tone_enabled(&self) -> bool {
        self.store.state.lock().await.true_tone_enabled
    }

    #[zbus(property, name = "TrueToneEnabled")]
    async fn set_true_tone_enabled(&self, val: bool) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        let temp = {
            let mut st = self.store.state.lock().await;
            st.true_tone_enabled = val;
            SettingsStateStore::save_async(&st).await.map_err(|e| fdo::Error::Failed(format!("Failed to save state: {}", e)))?;
            st.true_tone_temperature
        };
        apply_true_tone(val, temp).await;
        Ok(())
    }

    #[zbus(property, name = "TrueToneTemperature")]
    async fn true_tone_temperature(&self) -> u32 {
        self.store.state.lock().await.true_tone_temperature
    }

    #[zbus(property, name = "TrueToneTemperature")]
    async fn set_true_tone_temperature(&self, val: u32) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        let enabled = {
            let mut st = self.store.state.lock().await;
            st.true_tone_temperature = val;
            SettingsStateStore::save_async(&st).await.map_err(|e| fdo::Error::Failed(format!("Failed to save state: {}", e)))?;
            st.true_tone_enabled
        };
        apply_true_tone(enabled, val).await;
        Ok(())
    }

    #[zbus(property, name = "VoiceOverEnabled")]
    async fn voiceover_enabled(&self) -> bool {
        self.store.state.lock().await.voiceover_enabled
    }

    #[zbus(property, name = "VoiceOverEnabled")]
    async fn set_voiceover_enabled(&self, val: bool) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        let mut st = self.store.state.lock().await;
        st.voiceover_enabled = val;
        SettingsStateStore::save_async(&st).await.map_err(|e| fdo::Error::Failed(format!("Failed to save state: {}", e)))?;
        
        if val {
            let _ = tokio::process::Command::new("spd-say")
                .arg("Voice Over attivato. Accessibilità sistema pronta.")
                .spawn();
        }
        Ok(())
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
