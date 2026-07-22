use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DockConfig {
    pub pinned: Vec<String>,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            pinned: vec![
                "foot.desktop".to_string(),
                "firefox.desktop".to_string(),
                "nautilus.desktop".to_string(),
                "os.ermete.Settings.desktop".to_string(),
            ],
        }
    }
}

impl DockConfig {
    pub fn is_pinned(&self, desktop_id: &str) -> bool {
        self.pinned.iter().any(|id| id == desktop_id)
    }
}

pub fn get_dock_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".config/ermete-shell/dock.json")
}

pub fn load_dock_config() -> DockConfig {
    let path = get_dock_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<DockConfig>(&content) {
            return config;
        }
    }
    let default_config = DockConfig::default();
    if let Err(e) = save_dock_config(&default_config) {
        eprintln!("Warning: failed to save default dock config: {}", e);
    }
    default_config
}

pub fn save_dock_config(config: &DockConfig) -> Result<(), String> {
    let path = get_dock_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Impossibile creare directory {}: {}", parent.display(), e))?;
    }
    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Impossibile serializzare configurazione Dock: {}", e))?;
    fs::write(&path, json_str)
        .map_err(|e| format!("Impossibile scrivere file {}: {}", path.display(), e))?;
    Ok(())
}

pub fn add_pin(desktop_id: &str) -> Result<DockConfig, String> {
    let mut config = load_dock_config();
    if !config.is_pinned(desktop_id) {
        config.pinned.push(desktop_id.to_string());
        save_dock_config(&config)?;
    }
    Ok(config)
}

pub fn remove_pin(desktop_id: &str) -> Result<DockConfig, String> {
    let mut config = load_dock_config();
    config.pinned.retain(|id| id != desktop_id);
    save_dock_config(&config)?;
    Ok(config)
}

pub fn is_pinned(desktop_id: &str) -> bool {
    let config = load_dock_config();
    config.is_pinned(desktop_id)
}

#[cfg(test)]
pub(crate) static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    struct HomeGuard {
        original: Option<String>,
    }

    impl HomeGuard {
        fn set(new_home: &std::path::Path) -> Self {
            let original = std::env::var("HOME").ok();
            std::env::set_var("HOME", new_home.to_str().unwrap_or("/tmp"));
            Self { original }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(val) => std::env::set_var("HOME", val),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    fn test_add_and_remove_pin_logic() {
        let mut config = DockConfig {
            pinned: vec!["app1.desktop".to_string()],
        };
        
        // Test add
        if !config.is_pinned("app2.desktop") {
            config.pinned.push("app2.desktop".to_string());
        }
        assert_eq!(config.pinned, vec!["app1.desktop", "app2.desktop"]);

        // Test add duplicate (should not add)
        if !config.is_pinned("app2.desktop") {
            config.pinned.push("app2.desktop".to_string());
        }
        assert_eq!(config.pinned.len(), 2);

        // Test remove
        config.pinned.retain(|id| id != "app1.desktop");
        assert_eq!(config.pinned, vec!["app2.desktop"]);
        assert!(config.is_pinned("app2.desktop"));
        assert!(!config.is_pinned("app1.desktop"));
    }

    #[test]
    fn test_api_add_remove_and_is_pinned() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let tmp_dir = std::env::temp_dir().join("ermete_test_dock_config_api");
        let _ = fs::remove_dir_all(&tmp_dir);
        let _home_guard = HomeGuard::set(&tmp_dir);

        // Initial load should create default config
        let initial = load_dock_config();
        assert_eq!(initial, DockConfig::default());
        assert!(get_dock_config_path().exists(), "load_dock_config should save default if file didn't exist");

        // Test add_pin
        let added = add_pin("custom.app.desktop").unwrap_or(DockConfig::default());
        assert!(added.is_pinned("custom.app.desktop"));
        assert!(is_pinned("custom.app.desktop"));

        // Test adding duplicate
        let added_again = add_pin("custom.app.desktop").unwrap_or(DockConfig::default());
        let count = added_again.pinned.iter().filter(|id| *id == "custom.app.desktop").count();
        assert_eq!(count, 1, "duplicate pin should not be added");

        // Test remove_pin
        let removed = remove_pin("custom.app.desktop").unwrap_or(DockConfig::default());
        assert!(!removed.is_pinned("custom.app.desktop"));
        assert!(!is_pinned("custom.app.desktop"));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
