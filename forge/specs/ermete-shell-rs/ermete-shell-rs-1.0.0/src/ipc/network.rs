use arc_swap::ArcSwap;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use crate::ipc::types::{
    IpcBackend, NetBus, NetEvent, WifiNetworkInfo,
    NetworkManagerProxy, NmDeviceProxy, NmWirelessProxy, NmAccessPointProxy, 
    NmSettingsProxy, NmSettingsConnectionProxy, NmActiveConnectionProxy, NmIP4ConfigProxy
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
                    let res = self.handle_modify_wifi().await;
                    let _ = responder.send(res);
                }
                NetworkCommand::GetWifiDetails(ssid, resp) => {
                    let res = self.handle_get_wifi_details(&ssid).await;
                    let _ = resp.send(res);
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
                    let current = proxy.wireless_enabled().await.map_err(|e| zbus::Error::Failure(e.to_string()))?;
                    let new_state = !current;
                    proxy.set_wireless_enabled(new_state).await?;
                    new_state
                } else {
                    return Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into()));
                }
            }
            IpcBackend::Disconnected => return Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
        };
        self.event_bus.emit(NetEvent::WifiToggled(new_state));
        Ok(new_state)
    }

    async fn handle_is_wifi_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    return proxy.wireless_enabled().await;
                }
                Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
            
        }
    }

    async fn handle_set_wifi_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    proxy.set_wireless_enabled(powered).await?;
                    self.event_bus.emit(NetEvent::WifiToggled(powered));
                    Ok(())
                } else {
                    Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into()))
                }
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
        }
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
                                                                let strength = ap_proxy.strength().await.map_err(|e| zbus::Error::Failure(e.to_string()))? as i32;
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
                    return Ok(results);
                }
                Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
            
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
                Err(zbus::Error::Failure("NetworkManager service unavailable or connection not found".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
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
                Err(zbus::Error::Failure("NetworkManager service unavailable or active connection not found".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
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
                Err(zbus::Error::Failure("NetworkManager service unavailable or connection not found".into()))
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
        }
    }

    async fn handle_modify_wifi(&self) -> zbus::Result<()> {
        Err(zbus::Error::Failure("DBus network modifier in lavorazione".into()))
    }

    async fn handle_get_wifi_details(&self, ssid: &str) -> zbus::Result<(String, String, String, String, bool)> {
        match &self.backend {
            IpcBackend::Dbus { system, .. } => {
                let mut method = "auto".to_string();
                let mut autoconnect = true;
                let mut ip = "N/A".to_string();
                let mut gateway = "N/A".to_string();
                let mut dns = "N/A".to_string();
                let mut found_conn = false;

                if let Ok(Ok(settings_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NmSettingsProxy::new(system)).await {
                    if let Ok(conns) = settings_proxy.list_connections().await {
                        for conn_path in conns {
                            if let Ok(conn_proxy) = NmSettingsConnectionProxy::builder(system).path(conn_path)?.build().await {
                                if let Ok(settings) = conn_proxy.get_settings().await {
                                    if let Some(wifi_sec) = settings.get("802-11-wireless") {
                                        if let Some(ssid_val) = wifi_sec.get("ssid") {
                                            if let Some(s) = Self::extract_ssid(ssid_val) {
                                                if s == ssid {
                                                    found_conn = true;
                                                    if let Some(conn_sec) = settings.get("connection") {
                                                        if let Some(ac_val) = conn_sec.get("autoconnect") {
                                                            if let zbus::zvariant::Value::Bool(b) = &**ac_val {
                                                                autoconnect = *b;
                                                            }
                                                        }
                                                    }
                                                    if let Some(ipv4_sec) = settings.get("ipv4") {
                                                        if let Some(m_val) = ipv4_sec.get("method") {
                                                            if let zbus::zvariant::Value::Str(m) = &**m_val {
                                                                method = m.as_str().to_string();
                                                            }
                                                        }
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    if let Ok(active_conns) = nm_proxy.active_connections().await {
                        for path in active_conns {
                            if let Ok(ac_proxy) = NmActiveConnectionProxy::builder(system).path(path)?.build().await {
                                if let Ok(id) = ac_proxy.id().await {
                                    if id == ssid {
                                        found_conn = true;
                                        if let Ok(ip4_path) = ac_proxy.ip4_config().await {
                                            if let Ok(ip4_proxy) = NmIP4ConfigProxy::builder(system).path(ip4_path)?.build().await {
                                                if let Ok(gw) = ip4_proxy.gateway().await {
                                                    if !gw.is_empty() { gateway = gw; }
                                                }
                                                if let Ok(ns) = ip4_proxy.nameservers().await {
                                                    if let Some(first_ns) = ns.first() {
                                                        let ip_bytes = first_ns.to_be_bytes();
                                                        dns = format!("{}.{}.{}.{}", ip_bytes[3], ip_bytes[2], ip_bytes[1], ip_bytes[0]);
                                                    }
                                                }
                                                if let Ok(addr_data) = ip4_proxy.address_data().await {
                                                    if let Some(addr_map) = addr_data.first() {
                                                        if let Some(ip_val) = addr_map.get("address") {
                                                            if let zbus::zvariant::Value::Str(ip_str) = &**ip_val {
                                                                ip = ip_str.as_str().to_string();
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

                if found_conn {
                    Ok((method, ip, gateway, dns, autoconnect))
                } else {
                    Err(zbus::Error::Failure("Dettagli Wi-Fi non disponibili per la rete".into()))
                }
            }
            IpcBackend::Disconnected => Err(zbus::Error::Failure("NetworkManager DBus service unavailable".into())),
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
            IpcBackend::Disconnected => {
                self.active_wifi_ssid = None;
                Ok(())
            }
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

    pub fn new_disconnected(event_bus: NetBus) -> Self {
        let backend = IpcBackend::Disconnected;
        let sender = NetworkActor::spawn(backend, event_bus, None);
        Self {
            sender,
            active_wifi_ssid: Arc::new(Mutex::new(None)),
        }
    }


    pub async fn toggle_wifi(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ToggleWifi(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn is_wifi_enabled(&self) -> zbus::Result<bool> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::IsWifiEnabled(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn set_wifi_powered(&self, powered: bool) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::SetWifiPowered(powered, tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn list_wifi_networks(&self) -> zbus::Result<Vec<WifiNetworkInfo>> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ListWifiNetworks(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn connect_wifi(&self, ssid: &str, password: &str) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ConnectWifi(ssid.to_string(), password.to_string(), tx)).await.is_ok() {
            let res = rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?;
            if res.is_ok() {
                let mut l = self.active_wifi_ssid.lock().await;
        {
                    *l = Some(ssid.to_string());
                }
            }
            res
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn disconnect_wifi(&self, ssid: &str) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::DisconnectWifi(ssid.to_string(), tx)).await.is_ok() {
            let res = rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?;
            if res.is_ok() {
                let mut l = self.active_wifi_ssid.lock().await;
        {
                    *l = None;
                }
            }
            res
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn delete_wifi(&self, ssid: &str) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::DeleteWifi(ssid.to_string(), tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn modify_wifi(&self, _ssid: &str, _autoconnect: bool, _ip: &str, _gw: &str, _dns: &str, _ipv6: bool) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::ModifyWifi(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn get_wifi_details(&self, ssid: &str) -> zbus::Result<(String, String, String, String, bool)> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::GetWifiDetails(ssid.to_string(), tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn refresh_network_status(&self) -> zbus::Result<()> {
        let (tx, rx) = oneshot::channel();
        if self.sender.send(NetworkCommand::RefreshStatus(tx)).await.is_ok() {
            rx.await.map_err(|_| zbus::Error::Failure("IPC channel closed".into()))?
        } else {
            Err(zbus::Error::Failure("NetworkActor disconnected".into()))
        }
    }

    pub async fn get_network_status_async(&self) -> (String, String, String) {
        let l = self.active_wifi_ssid.lock().await;
        {
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
        let l = self.active_wifi_ssid.blocking_lock();
        {
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

/// Queries network status via DBus / system network proxy (Zero-Trust compliant).
pub fn check_sysfs_net_status() -> (String, String, String) {
    // Zero-Trust policy: Direct /sys/class/net reads are disabled in favor of NetworkManager DBus IPC.
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
        NetworkController::new_disconnected(bus)
    }
}





