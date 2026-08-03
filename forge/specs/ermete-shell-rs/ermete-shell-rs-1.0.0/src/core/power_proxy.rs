use zbus::proxy;
use std::sync::{Arc, Mutex};
use crate::core::system_proxies::{ControllerBackend, MockState, SystemEventBus};

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

#[derive(Clone, Debug)]
pub struct PowerController {
    backend: ControllerBackend,
    event_bus: SystemEventBus,
}

impl PowerController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self { backend, event_bus }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            event_bus,
        }
    }

    pub async fn lock_screen(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.lock_sessions().await;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }

    pub async fn power_off(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.power_off(true).await;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }

    pub async fn reboot(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.reboot(true).await;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }

    pub async fn suspend(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await {
                    let _ = proxy.suspend(true).await;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }
}
