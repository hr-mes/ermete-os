use std::sync::{Arc, Mutex};
use crate::core::system_proxies::{
    ControllerBackend, SystemEventBus, SystemEvent, MockState, WifiNetworkInfo,
    NetworkManagerProxy, NmDeviceProxy, NmWirelessProxy, NmAccessPointProxy, 
    NmSettingsProxy, NmSettingsConnectionProxy, NmActiveConnectionProxy
};

#[derive(Clone, Debug)]
pub struct NetworkController {
    backend: ControllerBackend,
    active_wifi_ssid: Arc<Mutex<Option<String>>>,
    event_bus: SystemEventBus,
}

impl NetworkController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self {
            backend,
            active_wifi_ssid: Arc::new(Mutex::new(None)),
            event_bus,
        }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            active_wifi_ssid: Arc::new(Mutex::new(Some("Ermete-5G".to_string()))),
            event_bus,
        }
    }

    pub async fn toggle_wifi(&self) -> zbus::Result<bool> {
        let new_state = match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    let current = proxy.wireless_enabled().await.unwrap_or(true);
                    let new_state = !current;
                    proxy.set_wireless_enabled(new_state).await?;
                    new_state
                } else {
                    true
                }
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.wifi_enabled = !s.wifi_enabled;
                s.wifi_enabled
            }
        };
        self.event_bus.emit(SystemEvent::WifiToggled(new_state));
        Ok(new_state)
    }

    pub async fn is_wifi_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    return Ok(proxy.wireless_enabled().await.unwrap_or(true));
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).wifi_enabled),
        }
    }

    pub async fn set_wifi_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    proxy.set_wireless_enabled(powered).await?;
                }
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).wifi_enabled = powered;
            }
        }
        self.event_bus.emit(SystemEvent::WifiToggled(powered));
        Ok(())
    }

    pub async fn list_wifi_networks(&self) -> zbus::Result<Vec<WifiNetworkInfo>> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
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
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).wifi_networks.clone()),
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

    pub async fn connect_wifi(&self, ssid: &str, _password: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
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
                                                        let _ = nm_proxy.activate_connection(&conn_path, &zbus::zvariant::ObjectPath::from_str_unchecked("/"), &zbus::zvariant::ObjectPath::from_str_unchecked("/")).await?;
                                                        if let Ok(mut l) = self.active_wifi_ssid.lock() {
                                                            *l = Some(ssid.to_string());
                                                        }
                                                        self.event_bus.emit(SystemEvent::NetworkUpdated(ssid.to_string()));
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
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                for net in &mut s.wifi_networks {
                    net.active = net.ssid == ssid;
                }
                self.event_bus.emit(SystemEvent::NetworkUpdated(ssid.to_string()));
                Ok(())
            }
        }
    }

    pub async fn disconnect_wifi(&self, ssid: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    if let Ok(active_conns) = nm_proxy.active_connections().await {
                        for path in active_conns {
                            if let Ok(ac_proxy) = NmActiveConnectionProxy::builder(system).path(path.clone())?.build().await {
                                if let Ok(id) = ac_proxy.id().await {
                                    if id == ssid {
                                        nm_proxy.deactivate_connection(&path).await?;
                                        if let Ok(mut l) = self.active_wifi_ssid.lock() {
                                            *l = None;
                                        }
                                        self.event_bus.emit(SystemEvent::NetworkUpdated("Disconnected".to_string()));
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                for net in &mut s.wifi_networks {
                    if net.ssid == ssid {
                        net.active = false;
                    }
                }
                self.event_bus.emit(SystemEvent::NetworkUpdated("Disconnected".to_string()));
                Ok(())
            }
        }
    }

    pub async fn delete_wifi(&self, ssid: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
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
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.wifi_networks.retain(|net| net.ssid != ssid);
                Ok(())
            }
        }
    }

    pub async fn modify_wifi(&self, _ssid: &str, _dhcp: bool, _ip: &str, _gw: &str, _dns: &str, _auto: bool) -> zbus::Result<()> {
        Ok(())
    }

    pub async fn get_wifi_details(&self, _ssid: &str) -> zbus::Result<(String, String, String, String, bool)> {
        Ok(("auto".to_string(), "192.168.1.100".to_string(), "192.168.1.1".to_string(), "8.8.8.8".to_string(), true))
    }

    pub async fn refresh_network_status(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(Ok(nm_proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await {
                    if let Ok(active_conns) = nm_proxy.active_connections().await {
                        for path in active_conns {
                            if let Ok(ac_proxy) = NmActiveConnectionProxy::builder(system).path(path)?.build().await {
                                if let Ok(id) = ac_proxy.id().await {
                                    if let Ok(mut l) = self.active_wifi_ssid.lock() {
                                        *l = Some(id);
                                    }
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                if let Ok(mut l) = self.active_wifi_ssid.lock() {
                    *l = None;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }

    pub fn get_cached_network_status(&self) -> (String, String, String) {
        if let ControllerBackend::Mock(state) = &self.backend {
            let s = state.lock().unwrap_or_else(|e| e.into_inner());
            if !s.wifi_enabled {
                return ("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Disattivato".to_string());
            }
            let active_ssid = s.wifi_networks.iter().find(|w| w.active).map(|w| w.ssid.clone()).unwrap_or_else(|| "Non connesso".to_string());
            let icon = if active_ssid == "Non connesso" { "󰖪" } else { "" };
            return (icon.to_string(), "Rete Wi-Fi".to_string(), active_ssid);
        }

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
                            let ssid = self.active_wifi_ssid.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_else(|| "Connesso".to_string());
                            return ("".to_string(), "Rete Wi-Fi".to_string(), ssid);
                        }
                    }
                }
            }
        }
        ("󰖪".to_string(), "Rete Wi-Fi".to_string(), "Non connesso".to_string())
    }
}
