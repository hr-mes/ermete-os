use serde::{Deserialize, Serialize};

/// Micro-domain state for theme and color palette settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeState {
    pub color_scheme: String,  // "prefer-dark" or "default" (light)
    pub accent_color: String,  // hex e.g. "#89b4fa"
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            color_scheme: "prefer-dark".to_string(),
            accent_color: "#89b4fa".to_string(),
        }
    }
}

/// Micro-domain state for desktop wallpaper settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WallpaperState {
    pub wallpaper: String,     // e.g. "/usr/share/backgrounds/ermete-default.png"
}

impl Default for WallpaperState {
    fn default() -> Self {
        Self {
            wallpaper: "/usr/share/backgrounds/ermete-default.png".to_string(),
        }
    }
}

/// Micro-domain state for desktop dock configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DockState {
    pub dock_pinned: Vec<String>,
}

impl Default for DockState {
    fn default() -> Self {
        Self {
            dock_pinned: vec![
                "org.gnome.Terminal.desktop".to_string(),
                "org.mozilla.firefox.desktop".to_string(),
                "os.ermete.Settings.desktop".to_string(),
            ],
        }
    }
}

/// Micro-domain state for display color temperature and TrueTone settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayState {
    pub true_tone_enabled: bool,
    pub true_tone_temperature: u32,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            true_tone_enabled: false,
            true_tone_temperature: 4500,
        }
    }
}

/// Decoupled aggregate domain state composing individual single-responsibility micro-domain states.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppearanceDomainState {
    #[serde(flatten)]
    pub theme: ThemeState,
    #[serde(flatten)]
    pub wallpaper: WallpaperState,
    #[serde(flatten)]
    pub dock: DockState,
    #[serde(flatten)]
    pub display: DisplayState,
}

#[allow(dead_code)]
impl AppearanceDomainState {
    pub fn new(theme: ThemeState, wallpaper: WallpaperState, dock: DockState, display: DisplayState) -> Self {
        Self {
            theme,
            wallpaper,
            dock,
            display,
        }
    }

    // Direct accessors for backward compatibility
    pub fn color_scheme(&self) -> &str {
        &self.theme.color_scheme
    }

    pub fn accent_color(&self) -> &str {
        &self.theme.accent_color
    }

    pub fn wallpaper_path(&self) -> &str {
        &self.wallpaper.wallpaper
    }

    pub fn dock_pinned(&self) -> &[String] {
        &self.dock.dock_pinned
    }

    pub fn true_tone_enabled(&self) -> bool {
        self.display.true_tone_enabled
    }

    pub fn true_tone_temperature(&self) -> u32 {
        self.display.true_tone_temperature
    }
}
