use zbus::proxy;
use std::sync::{Arc, Mutex};
use crate::core::system_proxies::{ControllerBackend, SystemEventBus, SystemEvent, MockState};

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto"
)]
pub trait LogindSession {
    fn set_brightness(&self, subsystem: &str, name: &str, value: u32) -> zbus::Result<()>;
}

#[derive(Clone, Debug)]
pub struct DisplayController {
    backend: ControllerBackend,
    event_bus: SystemEventBus,
}

impl DisplayController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self { backend, event_bus }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            event_bus,
        }
    }

    pub async fn set_brightness(&self, brightness: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                let val = (brightness * 100.0) as u32;
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindSessionProxy::new(system)).await {
                    proxy.set_brightness("backlight", "intel_backlight", val).await?;
                } else if let Ok(entries) = std::fs::read_dir("/sys/class/backlight") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Ok(max_str) = std::fs::read_to_string(path.join("max_brightness")) {
                            if let Ok(max_val) = max_str.trim().parse::<f64>() {
                                let target = (brightness * max_val).round() as u64;
                                let _ = std::fs::write(path.join("brightness"), target.to_string());
                            }
                        }
                    }
                }
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.brightness = brightness;
            }
        }
        self.event_bus.emit(SystemEvent::BrightnessChanged(brightness));
        Ok(())
    }

    pub async fn get_brightness(&self) -> zbus::Result<f64> {
        match &self.backend {
            ControllerBackend::Dbus { .. } => {
                let live = crate::core::live_state::get_live_state();
                Ok(live.brightness / 100.0)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).brightness),
        }
    }
}
