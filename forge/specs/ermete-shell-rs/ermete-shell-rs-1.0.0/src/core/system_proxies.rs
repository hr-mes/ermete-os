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

#[derive(Debug, Clone)]
pub enum SystemEvent {
    VolumeChanged(f64),
    MuteToggled(bool),
    WifiToggled(bool),
    BluetoothToggled(bool),
    BrightnessChanged(f64),
    MprisUpdated(Option<crate::core::mpris::MprisState>),
    NetworkUpdated(String),
}

type EventListener = Box<dyn Fn(&SystemEvent) + Send + Sync>;

#[derive(Clone)]
pub struct SystemEventBus {
    listeners: Arc<Mutex<Vec<EventListener>>>,
}

impl Default for SystemEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemEventBus {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe<F>(&self, listener: F)
    where
        F: Fn(&SystemEvent) + Send + Sync + 'static,
    {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.push(Box::new(listener));
        }
    }

    pub fn emit(&self, event: SystemEvent) {
        if let Ok(listeners) = self.listeners.lock() {
            for listener in listeners.iter() {
                listener(&event);
            }
        }
    }
}

impl std::fmt::Debug for SystemEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemEventBus").finish()
    }
}

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub mute: bool,
    pub source_mute: bool,
    pub volume: f64,
    pub source_volume: f64,
    pub brightness: f64,
    pub active_wifi_ssid: Option<String>,
    pub mpris_state: Option<crate::core::mpris::MprisState>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            bluetooth_enabled: true,
            mute: false,
            source_mute: false,
            volume: 0.5,
            source_volume: 0.5,
            brightness: 0.5,
            active_wifi_ssid: Some("Ermete-5G".to_string()),
            mpris_state: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SettingsStateStore {
    state: Arc<Mutex<SettingsState>>,
    event_bus: SystemEventBus,
}

impl SettingsStateStore {
    pub fn new(event_bus: SystemEventBus) -> Self {
        Self {
            state: Arc::new(Mutex::new(SettingsState::default())),
            event_bus,
        }
    }

    pub fn get_state(&self) -> SettingsState {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut SettingsState),
    {
        if let Ok(mut lock) = self.state.lock() {
            f(&mut lock);
        }
    }

    pub fn event_bus(&self) -> &SystemEventBus {
        &self.event_bus
    }
}

impl MockState {
    pub fn default_mock() -> Self {
        Self {
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
        }
    }
}

// ==========================================
// SPECIALIZED CONTROLLERS (Decoupled Nodes)
// ==========================================

pub use crate::core::audio_proxy::AudioController;
pub use crate::core::network_proxy::NetworkController;

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

#[derive(Clone, Debug)]
pub struct DisplayController {
    backend: ControllerBackend,
    event_bus: SystemEventBus,
}

