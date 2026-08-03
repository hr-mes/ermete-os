use zbus::proxy;
use std::sync::{Arc, Mutex};
use crate::core::system_proxies::{ControllerBackend, MockState, SystemEvent, SystemEventBus, BluetoothDeviceInfo};

#[proxy(
    interface = "org.bluez.Adapter1",
    default_service = "org.bluez",
    default_path = "/org/bluez/hci0"
)]
pub trait BlueZ {
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_powered(&self, val: bool) -> zbus::Result<()>;
}

#[derive(Clone, Debug)]
pub struct BluetoothController {
    backend: ControllerBackend,
    event_bus: SystemEventBus,
}

impl BluetoothController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self { backend, event_bus }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            event_bus,
        }
    }

    pub async fn toggle_bluetooth(&self) -> zbus::Result<bool> {
        let new_state = match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await {
                    let current = proxy.powered().await.unwrap_or(false);
                    let new_st = !current;
                    proxy.set_powered(new_st).await?;
                    new_st
                } else {
                    true
                }
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.bt_enabled = !s.bt_enabled;
                s.bt_enabled
            }
        };
        self.event_bus.emit(SystemEvent::BluetoothToggled(new_state));
        Ok(new_state)
    }

    pub async fn is_bluetooth_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await {
                    return Ok(proxy.powered().await.unwrap_or(true));
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).bt_enabled),
        }
    }

    pub async fn set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await {
                    proxy.set_powered(powered).await?;
                }
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).bt_enabled = powered;
            }
        }
        self.event_bus.emit(SystemEvent::BluetoothToggled(powered));
        Ok(())
    }

    pub async fn list_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDeviceInfo>> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                let mut results = Vec::new();
                if let Ok(obj_mgr) = zbus::fdo::ObjectManagerProxy::builder(system)
                    .destination("org.bluez")?
                    .path("/")?
                    .build().await
                {
                    if let Ok(objects) = obj_mgr.get_managed_objects().await {
                        for (path, interfaces) in objects {
                            if let Some(dev_props) = interfaces.get("org.bluez.Device1") {
                                let name = dev_props.get("Alias")
                                    .or_else(|| dev_props.get("Name"))
                                    .and_then(|v| String::try_from(&**v).ok())
                                    .unwrap_or_else(|| path.to_string());
                                let connected = dev_props.get("Connected")
                                    .and_then(|v| bool::try_from(&**v).ok())
                                    .unwrap_or(false);
                                results.push(BluetoothDeviceInfo { name, connected });
                            }
                        }
                    }
                }
                Ok(results)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).bt_devices.clone()),
        }
    }
}
