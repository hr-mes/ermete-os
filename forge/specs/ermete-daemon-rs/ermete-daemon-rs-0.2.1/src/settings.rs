use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::watch;
use zbus::fdo;
use zbus::interface;

#[zbus::proxy(
    interface = "os.ermete.SettingsWorker",
    default_service = "os.ermete.SettingsWorker",
    default_path = "/os/ermete/SettingsWorker"
)]
trait SettingsWorker {
    fn apply_color_scheme(&self, scheme: &str) -> zbus::Result<()>;
    fn apply_accent_color(&self, color: &str) -> zbus::Result<()>;
    fn apply_wallpaper(&self, wallpaper: &str) -> zbus::Result<()>;
    fn apply_true_tone(&self, enabled: bool, temp: u32) -> zbus::Result<()>;
    fn apply_voiceover(&self, enabled: bool) -> zbus::Result<()>;
}

async fn check_polkit_auth() -> bool {
    // Legacy internal stub retained for internal actor compatibility
    true
}

/// Domain micro-state for desktop appearance settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceDomainState {
    pub color_scheme: String,  // "prefer-dark" or "default" (light)
    pub accent_color: String,  // hex e.g. "#89b4fa"
    pub wallpaper: String,     // e.g. "/usr/share/backgrounds/ermete-default.png"
    pub dock_pinned: Vec<String>,
    pub true_tone_enabled: bool,
    pub true_tone_temperature: u32,
}

impl Default for AppearanceDomainState {
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
        }
    }
}

/// Domain micro-state for accessibility / voiceover settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceOverDomainState {
    pub enabled: bool,
}

impl Default for VoiceOverDomainState {
    fn default() -> Self {
        Self {
            enabled: false,
        }
    }
}

pub fn config_dir() -> PathBuf {
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
    path
}

#[derive(Clone)]
pub struct AppearanceStateStore {
    pub state_tx: watch::Sender<AppearanceDomainState>,
    pub state_rx: watch::Receiver<AppearanceDomainState>,
}

#[allow(dead_code)]
impl AppearanceStateStore {
    pub async fn new_async() -> Self {
        let initial_state = Self::load_async().await;
        let (state_tx, state_rx) = watch::channel(initial_state);
        Self { state_tx, state_rx }
    }

    pub fn new() -> Self {
        let initial_state = Self::load();
        let (state_tx, state_rx) = watch::channel(initial_state);
        Self { state_tx, state_rx }
    }

    pub async fn ensure_config_file() -> std::io::Result<PathBuf> {
        let dir = config_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let mut file_path = dir;
        file_path.push("appearance.json");
        Ok(file_path)
    }

