use arc_swap::ArcSwap;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{
    IpcBackend, NetBus, NetEvent, MockState, WifiNetworkInfo,
    NetworkManagerProxy, NmDeviceProxy, NmWirelessProxy, NmAccessPointProxy, 
    NmSettingsProxy, NmSettingsConnectionProxy, NmActiveConnectionProxy
};

pub enum NetworkCommand {
    ToggleWifi(oneshot::Sender<zbus::Result<bool>>),
    IsWifiEnabled(oneshot::Sender<zbus::Result<bool>>),
    SetWifiPowered(bool, oneshot::Sender<zbus::Result<()>>),
    ListWifiNetworks(oneshot::Sender<zbus::Result<Vec<WifiNetworkInfo>>>),
    ConnectWifi(String, String, oneshot::Sender<zbus::Result<()>>),
    DisconnectWifi(String, oneshot::Sender<zbus::Result<()>>),
    DeleteWifi(String, oneshot::Sender<zbus::Result<()>>),
    ModifyWifi(oneshot::Sender<zbus::Result<()>>),
    #[allow(clippy::type_complexity)]
    GetWifiDetails(String, oneshot::Sender<zbus::Result<(String, String, String, String, bool)>>),
    RefreshStatus(oneshot::Sender<zbus::Result<()>>),
}

pub struct NetworkActor {
    backend: IpcBackend,
    active_wifi_ssid: Option<String>,
    event_bus: NetBus,
    receiver: mpsc::Receiver<NetworkCommand>,
}

