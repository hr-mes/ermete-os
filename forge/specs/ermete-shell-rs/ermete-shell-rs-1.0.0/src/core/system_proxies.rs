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
pub use crate::core::bluetooth_proxy::BluetoothController;
pub use crate::core::power_proxy::PowerController;

pub use crate::core::display_proxy::DisplayController;
pub use crate::core::mpris_proxy::MprisController;

// ==========================================
// SYSTEM CONTROLLER FACADE & COMPATIBILITY
// ==========================================



static GLOBAL_AUDIO_CONTROLLER: std::sync::OnceLock<Arc<AudioController>> = std::sync::OnceLock::new();
static GLOBAL_NETWORK_CONTROLLER: std::sync::OnceLock<Arc<NetworkController>> = std::sync::OnceLock::new();
static GLOBAL_BLUETOOTH_CONTROLLER: std::sync::OnceLock<Arc<BluetoothController>> = std::sync::OnceLock::new();
static GLOBAL_DISPLAY_CONTROLLER: std::sync::OnceLock<Arc<DisplayController>> = std::sync::OnceLock::new();
static GLOBAL_POWER_CONTROLLER: std::sync::OnceLock<Arc<PowerController>> = std::sync::OnceLock::new();
static GLOBAL_MPRIS_CONTROLLER: std::sync::OnceLock<Arc<MprisController>> = std::sync::OnceLock::new();
static GLOBAL_STATE_STORE: std::sync::OnceLock<Arc<SettingsStateStore>> = std::sync::OnceLock::new();
pub fn init_system_controller() {
    glib::MainContext::default().spawn_local(async {
        if let (Ok(session), Ok(system)) = (Connection::session().await, Connection::system().await) {
            let event_bus = SystemEventBus::new();
            let state_store = Arc::new(SettingsStateStore::new(event_bus.clone()));
            let backend = ControllerBackend::Dbus { session, system };
            
            let audio = Arc::new(AudioController::new(backend.clone(), event_bus.clone()));
            let network = Arc::new(NetworkController::new(backend.clone(), event_bus.clone()));
            let bluetooth = Arc::new(BluetoothController::new(backend.clone(), event_bus.clone()));
            let display = Arc::new(DisplayController::new(backend.clone(), event_bus.clone()));
            let power = Arc::new(PowerController::new(backend.clone(), event_bus.clone()));
            let mpris = Arc::new(MprisController::new(backend, event_bus));

            let _ = mpris.refresh_mpris().await;
            let _ = network.refresh_network_status().await;

            // Start eBPF push notification hooks to bypass DBus polling
            crate::core::ebpf_hooks::start_ebpf_dbus_listener(event_bus.clone()).await;

            let _ = GLOBAL_AUDIO_CONTROLLER.set(audio);
            let _ = GLOBAL_NETWORK_CONTROLLER.set(network);
            let _ = GLOBAL_BLUETOOTH_CONTROLLER.set(bluetooth);
            let _ = GLOBAL_DISPLAY_CONTROLLER.set(display);
            let _ = GLOBAL_POWER_CONTROLLER.set(power);
            let _ = GLOBAL_MPRIS_CONTROLLER.set(mpris);
            let _ = GLOBAL_STATE_STORE.set(state_store);
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

    #[tokio::test]
    async fn test_system_controller_state_updates() {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        let network = Arc::new(NetworkController::new_mock(state.clone(), bus.clone()));
        let bluetooth = Arc::new(BluetoothController::new_mock(state.clone(), bus.clone()));
        let audio = Arc::new(AudioController::new_mock(state.clone(), bus.clone()));
        let display = Arc::new(DisplayController::new_mock(state.clone(), bus.clone()));
        let mpris = Arc::new(MprisController::new_mock(state.clone(), bus.clone()));

        assert_eq!(network.is_wifi_enabled().await.unwrap(), true);

        let new_wifi = network.toggle_wifi().await.unwrap();
        assert_eq!(new_wifi, false);
        assert_eq!(network.is_wifi_enabled().await.unwrap(), false);

        network.set_wifi_powered(true).await.unwrap();
        assert_eq!(network.is_wifi_enabled().await.unwrap(), true);

        let new_bt = bluetooth.toggle_bluetooth().await.unwrap();
        assert_eq!(new_bt, false);
        assert_eq!(bluetooth.is_bluetooth_enabled().await.unwrap(), false);

        bluetooth.set_bluetooth_powered(true).await.unwrap();
        assert_eq!(bluetooth.is_bluetooth_enabled().await.unwrap(), true);

        let new_mute = audio.toggle_mute().await.unwrap();
        assert_eq!(new_mute, true);

        let new_src_mute = audio.toggle_source_mute().await.unwrap();
        assert_eq!(new_src_mute, true);

        audio.set_volume(0.75).await.unwrap();
        assert_eq!(audio.get_volume().await.unwrap(), 0.75);
        assert_eq!(audio.get_cached_volume(), 0.75);

        audio.set_source_volume(0.60).await.unwrap();
        assert_eq!(display.get_brightness().await.unwrap(), 0.5);

        display.set_brightness(0.80).await.unwrap();
        assert_eq!(display.get_brightness().await.unwrap(), 0.80);

        mpris.player_command("play-pause").await.unwrap();
        assert_eq!(mpris.get_last_player_command(), Some("play-pause".to_string()));
    }

    #[tokio::test]
    async fn test_system_controller_ui_network_and_bt_methods() {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        let network = Arc::new(NetworkController::new_mock(state.clone(), bus.clone()));
        let bluetooth = Arc::new(BluetoothController::new_mock(state.clone(), bus.clone()));

        let wifi_list = network.list_wifi_networks().await.unwrap();
        assert_eq!(wifi_list.len(), 1);
        assert_eq!(wifi_list[0].ssid, "Ermete-5G");

        assert!(network.connect_wifi("Ermete-5G", "secret").await.is_ok());
        assert!(network.disconnect_wifi("Ermete-5G").await.is_ok());
        assert!(network.delete_wifi("Ermete-5G").await.is_ok());
        assert!(network.modify_wifi("Ermete-5G", true, "192.168.1.50", "192.168.1.1", "8.8.8.8", true).await.is_ok());

        let details = network.get_wifi_details("Ermete-5G").await.unwrap();
        assert_eq!(details.0, "auto");
        assert_eq!(details.4, true);

        let bt_list = bluetooth.list_bluetooth_devices().await.unwrap();
        assert_eq!(bt_list.len(), 1);
        assert_eq!(bt_list[0].name, "Ermete Headphones");
    }

    #[tokio::test]
    async fn test_system_controller_power_and_global_methods() {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        let power = Arc::new(PowerController::new_mock(state.clone(), bus.clone()));
        let mpris = Arc::new(MprisController::new_mock(state.clone(), bus.clone()));
        let network = Arc::new(NetworkController::new_mock(state.clone(), bus.clone()));

        assert!(power.lock_screen().await.is_ok());
        assert!(power.power_off().await.is_ok());
        assert!(power.reboot().await.is_ok());
        assert!(power.suspend().await.is_ok());

        assert!(mpris.get_cached_mpris_state().is_none());
        let (icon, label, sub) = network.get_cached_network_status();
        assert!(!icon.is_empty() && !label.is_empty() && !sub.is_empty());
    }

    #[tokio::test]
    async fn test_review_findings_compliance() {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        let network = Arc::new(NetworkController::new_mock(state.clone(), bus.clone()));
        let mpris = Arc::new(MprisController::new_mock(state.clone(), bus.clone()));
        
        network.connect_wifi("Ermete-5G", "secret").await.unwrap();
        let list = network.list_wifi_networks().await.unwrap();
        assert_eq!(list[0].active, true);
        
        let (icon, title, sub) = network.get_cached_network_status();
        assert_eq!(icon, "");
        assert_eq!(title, "Rete Wi-Fi");
        assert_eq!(sub, "Ermete-5G");

        network.disconnect_wifi("Ermete-5G").await.unwrap();
        let list = network.list_wifi_networks().await.unwrap();
        assert_eq!(list[0].active, false);

        assert!(mpris.get_cached_mpris_state().is_none());
        mpris.player_command("play-pause").await.unwrap();
        let mpris_state = mpris.get_cached_mpris_state().expect("cached_mpris should be populated");
        assert_eq!(mpris_state.status, "Playing");
    }
}
