use zbus::proxy;
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{IpcBackend,  NetEvent, NetBus, BluetoothDeviceInfo};

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
    backend: IpcBackend,
    event_bus: NetBus,
    receiver: mpsc::Receiver<BluetoothCommand>,
}

impl BluetoothActor {
    pub fn spawn(backend: IpcBackend, event_bus: NetBus) -> mpsc::Sender<BluetoothCommand> {
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
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await {
                    let current = proxy.powered().await?;
                    let new_st = !current;
                    proxy.set_powered(new_st).await?;
                    new_st
                } else {
                    return Err(zbus::Error::Failure("BlueZ Service Offline".into()));
                }
            }
            IpcBackend::Disconnected => return Err(zbus::Error::Failure("BlueZ Service Offline".into())),
        };
        self.event_bus.emit(NetEvent::BluetoothToggled(new_state));
        Ok(new_state)
    }

    async fn handle_is_bluetooth_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await {
                    return proxy.powered().await;
                }
                Err(zbus::Error::Failure("BlueZ Service Offline".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("BlueZ Service Offline".into())),
            
        }
    }

    async fn handle_set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await {
                    proxy.set_powered(powered).await?;
                    self.event_bus.emit(NetEvent::BluetoothToggled(powered));
                    Ok(())
                } else {
                    Err(zbus::Error::Failure("BlueZ Service Offline".into()))
                }
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("BlueZ Service Offline".into())),
        }
    }

    async fn handle_list_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDeviceInfo>> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                let mut results = Vec::new();
                if let Ok(Ok(obj_mgr)) = tokio::time::timeout(std::time::Duration::from_secs(5), zbus::fdo::ObjectManagerProxy::builder(system)
                    .destination("org.bluez")?
                    .path("/")?
                    .build()).await
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
                                    .ok_or_else(|| zbus::Error::Failure("Property not found".into()))?;
                                results.push(BluetoothDeviceInfo { name, connected });
                            }
                        }
                        return Ok(results);
                    }
                }
                Err(zbus::Error::Failure("BlueZ Service Offline".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("BlueZ Service Offline".into())),
            
        }
    }
}

#[derive(Clone, Debug)]
pub struct BluetoothController {
    sender: mpsc::Sender<BluetoothCommand>,
}

impl BluetoothController {
    pub fn new(backend: IpcBackend, event_bus: NetBus) -> Self {
        let sender = BluetoothActor::spawn(backend, event_bus);
        Self { sender }
    }

    pub fn new_disconnected(event_bus: NetBus) -> Self {
        let backend = IpcBackend::Disconnected;
        let sender = BluetoothActor::spawn(backend, event_bus);
        Self { sender }
    }


    pub async fn toggle_bluetooth(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::ToggleBluetooth(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("BluetoothActor disconnected".into()))
        }
    }

    pub async fn is_bluetooth_enabled(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::IsBluetoothEnabled(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("BluetoothActor disconnected".into()))
        }
    }

    pub async fn set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::SetBluetoothPowered(powered, tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("BluetoothActor disconnected".into()))
        }
    }

    pub async fn list_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDeviceInfo>> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(BluetoothCommand::ListBluetoothDevices(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("BluetoothActor disconnected".into()))
        }
    }
}

impl crate::ipc::system_proxies::ControllerBackend for BluetoothController {
    fn name(&self) -> &'static str {
        "bluetooth"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_bluetooth_controller() -> BluetoothController {
    if let Some(ctrl) = crate::ipc::system_proxies::get_registry().get_typed::<BluetoothController>("bluetooth") {
        ctrl
    } else {
        let bus = crate::ipc::system_proxies::get_net_bus();
        BluetoothController::new_disconnected(bus)
    }
}



