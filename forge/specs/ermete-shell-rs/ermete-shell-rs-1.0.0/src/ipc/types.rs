#![allow(dead_code)]
use zbus::proxy;
use zbus::Connection;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MprisState {
    pub title: String,
    pub artist: String,
    pub status: String,
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

#[derive(Clone, Debug)]
pub enum IpcBackend {
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
    MprisUpdated(Option<MprisState>),
    NetworkUpdated(String),
}

type EventListener = Box<dyn Fn(&SystemEvent) + Send + Sync>;

#[derive(Clone)]
pub struct SystemEventBus {
    listeners: Arc<Mutex<Vec<EventListener>>>,
    sender: tokio::sync::broadcast::Sender<SystemEvent>,
}

impl Default for SystemEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemEventBus {
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(128);
        Self {
            listeners: Arc::new(Mutex::new(Vec::new())),
            sender,
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

    pub fn subscribe_broadcast(&self) -> tokio::sync::broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }

    pub fn emit(&self, event: SystemEvent) {
        let _ = self.sender.send(event.clone());
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