impl DisplayController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self { backend, event_bus }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            event_bus,
        }
    }

    pub async fn set_brightness(&self, brightness: f64) -> zbus::Result<()> {
        match &self.backend {
            ControllerBackend::Dbus { system, .. } => {
                let val = (brightness * 100.0) as u32;
                if let Ok(Ok(proxy)) = tokio::time::timeout(std::time::Duration::from_secs(5), LogindSessionProxy::new(system)).await {
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
            }
            ControllerBackend::Mock(state) => {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.brightness = brightness;
            }
        }
        self.event_bus.emit(SystemEvent::BrightnessChanged(brightness));
        Ok(())
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

#[derive(Clone, Debug)]
pub struct MprisController {
    backend: ControllerBackend,
    cached_mpris: Arc<Mutex<Option<crate::core::mpris::MprisState>>>,
    event_bus: SystemEventBus,
}

impl MprisController {
    pub fn new(backend: ControllerBackend, event_bus: SystemEventBus) -> Self {
        Self {
            backend,
            cached_mpris: Arc::new(Mutex::new(None)),
            event_bus,
        }
    }

    pub fn new_mock(state: Arc<Mutex<MockState>>, event_bus: SystemEventBus) -> Self {
        Self {
            backend: ControllerBackend::Mock(state),
            cached_mpris: Arc::new(Mutex::new(None)),
            event_bus,
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
            }
            ControllerBackend::Mock(state) => {
                state.lock().unwrap_or_else(|e| e.into_inner()).last_player_command = Some(cmd.to_string());
                let _ = self.refresh_mpris().await;
            }
        }
        self.event_bus.emit(SystemEvent::MprisUpdated(self.get_cached_mpris_state()));
        Ok(())
    }

    pub fn get_cached_mpris_state(&self) -> Option<crate::core::mpris::MprisState> {
        self.cached_mpris.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_last_player_command(&self) -> Option<String> {
        match &self.backend {
            ControllerBackend::Dbus { .. } => None,
            ControllerBackend::Mock(state) => state.lock().unwrap_or_else(|e| e.into_inner()).last_player_command.clone(),
        }
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
}

// ==========================================
// SYSTEM CONTROLLER FACADE & COMPATIBILITY
// ==========================================

#[derive(Clone, Debug)]
pub struct SystemController {
    pub audio: Arc<AudioController>,
    pub network: Arc<NetworkController>,
    pub bluetooth: Arc<BluetoothController>,
    pub display: Arc<DisplayController>,
    pub power: Arc<PowerController>,
    pub mpris: Arc<MprisController>,
    pub state_store: Arc<SettingsStateStore>,
}

impl SystemController {
    pub async fn new() -> zbus::Result<Self> {
        let session = Connection::session().await?;
        let system = Connection::system().await?;
        let event_bus = SystemEventBus::new();
        let state_store = Arc::new(SettingsStateStore::new(event_bus.clone()));
        
        let backend = ControllerBackend::Dbus { session, system };
        let audio = Arc::new(AudioController::new(backend.clone(), event_bus.clone()));
        let network = Arc::new(NetworkController::new(backend.clone(), event_bus.clone()));
        let bluetooth = Arc::new(BluetoothController::new(backend.clone(), event_bus.clone()));
        let display = Arc::new(DisplayController::new(backend.clone(), event_bus.clone()));
        let power = Arc::new(PowerController::new(backend.clone(), event_bus.clone()));
        let mpris = Arc::new(MprisController::new(backend, event_bus));

        Ok(Self {
            audio,
            network,
            bluetooth,
            display,
            power,
            mpris,
            state_store,
        })
    }

    pub fn new_mock() -> Self {
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        let event_bus = SystemEventBus::new();
        let state_store = Arc::new(SettingsStateStore::new(event_bus.clone()));

        let audio = Arc::new(AudioController::new_mock(state.clone(), event_bus.clone()));
        let network = Arc::new(NetworkController::new_mock(state.clone(), event_bus.clone()));
        let bluetooth = Arc::new(BluetoothController::new_mock(state.clone(), event_bus.clone()));
        let display = Arc::new(DisplayController::new_mock(state.clone(), event_bus.clone()));
        let power = Arc::new(PowerController::new_mock(state.clone(), event_bus.clone()));
        let mpris = Arc::new(MprisController::new_mock(state, event_bus));

        Self {
            audio,
            network,
            bluetooth,
            display,
            power,
            mpris,
            state_store,
        }
    }

    // Forwarded methods for backward compatibility
    pub async fn toggle_wifi(&self) -> zbus::Result<bool> { self.network.toggle_wifi().await }
    pub async fn toggle_bluetooth(&self) -> zbus::Result<bool> { self.bluetooth.toggle_bluetooth().await }
    pub async fn toggle_mute(&self) -> zbus::Result<bool> { self.audio.toggle_mute().await }
    pub async fn toggle_source_mute(&self) -> zbus::Result<bool> { self.audio.toggle_source_mute().await }
    pub async fn set_volume(&self, volume: f64) -> zbus::Result<()> { self.audio.set_volume(volume).await }
    pub async fn set_source_volume(&self, volume: f64) -> zbus::Result<()> { self.audio.set_source_volume(volume).await }
    pub async fn set_brightness(&self, brightness: f64) -> zbus::Result<()> { self.display.set_brightness(brightness).await }
    pub async fn player_command(&self, cmd: &str) -> zbus::Result<()> { self.mpris.player_command(cmd).await }
    pub async fn is_wifi_enabled(&self) -> zbus::Result<bool> { self.network.is_wifi_enabled().await }
    pub async fn is_bluetooth_enabled(&self) -> zbus::Result<bool> { self.bluetooth.is_bluetooth_enabled().await }
    pub async fn get_volume(&self) -> zbus::Result<f64> { self.audio.get_volume().await }
    pub async fn get_brightness(&self) -> zbus::Result<f64> { self.display.get_brightness().await }
    pub fn get_last_player_command(&self) -> Option<String> { self.mpris.get_last_player_command() }
    pub async fn lock_screen(&self) -> zbus::Result<()> { self.power.lock_screen().await }
    pub async fn power_off(&self) -> zbus::Result<()> { self.power.power_off().await }
    pub async fn reboot(&self) -> zbus::Result<()> { self.power.reboot().await }
    pub async fn suspend(&self) -> zbus::Result<()> { self.power.suspend().await }
    pub async fn set_wifi_powered(&self, powered: bool) -> zbus::Result<()> { self.network.set_wifi_powered(powered).await }
    pub async fn set_bluetooth_powered(&self, powered: bool) -> zbus::Result<()> { self.bluetooth.set_bluetooth_powered(powered).await }
    pub async fn list_wifi_networks(&self) -> zbus::Result<Vec<WifiNetworkInfo>> { self.network.list_wifi_networks().await }
    pub async fn list_bluetooth_devices(&self) -> zbus::Result<Vec<BluetoothDeviceInfo>> { self.bluetooth.list_bluetooth_devices().await }
    pub async fn connect_wifi(&self, ssid: &str, password: &str) -> zbus::Result<()> { self.network.connect_wifi(ssid, password).await }
    pub async fn disconnect_wifi(&self, ssid: &str) -> zbus::Result<()> { self.network.disconnect_wifi(ssid).await }
    pub async fn delete_wifi(&self, ssid: &str) -> zbus::Result<()> { self.network.delete_wifi(ssid).await }
    pub async fn modify_wifi(&self, ssid: &str, dhcp: bool, ip: &str, gw: &str, dns: &str, auto: bool) -> zbus::Result<()> { self.network.modify_wifi(ssid, dhcp, ip, gw, dns, auto).await }
    pub async fn get_wifi_details(&self, ssid: &str) -> zbus::Result<(String, String, String, String, bool)> { self.network.get_wifi_details(ssid).await }
    pub fn get_cached_volume(&self) -> f64 { self.audio.get_cached_volume() }
    pub fn get_cached_mpris_state(&self) -> Option<crate::core::mpris::MprisState> { self.mpris.get_cached_mpris_state() }
    pub async fn refresh_mpris(&self) -> zbus::Result<()> { self.mpris.refresh_mpris().await }
    pub async fn refresh_network_status(&self) -> zbus::Result<()> { self.network.refresh_network_status().await }
    pub fn get_cached_network_status(&self) -> (String, String, String) { self.network.get_cached_network_status() }
}

static GLOBAL_AUDIO_CONTROLLER: std::sync::OnceLock<Arc<AudioController>> = std::sync::OnceLock::new();
static GLOBAL_NETWORK_CONTROLLER: std::sync::OnceLock<Arc<NetworkController>> = std::sync::OnceLock::new();
static GLOBAL_BLUETOOTH_CONTROLLER: std::sync::OnceLock<Arc<BluetoothController>> = std::sync::OnceLock::new();
static GLOBAL_DISPLAY_CONTROLLER: std::sync::OnceLock<Arc<DisplayController>> = std::sync::OnceLock::new();
static GLOBAL_POWER_CONTROLLER: std::sync::OnceLock<Arc<PowerController>> = std::sync::OnceLock::new();
static GLOBAL_MPRIS_CONTROLLER: std::sync::OnceLock<Arc<MprisController>> = std::sync::OnceLock::new();
static GLOBAL_STATE_STORE: std::sync::OnceLock<Arc<SettingsStateStore>> = std::sync::OnceLock::new();
static GLOBAL_CONTROLLER: std::sync::OnceLock<Arc<SystemController>> = std::sync::OnceLock::new();

pub fn init_system_controller() {
    glib::MainContext::default().spawn_local(async {
        if let Ok(controller) = SystemController::new().await {
            let _ = controller.mpris.refresh_mpris().await;
            let _ = controller.network.refresh_network_status().await;
            let _ = GLOBAL_AUDIO_CONTROLLER.set(controller.audio.clone());
            let _ = GLOBAL_NETWORK_CONTROLLER.set(controller.network.clone());
            let _ = GLOBAL_BLUETOOTH_CONTROLLER.set(controller.bluetooth.clone());
            let _ = GLOBAL_DISPLAY_CONTROLLER.set(controller.display.clone());
            let _ = GLOBAL_POWER_CONTROLLER.set(controller.power.clone());
            let _ = GLOBAL_MPRIS_CONTROLLER.set(controller.mpris.clone());
            let _ = GLOBAL_STATE_STORE.set(controller.state_store.clone());
            let _ = GLOBAL_CONTROLLER.set(Arc::new(controller));
        }
    });
}

pub fn get_audio_controller() -> Arc<AudioController> {
    GLOBAL_AUDIO_CONTROLLER.get().cloned().unwrap_or_else(|| {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        Arc::new(AudioController::new_mock(state, bus))
    })
}

pub fn get_network_controller() -> Arc<NetworkController> {
    GLOBAL_NETWORK_CONTROLLER.get().cloned().unwrap_or_else(|| {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        Arc::new(NetworkController::new_mock(state, bus))
    })
}

pub fn get_bluetooth_controller() -> Arc<BluetoothController> {
    GLOBAL_BLUETOOTH_CONTROLLER.get().cloned().unwrap_or_else(|| {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        Arc::new(BluetoothController::new_mock(state, bus))
    })
}

pub fn get_display_controller() -> Arc<DisplayController> {
    GLOBAL_DISPLAY_CONTROLLER.get().cloned().unwrap_or_else(|| {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        Arc::new(DisplayController::new_mock(state, bus))
    })
}

pub fn get_power_controller() -> Arc<PowerController> {
    GLOBAL_POWER_CONTROLLER.get().cloned().unwrap_or_else(|| {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        Arc::new(PowerController::new_mock(state, bus))
    })
}

pub fn get_mpris_controller() -> Arc<MprisController> {
    GLOBAL_MPRIS_CONTROLLER.get().cloned().unwrap_or_else(|| {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        Arc::new(MprisController::new_mock(state, bus))
    })
}

pub fn get_state_store() -> Arc<SettingsStateStore> {
    GLOBAL_STATE_STORE.get().cloned().unwrap_or_else(|| {
        Arc::new(SettingsStateStore::new(SystemEventBus::new()))
    })
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
