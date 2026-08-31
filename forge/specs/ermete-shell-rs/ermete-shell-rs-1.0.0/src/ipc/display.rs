use zbus::proxy;
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{IpcBackend, HardwareBus, HardwareEvent};

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
    backend: IpcBackend,
    event_bus: HardwareBus,
    receiver: mpsc::Receiver<DisplayCommand>,
}

impl DisplayActor {
    pub fn spawn(backend: IpcBackend, event_bus: HardwareBus) -> mpsc::Sender<DisplayCommand> {
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
            IpcBackend::Dbus { system, .. } => {
                let val = (brightness * 100.0) as u32;
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindSessionProxy::new(system)).await {
                    proxy.set_brightness("backlight", "intel_backlight", val).await?;
                } else {
                    tracing::warn!("[Zero-Trust] LogindSessionProxy unavailable for setting brightness via DBus; direct sysfs write bypass blocked.");
                }
            }
            IpcBackend::Disconnected => {}
        }
        self.event_bus.emit(HardwareEvent::BrightnessChanged(brightness));
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct DisplayController {
    sender: mpsc::Sender<DisplayCommand>,
}

impl DisplayController {
    pub fn new(backend: IpcBackend, event_bus: HardwareBus) -> Self {
        let sender = DisplayActor::spawn(backend, event_bus);
        Self { sender }
    }


    pub async fn set_brightness(&self, brightness: f64) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(DisplayCommand::SetBrightness(brightness, tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else { Err(zbus::Error::Failure("Actor channel offline".into())) }
    }
}

impl crate::ipc::system_proxies::ControllerBackend for DisplayController {
    fn name(&self) -> &'static str {
        "display"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_display_controller() -> DisplayController {
    if let Some(ctrl) = crate::ipc::system_proxies::get_registry().get_typed::<DisplayController>("display") {
        ctrl
    } else {
        let bus = crate::ipc::system_proxies::get_hardware_bus();
        DisplayController::new(IpcBackend::Disconnected, bus)
    }
}