    pub async fn load_async() -> AppearanceDomainState {
        if let Ok(path) = Self::ensure_config_file().await {
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        AppearanceDomainState::default()
    }

    pub fn load() -> AppearanceDomainState {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        let mut path = dir;
        path.push("appearance.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
        AppearanceDomainState::default()
    }

    pub fn save(state: &AppearanceDomainState) -> std::io::Result<()> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir)?;
        let mut path = dir;
        path.push("appearance.json");
        let content = serde_json::to_string_pretty(state)?;
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &content)?;
        std::fs::rename(&temp_path, &path)?;
        Ok(())
    }

    pub async fn save_async(state: &AppearanceDomainState) -> std::io::Result<()> {
        let path = Self::ensure_config_file().await?;
        let content = serde_json::to_string_pretty(state)?;
        let temp_path = path.with_extension("json.tmp");
        tokio::fs::write(&temp_path, &content).await?;
        tokio::fs::rename(&temp_path, &path).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct VoiceOverStateStore {
    pub state_tx: watch::Sender<VoiceOverDomainState>,
    pub state_rx: watch::Receiver<VoiceOverDomainState>,
}

#[allow(dead_code)]
impl VoiceOverStateStore {
    pub async fn new_async() -> Self {
        let initial_state = Self::load_async().await;
        let (state_tx, state_rx) = watch::channel(initial_state);
        Self { state_tx, state_rx }
    }

    pub fn new() -> Self {
        let initial_state = Self::load();
        let (state_tx, state_rx) = watch::channel(initial_state);
        Self { state_tx, state_rx }
    }

    pub async fn ensure_config_file() -> std::io::Result<PathBuf> {
        let dir = config_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let mut file_path = dir;
        file_path.push("voiceover.json");
        Ok(file_path)
    }

    pub async fn load_async() -> VoiceOverDomainState {
        if let Ok(path) = Self::ensure_config_file().await {
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        VoiceOverDomainState::default()
    }

    pub fn load() -> VoiceOverDomainState {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        let mut path = dir;
        path.push("voiceover.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
        VoiceOverDomainState::default()
    }

    pub fn save(state: &VoiceOverDomainState) -> std::io::Result<()> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir)?;
        let mut path = dir;
        path.push("voiceover.json");
        let content = serde_json::to_string_pretty(state)?;
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &content)?;
        std::fs::rename(&temp_path, &path)?;
        Ok(())
    }

    pub async fn save_async(state: &VoiceOverDomainState) -> std::io::Result<()> {
        let path = Self::ensure_config_file().await?;
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

use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct SettingsService {
    tx: mpsc::Sender<SettingsCommand>,
}

impl SettingsService {
    #[allow(dead_code)]
    pub fn new(
        appearance_tx: watch::Sender<AppearanceDomainState>,
        voiceover_tx: watch::Sender<VoiceOverDomainState>,
    ) -> Self {
        Self::new_with_token(appearance_tx, voiceover_tx, CancellationToken::new())
    }

    pub fn new_with_token(
        appearance_tx: watch::Sender<AppearanceDomainState>,
        voiceover_tx: watch::Sender<VoiceOverDomainState>,
        cancel_token: CancellationToken,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<SettingsCommand>(32);
        
        tokio::spawn(async move {
            let mut appearance_state = appearance_tx.borrow().clone();
            let mut voiceover_state = voiceover_tx.borrow().clone();
            let conn = zbus::Connection::session().await.ok();
            let worker = if let Some(ref c) = conn {
                SettingsWorkerProxy::new(c).await.ok()
            } else {
                None
            };
            
            loop {
                let cmd = tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("Shutdown token received. Exiting Settings actor loop.");
                        break;
                    }
                    opt = rx.recv() => {
                        match opt {
                            Some(c) => c,
                            None => break,
                        }
                    }
                };

                match cmd {
                    SettingsCommand::GetColorScheme(reply) => {
                        let _ = reply.send(appearance_state.color_scheme.clone());
                    }
                    SettingsCommand::SetColorScheme(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        appearance_state.color_scheme = val.clone();
                        let res = AppearanceStateStore::save_async(&appearance_state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = appearance_tx.send(appearance_state.clone());
                            if let Some(ref w) = worker {
                                let _ = w.apply_color_scheme(&val).await;
                            }
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetAccentColor(reply) => {
                        let _ = reply.send(appearance_state.accent_color.clone());
                    }
                    SettingsCommand::SetAccentColor(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        appearance_state.accent_color = val.clone();
                        let res = AppearanceStateStore::save_async(&appearance_state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = appearance_tx.send(appearance_state.clone());
                            if let Some(ref w) = worker {
                                let _ = w.apply_accent_color(&val).await;
                            }
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetWallpaper(reply) => {
                        let _ = reply.send(appearance_state.wallpaper.clone());
                    }
                    SettingsCommand::SetWallpaper(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        appearance_state.wallpaper = val.clone();
                        let res = AppearanceStateStore::save_async(&appearance_state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = appearance_tx.send(appearance_state.clone());
                            if let Some(ref w) = worker {
                                let _ = w.apply_wallpaper(&val).await;
                            }
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetTrueToneEnabled(reply) => {
                        let _ = reply.send(appearance_state.true_tone_enabled);
                    }
                    SettingsCommand::SetTrueToneEnabled(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        appearance_state.true_tone_enabled = val;
                        let res = AppearanceStateStore::save_async(&appearance_state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = appearance_tx.send(appearance_state.clone());
                            if let Some(ref w) = worker {
                                let _ = w.apply_true_tone(appearance_state.true_tone_enabled, appearance_state.true_tone_temperature).await;
                            }
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetTrueToneTemperature(reply) => {
                        let _ = reply.send(appearance_state.true_tone_temperature);
                    }
                    SettingsCommand::SetTrueToneTemperature(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        appearance_state.true_tone_temperature = val;
                        let res = AppearanceStateStore::save_async(&appearance_state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = appearance_tx.send(appearance_state.clone());
                            if let Some(ref w) = worker {
                                let _ = w.apply_true_tone(appearance_state.true_tone_enabled, appearance_state.true_tone_temperature).await;
                            }
                        }
                        let _ = reply.send(res);
                    }
                    SettingsCommand::GetVoiceoverEnabled(reply) => {
                        let _ = reply.send(voiceover_state.enabled);
                    }
                    SettingsCommand::SetVoiceoverEnabled(val, reply) => {
                        if !check_polkit_auth().await {
                            let _ = reply.send(Err(fdo::Error::Failed("Polkit authorization failed".into())));
                            continue;
                        }
                        voiceover_state.enabled = val;
                        let res = VoiceOverStateStore::save_async(&voiceover_state).await.map_err(|e| fdo::Error::Failed(e.to_string()));
                        if res.is_ok() {
                            let _ = voiceover_tx.send(voiceover_state.clone());
                            if let Some(ref w) = worker {
                                let _ = w.apply_voiceover(val).await;
                            }
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
    async fn set_color_scheme(
        &self,
        val: String,
        #[zbus(header)] hdr: Option<zbus::message::Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.settings.change").await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
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
    async fn set_accent_color(
        &self,
        val: String,
        #[zbus(header)] hdr: Option<zbus::message::Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.settings.change").await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
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
    async fn set_wallpaper(
        &self,
        val: String,
        #[zbus(header)] hdr: Option<zbus::message::Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.settings.change").await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
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
    async fn set_true_tone_enabled(
        &self,
        val: bool,
        #[zbus(header)] hdr: Option<zbus::message::Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.settings.change").await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
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
    async fn set_true_tone_temperature(
        &self,
        val: u32,
        #[zbus(header)] hdr: Option<zbus::message::Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.settings.change").await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
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
    async fn set_voiceover_enabled(
        &self,
        val: bool,
        #[zbus(header)] hdr: Option<zbus::message::Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.settings.change").await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(SettingsCommand::SetVoiceoverEnabled(val, reply)).await;
        rx.await.unwrap_or(Err(fdo::Error::Failed("Actor dead".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings_service_cancellation() {
        let app_store = AppearanceStateStore::new_async().await;
        let vo_store = VoiceOverStateStore::new_async().await;
        let token = CancellationToken::new();
        let _service = SettingsService::new_with_token(app_store.state_tx, vo_store.state_tx, token.clone());
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
}
