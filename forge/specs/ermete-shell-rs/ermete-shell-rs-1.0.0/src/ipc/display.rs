use zbus::proxy;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{ControllerBackend, SystemEventBus, SystemEvent, MockState};

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto"
)]
pub trait LogindSession {
    fn set_brightness(&self, subsystem: &str, name: &str, value: u32) -> zbus::Result<()>;
}

pub enum DisplayCommand {
    SetBrightness(f64, oneshot::Sender<zbus::Result<()>>),
}

pub struct DisplayActor {
    backend: ControllerBackend,
    event_bus: SystemEventBus,
    receiver: mpsc::Receiver<DisplayCommand>,
}

impl DisplayActor {
    pub fn spawn(backend: ControllerBackend, event_bus: SystemEventBus) -> mpsc::Sender<DisplayCommand> {
        let (tx, rx) = mpsc::channel(32);
        let actor = Self {
            backend,
            event_bus,
            receiver: rx,
        };
        tokio::spawn(actor.run());
        tx
    }

    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                DisplayCommand::SetBrightness(b, resp) => {
                    let res = self.handle_set_brightness(b).await;
                    let _ = resp.send(res);
                }
            }
        }
    }

    async fn handle_set_brightness(&self, brightness: f64) -> zbus::Result<()> {
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
}

#[derive(Clone, Debug)]
pub struct DisplayController {
    sender: mpsc::Sender<DisplayCommand>,
}

impl DisplayController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        let sender = DisplayActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        let backend = ControllerBackend::Mock(state);
        let sender = DisplayActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub async fn set_brightness(&self, brightness: f64) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(DisplayCommand::SetBrightness(brightness, tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }
}
