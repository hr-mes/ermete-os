use zbus::proxy;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{IpcBackend, MockState, SystemEventBus};

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait Logind {
    fn lock_sessions(&self) -> zbus::Result<()>;
    fn power_off(&self, interactive: bool) -> zbus::Result<()>;
    fn reboot(&self, interactive: bool) -> zbus::Result<()>;
    fn suspend(&self, interactive: bool) -> zbus::Result<()>;
}

pub enum PowerCommand {
    LockScreen(oneshot::Sender<zbus::Result<()>>),
    PowerOff(oneshot::Sender<zbus::Result<()>>),
    Reboot(oneshot::Sender<zbus::Result<()>>),
    Suspend(oneshot::Sender<zbus::Result<()>>),
}

pub struct PowerActor {
    backend: IpcBackend,
    _event_bus: SystemEventBus,
    receiver: mpsc::Receiver<PowerCommand>,
}

impl PowerActor {
    pub fn spawn(backend: IpcBackend, event_bus: SystemEventBus) -> mpsc::Sender<PowerCommand> {
        let (tx, rx) = mpsc::channel(32);
        let actor = Self {
            backend,
            _event_bus: event_bus,
            receiver: rx,
        };
        tokio::spawn(actor.run());
        tx
    }

    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                PowerCommand::LockScreen(resp) => {
                    let res = self.handle_lock_screen().await;
                    let _ = resp.send(res);
                }
                PowerCommand::PowerOff(resp) => {
                    let res = self.handle_power_off().await;
                    let _ = resp.send(res);
                }
                PowerCommand::Reboot(resp) => {
                    let res = self.handle_reboot().await;
                    let _ = resp.send(res);
                }
                PowerCommand::Suspend(resp) => {
                    let res = self.handle_suspend().await;
                    let _ = resp.send(res);
                }
            }
        }
    }

    async fn handle_lock_screen(&self) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.lock_sessions().await;
                }
                Ok(())
            }
            IpcBackend::Mock(_) => Ok(()),
        }
    }

    async fn handle_power_off(&self) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.power_off(true).await;
                }
                Ok(())
            }
            IpcBackend::Mock(_) => Ok(()),
        }
    }

    async fn handle_reboot(&self) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.reboot(true).await;
                }
                Ok(())
            }
            IpcBackend::Mock(_) => Ok(()),
        }
    }

    async fn handle_suspend(&self) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.suspend(true).await;
                }
                Ok(())
            }
            IpcBackend::Mock(_) => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PowerController {
    sender: mpsc::Sender<PowerCommand>,
}

impl PowerController {
    pub fn new(backend: IpcBackend, event_bus: SystemEventBus) -> Self {
        let sender = PowerActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        let backend = IpcBackend::Mock(state);
        let sender = PowerActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub async fn lock_screen(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(PowerCommand::LockScreen(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn power_off(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(PowerCommand::PowerOff(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn reboot(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(PowerCommand::Reboot(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn suspend(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(PowerCommand::Suspend(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }
}

impl crate::ipc::system_proxies::ControllerBackend for PowerController {
    fn name(&self) -> &'static str {
        "power"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_power_controller() -> PowerController {
    if let Some(ctrl) = crate::ipc::system_proxies::get_registry().get_typed::<PowerController>("power") {
        ctrl
    } else {
        let bus = crate::ipc::system_proxies::get_event_bus();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        PowerController::new_mock(state, bus)
    }
}
