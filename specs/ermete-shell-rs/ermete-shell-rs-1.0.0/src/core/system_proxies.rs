use zbus::{proxy, Connection};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NetworkManager {
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_wireless_enabled(&self, val: bool) -> zbus::Result<()>;
    fn get_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    fn activate_connection(
        &self,
        connection: &zbus::zvariant::ObjectPath<'_>,
        device: &zbus::zvariant::ObjectPath<'_>,
        specific_object: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn deactivate_connection(&self, active_connection: &zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub trait NmSettings {
    fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    fn get_connection_by_uuid(&self, uuid: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NmSettingsConnection {
    fn get_settings(&self) -> zbus::Result<HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>>>;
    fn delete(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NmActiveConnection {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NmDevice {
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NmWireless {
    fn get_access_points(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    fn request_scan(&self, options: HashMap<&str, zbus::zvariant::Value<'_>>) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NmAccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
}

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

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto"
)]
pub trait LogindSession {
    fn set_brightness(&self, subsystem: &str, name: &str, value: u32) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_service = "org.mpris.MediaPlayer2.player",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub trait MprisPlayer {
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn play_pause(&self) -> zbus::Result<()>;
    fn play(&self) -> zbus::Result<()>;
    fn pause(&self) -> zbus::Result<()>;
    fn stop(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "os.ermete.Bedrock",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock"
)]
pub trait BedrockAudio {
    #[zbus(property, name = "Volume")]
    fn volume(&self) -> zbus::Result<f64>;
    #[zbus(property, name = "Volume")]
    fn set_volume(&self, val: f64) -> zbus::Result<()>;
    #[zbus(property, name = "Muted")]
    fn muted(&self) -> zbus::Result<bool>;
    #[zbus(property, name = "Muted")]
    fn set_muted(&self, val: bool) -> zbus::Result<()>;
    #[zbus(property, name = "SourceMuted")]
    fn source_muted(&self) -> zbus::Result<bool>;
    #[zbus(property, name = "SourceMuted")]
    fn set_source_muted(&self, val: bool) -> zbus::Result<()>;
    #[zbus(property, name = "SourceVolume")]
    fn source_volume(&self) -> zbus::Result<f64>;
    #[zbus(property, name = "SourceVolume")]
    fn set_source_volume(&self, val: f64) -> zbus::Result<()>;
}

#[proxy(
    interface = "os.ermete.Bedrock.SecretEnroller",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock/SecretEnroller"
)]
pub trait SecretEnroller {
    fn enroll_secret(&self, username: &str, password: &str) -> zbus::Result<String>;
    fn decrypt_secret(&self, username: &str) -> zbus::Result<String>;
    fn unlock_keyring(&self, username: &str, secret: &str) -> zbus::Result<bool>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WifiNetworkInfo {
    pub ssid: String,
    pub signal: i32,
    pub active: bool,
    pub saved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BluetoothDeviceInfo {
    pub name: String,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct MockState {
    pub wifi_enabled: bool,
    pub bt_enabled: bool,
    pub mute: bool,
    pub source_mute: bool,
    pub volume: f64,
    pub source_volume: f64,
    pub brightness: f64,
    pub last_player_command: Option<String>,
    pub wifi_networks: Vec<WifiNetworkInfo>,
    pub bt_devices: Vec<BluetoothDeviceInfo>,
}

#[derive(Clone, Debug)]
pub enum ControllerBackend {
    Dbus {
        session: Connection,
        system: Connection,
    },
    Mock(Arc<Mutex<MockState>>),
}

#[derive(Clone, Debug)]
pub struct SystemController {
    backend: ControllerBackend,
    cached_volume: Arc<Mutex<f64>>,
    cached_mpris: Arc<Mutex<Option<crate::core::mpris::MprisState>>>,
    active_wifi_ssid: Arc<Mutex<Option<String>>>,
}

impl SystemController {
    pub async fn new() -> zbus::Result<Self> {
        let session = Connection::session().await?;
        let system = Connection::system().await?;
        Ok(Self {
            backend: ControllerBackend::Dbus { session, system },
            cached_volume: Arc::new(Mutex::new(0.8)),
            cached_mpris: Arc::new(Mutex::new(None)),
            active_wifi_ssid: Arc::new(Mutex::new(None)),
        })
    }

    pub fn new_mock() -> Self {
        let state = MockState {
            wifi_enabled: true,
            bt_enabled: true,
            mute: false,
            source_mute: false,
            volume: 0.5,
            source_volume: 0.5,
            brightness: 0.5,
            last_player_command: None,
            wifi_networks: vec![
                WifiNetworkInfo {
                    ssid: "Ermete-5G".to_string(),
                    signal: 85,
                    active: true,
                    saved: true,
                },
            ],
            bt_devices: vec![
                BluetoothDeviceInfo {
                    name: "Ermete Headphones".to_string(),
                    connected: true,
                },
            ],
        };
        Self {
            backend: ControllerBackend::Mock(Arc::new(Mutex::new(state))),
            cached_volume: Arc::new(Mutex::new(0.5)),
            cached_mpris: Arc::new(Mutex::new(None)),
            active_wifi_ssid: Arc::new(Mutex::new(Some("Ermete-5G".to_string()))),
        }
    }

    pub async fn toggle_wifi(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    let current = proxy.wireless_enabled().await.unwrap_or(true);
                    let new_state = !current;
                    proxy.set_wireless_enabled(new_state).await?;
                    return Ok(new_state);
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.wifi_enabled = !s.wifi_enabled;
                Ok(s.wifi_enabled)
            }
        }
    }

    pub async fn toggle_bluetooth(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    let current = proxy.powered().await.unwrap_or(false);
                    let new_state = !current;
                    proxy.set_powered(new_state).await?;
                    return Ok(new_state);
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.bt_enabled = !s.bt_enabled;
                Ok(s.bt_enabled)
            }
        }
    }

    pub async fn toggle_mute(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    let current = proxy.muted().await.unwrap_or(false);
                    let new_state = !current;
                    proxy.set_muted(new_state).await?;
                    return Ok(new_state);
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.mute = !s.mute;
                Ok(s.mute)
            }
        }
    }

    pub async fn toggle_source_mute(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    let current = proxy.source_muted().await.unwrap_or(false);
                    let new_state = !current;
                    proxy.set_source_muted(new_state).await?;
                    return Ok(new_state);
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.source_mute = !s.source_mute;
                Ok(s.source_mute)
            }
        }
    }

    pub async fn set_volume(&self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    proxy.set_volume(volume).await?;
                    if let Ok(mut c) = self.cached_volume.lock() {
                        *c = volume;
                    }
                }
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                if let Ok(mut c) = self.cached_volume.lock() {
                    *c = volume;
                }
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.volume = volume;
                Ok(())
            }
        }
    }

    pub async fn set_source_volume(&self, volume: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    proxy.set_source_volume(volume).await?;
                }
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.source_volume = volume;
                Ok(())
            }
        }
    }

    pub async fn set_brightness(&self, brightness: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                let val = (brightness * 100.0) as u32;
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindSessionProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.brightness = brightness;
                Ok(())
            }
        }
    }

    pub async fn player_command(&self, cmd: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(dbus) = zbus::fdo::DBusProxy::new(session).await {
                    if let Ok(names) = dbus.list_names().await {
                        for name in names {
                            if name.as_str().starts_with("org.mpris.MediaPlayer2.") {
                                if let Ok(player) = MprisPlayerProxy::builder(session)
                                    .destination(name.as_str())?
                                    .path("/org/mpris/MediaPlayer2")?
                                    .build().await
                                {
                                    match cmd {
                                        "play-pause" => { let _ = player.play_pause().await; }
                                        "next" => { let _ = player.next().await; }
                                        "previous" => { let _ = player.previous().await; }
                                        "play" => { let _ = player.play().await; }
                                        "pause" => { let _ = player.pause().await; }
                                        _ => {}
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                let _ = self.refresh_mpris().await;
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).last_player_command = Some(cmd.to_string());
                let _ = self.refresh_mpris().await;
                Ok(())
            }
        }
    }

    pub async fn is_wifi_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    return Ok(proxy.wireless_enabled().await.unwrap_or(true));
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).wifi_enabled),
        }
    }

    pub async fn is_bluetooth_enabled(&self) -> zbus::Result<bool> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    return Ok(proxy.powered().await.unwrap_or(true));
                }
                Ok(true)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).bt_enabled),
        }
    }

    pub async fn get_volume(&self) -> zbus::Result<f64> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BedrockAudioProxy::new(session)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    if let Ok(vol) = proxy.volume().await {
                        if let Ok(mut c) = self.cached_volume.lock() {
                            *c = vol;
                        }
                        return Ok(vol);
                    }
                }
                Ok(*self.cached_volume.lock().unwrap_or_else(|e| e.into_inner()))
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).volume),
        }
    }

    pub async fn get_brightness(&self) -> zbus::Result<f64> {
        match &self.backend {
            ControllerBackend::Dbus { .. } => {
                let live = crate::core::live_state::get_live_state();
                Ok(live.brightness / 100.0)
            }
            ControllerBackend::Mock(state) => Ok(state.lock().unwrap_or_else(|e| e.into_inner()).brightness),
        }
    }

    pub fn get_last_player_command(&self) -> Option<String> {
        match &self.backend {
            ControllerBackend::Dbus { .. } => None,
            ControllerBackend::Mock(state) => state.lock().unwrap_or_else(|e| e.into_inner()).last_player_command.clone(),
        }
    }

    pub async fn lock_screen(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    let _ = proxy.suspend(true).await;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }

    pub async fn set_wifi_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    proxy.set_wireless_enabled(powered).await?;
                }
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).wifi_enabled = powered;
                Ok(())
            }
        }
    }

    pub async fn set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), BlueZProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    proxy.set_powered(powered).await?;
                }
                Ok(())
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).bt_enabled = powered;
                Ok(())
            }
        }
    }

    pub async fn list_wifi_networks(&self) -> zbus::Result<Vec<WifiNetworkInfo>> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                let mut results = Vec::new();
                if let Ok(nm_proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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
                if let Ok(settings_proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NmSettingsProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    if let Ok(conns) = settings_proxy.list_connections().await {
                        for conn_path in conns {
                            if let Ok(conn_proxy) = NmSettingsConnectionProxy::builder(system).path(conn_path.clone())?.build().await {
                                if let Ok(settings) = conn_proxy.get_settings().await {
                                    if let Some(wifi_sec) = settings.get("802-11-wireless") {
                                        if let Some(ssid_val) = wifi_sec.get("ssid") {
                                            if let Some(s) = Self::extract_ssid(ssid_val) {
                                                if s == ssid {
                                                    if let Ok(nm_proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                                                        let _ = nm_proxy.activate_connection(&conn_path, &zbus::zvariant::ObjectPath::from_str_unchecked("/"), &zbus::zvariant::ObjectPath::from_str_unchecked("/")).await?;
                                                        if let Ok(mut l) = self.active_wifi_ssid.lock() {
                                                            *l = Some(ssid.to_string());
                                                        }
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
                Ok(())
            }
        }
    }

    pub async fn disconnect_wifi(&self, ssid: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(nm_proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
                    if let Ok(active_conns) = nm_proxy.active_connections().await {
                        for path in active_conns {
                            if let Ok(ac_proxy) = NmActiveConnectionProxy::builder(system).path(path.clone())?.build().await {
                                if let Ok(id) = ac_proxy.id().await {
                                    if id == ssid {
                                        nm_proxy.deactivate_connection(&path).await?;
                                        if let Ok(mut l) = self.active_wifi_ssid.lock() {
                                            *l = None;
                                        }
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
                Ok(())
            }
        }
    }

    pub async fn delete_wifi(&self, ssid: &str) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(settings_proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NmSettingsProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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
        match &self.backend {
            ControllerBackend::Dbus { system: _, .. } => {
                Ok(())
            }
            ControllerBackend::Mock(_) => Ok(()),
        }
    }

    pub async fn get_wifi_details(&self, _ssid: &str) -> zbus::Result<(String, String, String, String, bool)> {
        match &self.backend {
            ControllerBackend::Dbus { system: _, .. } => {
                Ok(("auto".to_string(), "192.168.1.100".to_string(), "192.168.1.1".to_string(), "8.8.8.8".to_string(), true))
            }
            ControllerBackend::Mock(_) => {
                Ok(("auto".to_string(), "192.168.1.100".to_string(), "192.168.1.1".to_string(), "8.8.8.8".to_string(), true))
            }
        }
    }

    pub fn get_cached_volume(&self) -> f64 {
        *self.cached_volume.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get_cached_mpris_state(&self) -> Option<crate::core::mpris::MprisState> {
        self.cached_mpris.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub async fn refresh_mpris(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { session, .. } => {
                if let Ok(dbus_proxy) = zbus::fdo::DBusProxy::new(session).await {
                    if let Ok(names) = dbus_proxy.list_names().await {
                        for name in names {
                            if name.as_str().starts_with("org.mpris.MediaPlayer2.") {
                                if let Ok(props_proxy) = zbus::fdo::PropertiesProxy::builder(session)
                                    .destination(name.as_str())?
                                    .path("/org/mpris/MediaPlayer2")?
                                    .build().await
                                {
                                    let iface: zbus::names::InterfaceName = "org.mpris.MediaPlayer2.Player".try_into().unwrap_or_else(|_| "org.mpris.MediaPlayer2.Player".try_into().unwrap());
                                    let status = props_proxy.get(iface.clone(), "PlaybackStatus").await
                                        .ok()
                                        .and_then(|v| match &*v {
                                            zbus::zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                                            _ => None,
                                        })
                                        .unwrap_or_else(|| "Stopped".to_string());
                                    let title = props_proxy.get(iface.clone(), "Metadata").await
                                        .ok()
                                        .and_then(|v| {
                                            if let zbus::zvariant::Value::Dict(dict) = &*v {
                                                if let Ok(Some(val)) = dict.get(&zbus::zvariant::Value::from("xesam:title")) {
                                                    if let zbus::zvariant::Value::Str(s) = val {
                                                        return Some(s.as_str().to_string());
                                                    }
                                                }
                                            }
                                            None
                                        }).unwrap_or_else(|| "Sconosciuto".to_string());
                                    let artist = props_proxy.get(iface.clone(), "Metadata").await
                                        .ok()
                                        .and_then(|v| {
                                            if let zbus::zvariant::Value::Dict(dict) = &*v {
                                                if let Ok(Some(val)) = dict.get(&zbus::zvariant::Value::from("xesam:artist")) {
                                                    if let zbus::zvariant::Value::Array(arr) = val {
                                                        if let Ok(Some(first)) = arr.get(0) {
                                                            if let zbus::zvariant::Value::Str(s) = first {
                                                                return Some(s.as_str().to_string());
                                                            }
                                                        }
                                                    } else if let zbus::zvariant::Value::Str(s) = val {
                                                        return Some(s.as_str().to_string());
                                                    }
                                                }
                                            }
                                            None
                                        }).unwrap_or_else(|| "-".to_string());
                                    let new_state = crate::core::mpris::MprisState {
                                        title,
                                        artist,
                                        status,
                                    };
                                    if let Ok(mut lock) = self.cached_mpris.lock() {
                                        *lock = Some(new_state);
                                    }
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                if let Ok(mut lock) = self.cached_mpris.lock() {
                    *lock = None;
                }
                Ok(())
            }
            ControllerBackend::Mock(_) => {
                if let Ok(mut lock) = self.cached_mpris.lock() {
                    if lock.is_none() {
                        *lock = Some(crate::core::mpris::MprisState {
                            title: "Track Title".to_string(),
                            artist: "Artist".to_string(),
                            status: "Playing".to_string(),
                        });
                    }
                }
                Ok(())
            }
        }
    }

    pub async fn refresh_network_status(&self) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                if let Ok(nm_proxy) = tokio::time::timeout(std::time::Duration::from_secs(5), NetworkManagerProxy::new(system)).await.map_err(|_| zbus::Error::Failure("timeout".into()))?? {
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

static GLOBAL_CONTROLLER: std::sync::OnceLock<Arc<SystemController>> = std::sync::OnceLock::new();

pub fn init_system_controller() {
    glib::MainContext::default().spawn_local(async {
        if let Ok(controller) = SystemController::new().await {
            let _ = controller.refresh_mpris().await;
            let _ = controller.refresh_network_status().await;
            let _ = GLOBAL_CONTROLLER.set(Arc::new(controller));
        }
    });
}

pub fn get_global_controller() -> Arc<SystemController> {
    GLOBAL_CONTROLLER.get().cloned().unwrap_or_else(|| Arc::new(SystemController::new_mock()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_controller_state_updates() {
        let controller = SystemController::new_mock();
        assert_eq!(controller.is_wifi_enabled().await.unwrap(), true);

        let new_wifi = controller.toggle_wifi().await.unwrap();
        assert_eq!(new_wifi, false);
        assert_eq!(controller.is_wifi_enabled().await.unwrap(), false);

        controller.set_wifi_powered(true).await.unwrap();
        assert_eq!(controller.is_wifi_enabled().await.unwrap(), true);

        let new_bt = controller.toggle_bluetooth().await.unwrap();
        assert_eq!(new_bt, false);
        assert_eq!(controller.is_bluetooth_enabled().await.unwrap(), false);

        controller.set_bluetooth_powered(true).await.unwrap();
        assert_eq!(controller.is_bluetooth_enabled().await.unwrap(), true);

        let new_mute = controller.toggle_mute().await.unwrap();
        assert_eq!(new_mute, true);

        let new_src_mute = controller.toggle_source_mute().await.unwrap();
        assert_eq!(new_src_mute, true);

        controller.set_volume(0.75).await.unwrap();
        assert_eq!(controller.get_volume().await.unwrap(), 0.75);
        assert_eq!(controller.get_cached_volume(), 0.75);

        controller.set_source_volume(0.60).await.unwrap();
        assert_eq!(controller.get_brightness().await.unwrap(), 0.5); // default brightness

        controller.set_brightness(0.80).await.unwrap();
        assert_eq!(controller.get_brightness().await.unwrap(), 0.80);

        controller.player_command("play-pause").await.unwrap();
        assert_eq!(controller.get_last_player_command(), Some("play-pause".to_string()));
    }

    #[tokio::test]
    async fn test_system_controller_ui_network_and_bt_methods() {
        let controller = SystemController::new_mock();

        let wifi_list = controller.list_wifi_networks().await.unwrap();
        assert_eq!(wifi_list.len(), 1);
        assert_eq!(wifi_list[0].ssid, "Ermete-5G");

        assert!(controller.connect_wifi("Ermete-5G", "secret").await.is_ok());
        assert!(controller.disconnect_wifi("Ermete-5G").await.is_ok());
        assert!(controller.delete_wifi("Ermete-5G").await.is_ok());
        assert!(controller.modify_wifi("Ermete-5G", true, "192.168.1.50", "192.168.1.1", "8.8.8.8", true).await.is_ok());

        let details = controller.get_wifi_details("Ermete-5G").await.unwrap();
        assert_eq!(details.0, "auto");
        assert_eq!(details.4, true);

        let bt_list = controller.list_bluetooth_devices().await.unwrap();
        assert_eq!(bt_list.len(), 1);
        assert_eq!(bt_list[0].name, "Ermete Headphones");
    }

    #[tokio::test]
    async fn test_system_controller_power_and_global_methods() {
        let controller = SystemController::new_mock();
        assert!(controller.lock_screen().await.is_ok());
        assert!(controller.power_off().await.is_ok());
        assert!(controller.reboot().await.is_ok());
        assert!(controller.suspend().await.is_ok());

        assert!(controller.get_cached_mpris_state().is_none());
        let (icon, label, sub) = controller.get_cached_network_status();
        assert!(!icon.is_empty() && !label.is_empty() && !sub.is_empty());

        let global = get_global_controller();
        assert_eq!(global.get_cached_volume(), 0.5);
    }

    #[tokio::test]
    async fn test_review_findings_compliance() {
        let controller = SystemController::new_mock();
        
        // Check connect/disconnect updates mock state
        controller.connect_wifi("Ermete-5G", "secret").await.unwrap();
        let list = controller.list_wifi_networks().await.unwrap();
        assert_eq!(list[0].active, true);
        
        // Check get_cached_network_status returns connected SSID instead of hardcoded "Connesso"
        let (icon, title, sub) = controller.get_cached_network_status();
        assert_eq!(icon, "");
        assert_eq!(title, "Rete Wi-Fi");
        assert_eq!(sub, "Ermete-5G");

        controller.disconnect_wifi("Ermete-5G").await.unwrap();
        let list = controller.list_wifi_networks().await.unwrap();
        assert_eq!(list[0].active, false);

        // Check get_cached_mpris_state is populated after player_command
        assert!(controller.get_cached_mpris_state().is_none());
        controller.player_command("play-pause").await.unwrap();
        let mpris = controller.get_cached_mpris_state().expect("cached_mpris should be populated");
        assert_eq!(mpris.status, "Playing");
    }
}
