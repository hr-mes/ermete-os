use zbus::Connection;
use std::sync::{Arc, Mutex};

pub use crate::ipc::types::*;

pub fn subscribe_system_events() -> tokio::sync::broadcast::Receiver<SystemEvent> {
    get_event_bus().subscribe_broadcast()
}


// ==========================================
// SPECIALIZED CONTROLLERS (Decoupled Nodes)
// ==========================================

pub use crate::ipc::audio::AudioController;
pub use crate::ipc::network::NetworkController;
pub use crate::ipc::bluetooth::BluetoothController;
pub use crate::ipc::power::PowerController;

pub use crate::ipc::display::DisplayController;
pub use crate::ipc::mpris::MprisController;

// ==========================================
// SYSTEM CONTROLLER FACADE & COMPATIBILITY
// ==========================================

static GLOBAL_AUDIO_CONTROLLER: std::sync::OnceLock<Arc<AudioController>> = std::sync::OnceLock::new();
static GLOBAL_NETWORK_CONTROLLER: std::sync::OnceLock<Arc<NetworkController>> = std::sync::OnceLock::new();
static GLOBAL_BLUETOOTH_CONTROLLER: std::sync::OnceLock<Arc<BluetoothController>> = std::sync::OnceLock::new();
static GLOBAL_DISPLAY_CONTROLLER: std::sync::OnceLock<Arc<DisplayController>> = std::sync::OnceLock::new();
static GLOBAL_POWER_CONTROLLER: std::sync::OnceLock<Arc<PowerController>> = std::sync::OnceLock::new();
static GLOBAL_MPRIS_CONTROLLER: std::sync::OnceLock<Arc<MprisController>> = std::sync::OnceLock::new();
static GLOBAL_EVENT_BUS: std::sync::OnceLock<SystemEventBus> = std::sync::OnceLock::new();

pub fn init_system_controller() {
    glib::MainContext::default().spawn_local(async {
        if let (Ok(session), Ok(system)) = (Connection::session().await, Connection::system().await) {
            let event_bus = SystemEventBus::new();
            let backend = ControllerBackend::Dbus { session, system };
            
            let audio = Arc::new(AudioController::new(backend.clone(), event_bus.clone()));
            let network = Arc::new(NetworkController::new(backend.clone(), event_bus.clone()));
            let bluetooth = Arc::new(BluetoothController::new(backend.clone(), event_bus.clone()));
            let display = Arc::new(DisplayController::new(backend.clone(), event_bus.clone()));
            let power = Arc::new(PowerController::new(backend.clone(), event_bus.clone()));
            let mpris = Arc::new(MprisController::new(backend, event_bus.clone()));

            let _ = mpris.refresh_mpris().await;
            let _ = network.refresh_network_status().await;

            // Start eBPF push notification hooks to bypass DBus polling
            crate::sys::ebpf::start_ebpf_dbus_listener(event_bus.clone()).await;

            let _ = GLOBAL_AUDIO_CONTROLLER.set(audio);
            let _ = GLOBAL_NETWORK_CONTROLLER.set(network);
            let _ = GLOBAL_BLUETOOTH_CONTROLLER.set(bluetooth);
            let _ = GLOBAL_DISPLAY_CONTROLLER.set(display);
            let _ = GLOBAL_POWER_CONTROLLER.set(power);
            let _ = GLOBAL_MPRIS_CONTROLLER.set(mpris);
            let _ = GLOBAL_EVENT_BUS.set(event_bus);
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

pub fn get_event_bus() -> SystemEventBus {
    GLOBAL_EVENT_BUS.get().cloned().unwrap_or_else(SystemEventBus::new)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(audio.get_cached_volume(), 0.75);

        audio.set_source_volume(0.60).await.unwrap();
        display.set_brightness(0.80).await.unwrap();

        mpris.player_command("play-pause").await.unwrap();
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