impl NetworkActor {
    pub fn spawn(backend: IpcBackend, event_bus: NetBus, initial_ssid: Option<String>) -> mpsc::Sender<NetworkCommand> {
        let (tx, rx) = mpsc::channel(32);
        let actor = Self {
            backend,
            active_wifi_ssid: initial_ssid,
            event_bus,
            receiver: rx,
        };
        tokio::spawn(actor.run());
        tx
    }

    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                NetworkCommand::ToggleWifi(resp) => {
                    let res = self.handle_toggle_wifi().await;
                    let _ = resp.send(res);
                }
                NetworkCommand::IsWifiEnabled(resp) => {
                    let res = self.handle_is_wifi_enabled().await;
                    let _ = resp.send(res);
                }
                NetworkCommand::SetWifiPowered(powered, resp) => {
                    let res = self.handle_set_wifi_powered(powered).await;
                    let _ = resp.send(res);
                }
                NetworkCommand::ListWifiNetworks(resp) => {
                    let res = self.handle_list_wifi_networks().await;
                    let _ = resp.send(res);
                }
                NetworkCommand::ConnectWifi(ssid, pass, resp) => {
                    let res = self.handle_connect_wifi(&ssid, &pass).await;
                    let _ = resp.send(res);
                }
                NetworkCommand::DisconnectWifi(ssid, resp) => {
                    let res = self.handle_disconnect_wifi(&ssid).await;
                    let _ = resp.send(res);
                }
                NetworkCommand::DeleteWifi(ssid, resp) => {
                    let res = self.handle_delete_wifi(&ssid).await;
                    let _ = resp.send(res);
                }
                NetworkCommand::ModifyWifi(responder) => {
                    let _ = responder.send(Ok(()));
                }
                NetworkCommand::GetWifiDetails(_ssid, resp) => {
                    let _ = resp.send(Ok(("auto".to_string(), "192.168.1.100".to_string(), "192.168.1.1".to_string(), "8.8.8.8".to_string(), true)));
                }
                NetworkCommand::RefreshStatus(resp) => {
                    let res = self.handle_refresh_network_status().await;
                    let _ = resp.send(res);
                }
            }
        }
    }

    async fn handle_toggle_wifi(&self) -> zbus::Result<bool> {
        let new_state = match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    let current = proxy.wireless_enabled().await.unwrap_or(true);
                    let new_state = !current;
                    proxy.set_wireless_enabled(new_state).await?;
                    new_state
                } else {
                    true
                }
            }
            IpcBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.wifi_enabled = !s.wifi_enabled;
                s.wifi_enabled
            }
        };
        self.event_bus.emit(NetEvent::WifiToggled(new_state));
        Ok(new_state)
    }

    async fn handle_is_wifi_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    return Ok(proxy.wireless_enabled().await.unwrap_or(true));
                }
                Ok(true)
            }
            IpcBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).wifi_enabled),
        }
    }

    async fn handle_set_wifi_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    proxy.set_wireless_enabled(powered).await?;
                }
            }
            IpcBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).wifi_enabled = powered;
            }
        }
        self.event_bus.emit(NetEvent::WifiToggled(powered));
        Ok(())
    }

    async fn handle_list_wifi_networks(&self) -> zbus::Result<Vec<WifiNetworkInfo>> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                let mut results = Vec::new();
                if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    if let Ok(devices) = nm_proxy.get_devices().await {
                        for dev_path in devices {
                            if let Ok(dev_proxy) = NmDeviceProxy::builder(system).path(dev_path.clone())?.build().await {
                                if let Ok(dev_type) = dev_proxy.device_type().await {
                                    if dev_type == 2 {
                                        if let Ok(wifi_proxy) = NmWirelessProxy::builder(system).path(dev_path)?.build().await {
                                            if let Ok(aps) = wifi_proxy.get_access_points().await {
                                                for ap_path in aps {
                                                    if let Ok(ap_proxy) = NmAccessPointProxy::builder(system).path(ap_path)?.build().await {
                                                        if let Ok(ssid_bytes) = ap_proxy.ssid().await {
                                                            let ssid = String::from_utf8_lossy(&ssid_bytes).trim().to_string();
                                                            if !ssid.is_empty() {
                                                                let strength = ap_proxy.strength().await.unwrap_or(50) as i32;
                                                                results.push(WifiNetworkInfo {
                                                                    ssid,
                                                                    signal: strength,
                                                                    active: false,
                                                                    saved: false,
                                                                });
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(results)
            }
            IpcBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).wifi_networks.clone()),
        }
    }

    fn extract_ssid(val: &zbus::zvariant::Value) -> Option<String> {
        if let zbus::zvariant::Value::Array(arr) = val {
            let bytes: std::vec::Vec<u8> = arr.iter().filter_map(|v| match v {
                zbus::zvariant::Value::U8(b) => Some(*b),
                _ => None,
            }).collect();
            Some(String::from_utf8_lossy(&bytes).to_string())
        } else if let zbus::zvariant::Value::Str(s) = val {
            Some(s.as_str().to_string())
        } else {
            None
        }
    }

    async fn handle_connect_wifi(&mut self, ssid: &str, _password: &str) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(settings_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NmSettingsProxy::new(system)).await {
                    if let Ok(conns) = settings_proxy.list_connections().await {
                        for conn_path in conns {
                            if let Ok(conn_proxy) = NmSettingsConnectionProxy::builder(system).path(conn_path.clone())?.build().await {
                                if let Ok(settings) = conn_proxy.get_settings().await {
                                    if let Some(wifi_sec) = settings.get("802-11-wireless") {
                                        if let Some(ssid_val) = wifi_sec.get("ssid") {
                                            if let Some(s) = Self::extract_ssid(ssid_val) {
                                                if s == ssid {
                                                    if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                                                        let mut device_path = zbus::zvariant::ObjectPath::from_str_unchecked("/");
                                                        if let Ok(devices) = nm_proxy.get_devices().await {
                                                            for dev_path in devices {
                                                                if let Ok(dev_proxy) = NmDeviceProxy::builder(system).path(dev_path.clone())?.build().await {
                                                                    if let Ok(dev_type) = dev_proxy.device_type().await {
                                                                        if dev_type == 2 {
                                                                            device_path = dev_path.into_inner();
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        let _ = nm_proxy.activate_connection(&conn_path, &device_path, &zbus::zvariant::ObjectPath::from_str_unchecked("/")).await?;
                                                        self.active_wifi_ssid = Some(ssid.to_string());
                                                        self.event_bus.emit(NetEvent::NetworkUpdated(ssid.to_string()));
                                                        return Ok(());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            IpcBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                for net in &mut s.wifi_networks {
                    net.active = net.ssid == ssid;
                }
                self.event_bus.emit(NetEvent::NetworkUpdated(ssid.to_string()));
                Ok(())
            }
        }
    }

    async fn handle_disconnect_wifi(&mut self, ssid: &str) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    if let Ok(active_conns) = nm_proxy.active_connections().await {
                        for path in active_conns {
                            if let Ok(ac_proxy) = NmActiveConnectionProxy::builder(system).path(path.clone())?.build().await {
                                if let Ok(id) = ac_proxy.id().await {
                                    if id == ssid {
                                        nm_proxy.deactivate_connection(&path).await?;
                                        self.active_wifi_ssid = None;
                                        self.event_bus.emit(NetEvent::NetworkUpdated("Disconnected".to_string()));
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            IpcBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                for net in &mut s.wifi_networks {
                    if net.ssid == ssid {
                        net.active = false;
                    }
                }
                self.event_bus.emit(NetEvent::NetworkUpdated("Disconnected".to_string()));
                Ok(())
            }
        }
    }

    async fn handle_delete_wifi(&self, ssid: &str) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(settings_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NmSettingsProxy::new(system)).await {
                    if let Ok(conns) = settings_proxy.list_connections().await {
                        for conn_path in conns {
                            if let Ok(conn_proxy) = NmSettingsConnectionProxy::builder(system).path(conn_path)?.build().await {
                                if let Ok(settings) = conn_proxy.get_settings().await {
                                    if let Some(wifi_sec) = settings.get("802-11-wireless") {
                                        if let Some(ssid_val) = wifi_sec.get("ssid") {
                                            if let Some(s) = Self::extract_ssid(ssid_val) {
                                                if s == ssid {
                                                    conn_proxy.delete().await?;
                                                    return Ok(());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            IpcBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.wifi_networks.retain(|net| net.ssid != ssid);
                Ok(())
            }
        }
    }

    async fn handle_refresh_network_status(&mut self) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    if let Ok(active_conns) = nm_proxy.active_connections().await {
                        for path in active_conns {
                            if let Ok(ac_proxy) = NmActiveConnectionProxy::builder(system).path(path)?.build().await {
                                if let Ok(id) = ac_proxy.id().await {
                                    self.active_wifi_ssid = Some(id);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                self.active_wifi_ssid = None;
                Ok(())
            }
            IpcBackend::Mock(_) => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkController {
    sender: mpsc::Sender<NetworkCommand>,
    active_wifi_ssid: Arc<Mutex<Option<String>>>,
}

impl NetworkController {
    pub fn new(backend: IpcBackend, event_bus: NetBus) -> Self {
        let sender = NetworkActor::spawn(backend, event_bus, None);
        Self {
            sender,
            active_wifi_ssid: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: NetBus) -> Self {
        let backend = IpcBackend::Mock(state);
        let sender = NetworkActor::spawn(backend, event_bus, Some("Ermete-5G".to_string()));
        Self {
            sender,
            active_wifi_ssid: Arc::new(Mutex::new(Some("Ermete-5G".to_string()))),
        }
    }

    pub async fn toggle_wifi(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ToggleWifi(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(true))
        } else {
            Ok(true)
        }
    }

    pub async fn is_wifi_enabled(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::IsWifiEnabled(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(true))
        } else {
            Ok(true)
        }
    }

    pub async fn set_wifi_powered(&self, powered: bool) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::SetWifiPowered(powered, tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn list_wifi_networks(&self) -> zbus::Result<Vec<WifiNetworkInfo>> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ListWifiNetworks(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(Vec::new()))
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn connect_wifi(&self, ssid: &str, password: &str) -> zbus::Result<()> {
        if let Ok(mut l) = self.active_wifi_ssid.lock() {
            *l = Some(ssid.to_string());
        }
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ConnectWifi(ssid.to_string(), password.to_string(), tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn disconnect_wifi(&self, ssid: &str) -> zbus::Result<()> {
        if let Ok(mut l) = self.active_wifi_ssid.lock() {
            *l = None;
        }
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::DisconnectWifi(ssid.to_string(), tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn delete_wifi(&self, ssid: &str) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::DeleteWifi(ssid.to_string(), tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn modify_wifi(&self, _ssid: &str, _autoconnect: bool, _ip: &str, _gw: &str, _dns: &str, _ipv6: bool) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ModifyWifi(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn get_wifi_details(&self, ssid: &str) -> zbus::Result<(String, String, String, String, bool)> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::GetWifiDetails(ssid.to_string(), tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(("auto".to_string(), "192.168.1.100".to_string(), "192.168.1.1".to_string(), "8.8.8.8".to_string(), true)))
        } else {
            Ok(("auto".to_string(), "192.168.1.100".to_string(), "192.168.1.1".to_string(), "8.8.8.8".to_string(), true))
        }
    }

    pub async fn refresh_network_status(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::RefreshStatus(tx)).await.is_ok() {
            rx.await.unwrap_or(Ok(()))
        } else {
            Ok(())
        }
    }

    pub async fn get_network_status_async(&self) -> (String, String, String) {
        if let Ok(l) = self.active_wifi_ssid.lock() {
            if let Some(ssid) = l.as_ref() {
                let status = ("".to_string(), "Rete Wi-Fi".to_string(), ssid.clone());
                get_net_cache().store(Arc::new(status.clone()));
                return status;
            }
        }
        let status = tokio::task::spawn_blocking(check_sysfs_net_status)
            .await
            .unwrap_or_else(|_| ("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Non connesso".to_string()));
        get_net_cache().store(Arc::new(status.clone()));
        status
    }

    pub fn get_cached_network_status(&self) -> (String, String, String) {
        if let Ok(l) = self.active_wifi_ssid.lock() {
            if let Some(ssid) = l.as_ref() {
                return ("".to_string(), "Rete Wi-Fi".to_string(), ssid.clone());
            }
        }

        let cached = (**get_net_cache().load()).clone();

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async {
                let updated = tokio::task::spawn_blocking(check_sysfs_net_status)
                    .await
                    .unwrap_or_else(|_| ("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Non connesso".to_string()));
                get_net_cache().store(Arc::new(updated));
            });
        }

        cached
    }
}

static NET_STATUS_CACHE: OnceLock<ArcSwap<(String, String, String)>> = OnceLock::new();

fn get_net_cache() -> &'static ArcSwap<(String, String, String)> {
    NET_STATUS_CACHE.get_or_init(|| {
        ArcSwap::from_pointee(("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Non connesso".to_string()))
    })
}

/// Reads `/sys/class/net` for network interface operstates off the main thread.
pub fn check_sysfs_net_status() -> (String, String, String) {
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "lo" {
                continue;
            }
            if let Ok(state) = std::fs::read_to_string(entry.path().join("operstate")) {
                if state.trim() == "up" {
                    if name.starts_with("eth") || name.starts_with("en") {
                        return ("󰈀".to_string(), "Ethernet".to_string(), "Connesso via cavo".to_string());
                    } else if name.starts_with("wl") {
                        return ("".to_string(), "Rete Wi-Fi".to_string(), "Connesso".to_string());
                    }
                }
            }
        }
    }
    ("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Non connesso".to_string())
}

impl crate::ipc::system_proxies::ControllerBackend for NetworkController {
    fn name(&self) -> &'static str {
        "network"
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get_network_controller() -> NetworkController {
    if let Some(ctrl) = crate::ipc::system_proxies::get_registry().get_typed::<NetworkController>("network") {
        ctrl
    } else {
        let bus = crate::ipc::system_proxies::get_net_bus();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        NetworkController::new_mock(state, bus)
    }
}

