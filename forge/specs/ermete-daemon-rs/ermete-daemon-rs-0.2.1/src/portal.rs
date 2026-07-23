use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::interface;
use zbus::zvariant::{OwnedValue, Value};
use crate::settings::SettingsState;

#[derive(Clone)]
pub struct PortalSettingsService {
    pub state: Arc<Mutex<SettingsState>>,
}

impl PortalSettingsService {
    pub fn new(state: Arc<Mutex<SettingsState>>) -> Self {
        Self { state }
    }

    pub fn parse_hex_rgb(hex: &str) -> (f64, f64, f64) {
        let clean = hex.trim_start_matches('#');
        if clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(137) as f64 / 255.0;
            let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(180) as f64 / 255.0;
            let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(250) as f64 / 255.0;
            (r, g, b)
        } else {
            (0.537, 0.706, 0.980) // default accent (#89b4fa)
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.Settings")]
impl PortalSettingsService {
    async fn read(&self, namespace: String, key: String) -> zbus::fdo::Result<OwnedValue> {
        let st = self.state.lock().await;
        if namespace == "org.freedesktop.appearance" {
            match key.as_str() {
                "color-scheme" => {
                    let val: u32 = if st.color_scheme == "prefer-dark" { 1 } else { 0 };
                    let v = Value::from(val);
                    v.try_into().map_err(|e| zbus::fdo::Error::Failed(format!("Conversion error: {}", e)))
                }
                "accent-color" => {
                    let (r, g, b) = Self::parse_hex_rgb(&st.accent_color);
                    let v = Value::from((r, g, b));
                    v.try_into().map_err(|e| zbus::fdo::Error::Failed(format!("Conversion error: {}", e)))
                }
                "contrast" => {
                    let v = Value::from(0u32);
                    v.try_into().map_err(|e| zbus::fdo::Error::Failed(format!("Conversion error: {}", e)))
                }
                _ => Err(zbus::fdo::Error::InvalidArgs(format!("Unknown appearance key: {}", key))),
            }
        } else {
            Err(zbus::fdo::Error::InvalidArgs(format!("Unknown namespace: {}", namespace)))
        }
    }

    async fn read_all(&self, namespaces: Vec<String>) -> zbus::fdo::Result<HashMap<String, HashMap<String, OwnedValue>>> {
        let st = self.state.lock().await;
        let mut result = HashMap::new();

        if namespaces.is_empty() || namespaces.iter().any(|n| n == "org.freedesktop.appearance") {
            let mut appearance = HashMap::new();
            
            let scheme_val: u32 = if st.color_scheme == "prefer-dark" { 1 } else { 0 };
            if let Ok(ov) = Value::from(scheme_val).try_into() {
                appearance.insert("color-scheme".to_string(), ov);
            }

            let (r, g, b) = Self::parse_hex_rgb(&st.accent_color);
            if let Ok(ov) = Value::from((r, g, b)).try_into() {
                appearance.insert("accent-color".to_string(), ov);
            }

            if let Ok(ov) = Value::from(0u32).try_into() {
                appearance.insert("contrast".to_string(), ov);
            }

            result.insert("org.freedesktop.appearance".to_string(), appearance);
        }

        Ok(result)
    }

    #[zbus(signal)]
    pub async fn setting_changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        namespace: &str,
        key: &str,
        value: Value<'_>,
    ) -> zbus::Result<()>;
}
