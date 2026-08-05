use zbus::proxy;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use crate::ipc::system_proxies::{ControllerBackend, MockState, SystemEvent, SystemEventBus, BluetoothDeviceInfo};

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

pub enum BluetoothCommand {
    ToggleBluetooth(oneshot::Sender<zbus::Result<bool>>),
    IsBluetoothEnabled(oneshot::Sender<zbus::Result<bool>>),
    SetBluetoothPowered(bool, oneshot::Sender<zbus::Result<()>>),
    ListBluetoothDevices(oneshot::Sender<zbus::Result<Vec<BluetoothDeviceInfo>>>),
}

pub struct BluetoothActor {
    backend: ControllerBackend,
    event_bus: SystemEventBus,
    receiver: mpsc::Receiver<BluetoothCommand>,
}

impl BluetoothActor {
    pub fn spawn(backend: ControllerBackend, event_bus: SystemEventBus) -> mpsc::Sender<BluetoothCommand> {
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
                BluetoothCommand::ToggleBluetooth(resp) => {
                    let res = self.handle_toggle_bluetooth().await;
                    let _ = resp.send(res);
                }
                BluetoothCommand::IsBluetoothEnabled(resp) => {
                    let res = self.handle_is_bluetooth_enabled().await;
                    let _ = resp.send(res);
                }
                BluetoothCommand::SetBluetoothPowered(powered, resp) => {
                    let res = self.handle_set_bluetooth_powered(powered).await;
                    let _ = resp.send(res);
                }
                BluetoothCommand::ListBluetoothDevices(resp) => {
                    let res = self.handle_list_bluetooth_devices().await;
                    let _ = resp.send(res);
                }
            }
        }
    }

    async fn handle_toggle_bluetooth(&self) -> zbus::Result<bool> {
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

    async fn handle_is_bluetooth_enabled(&self) -> zbus::Result<bool> {
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

    async fn handle_set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> {
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

    async fn handle_list_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDeviceInfo>> {
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

#[derive(Clone, Debug)]
pub struct BluetoothController {
    sender: mpsc::Sender<BluetoothCommand>,
}

impl BluetoothController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        let sender = BluetoothActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        let backend = ControllerBackend::Mock(state);
        let sender = BluetoothActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub async fn toggle_bluetooth(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::ToggleBluetooth(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(false))
        } else {
            Ok(false)
        }
    }

    pub async fn is_bluetooth_enabled(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::IsBluetoothEnabled(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(true))
        } else {
            Ok(true)
        }
    }

    pub async fn set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::SetBluetoothPowered(powered, tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn list_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDeviceInfo>> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::ListBluetoothDevices(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(Vec::new()))
        } else {
            Ok(Vec::new())
        }
    }
}
