pub mod types;
pub mod system_proxies;
pub mod audio;
pub mod bluetooth;
pub mod network;
pub mod power;
pub mod display;
pub mod mpris;
pub mod notifications;
pub mod voiceover;

pub use audio::{get_audio_controller, AudioController};
pub use bluetooth::{get_bluetooth_controller, BluetoothController};
pub use display::{get_display_controller, DisplayController};
pub use mpris::{get_mpris_controller, MprisController};
pub use network::{get_network_controller, NetworkController};
pub use power::{get_power_controller, PowerController};

use zbus::Connection;
use system_proxies::ControllerBackend;

pub fn init_system_controller() {
    glib::MainContext::default().spawn_local(async {
        if let (Ok(session), Ok(system)) = (Connection::session().await, Connection::system().await) {
            let event_bus = system_proxies::get_event_bus();
            let backend = types::IpcBackend::Dbus { session, system };
            
            let audio: Box<dyn ControllerBackend> = Box::new(AudioController::new(backend.clone(), event_bus.clone()));
            let network_ctrl = NetworkController::new(backend.clone(), event_bus.clone());
            let bluetooth: Box<dyn ControllerBackend> = Box::new(BluetoothController::new(backend.clone(), event_bus.clone()));
            let display: Box<dyn ControllerBackend> = Box::new(DisplayController::new(backend.clone(), event_bus.clone()));
            let power: Box<dyn ControllerBackend> = Box::new(PowerController::new(backend.clone(), event_bus.clone()));
            let mpris_ctrl = MprisController::new(backend, event_bus.clone());

            let _ = mpris_ctrl.refresh_mpris().await;
            let _ = network_ctrl.refresh_network_status().await;

            let network: Box<dyn ControllerBackend> = Box::new(network_ctrl);
            let mpris: Box<dyn ControllerBackend> = Box::new(mpris_ctrl);

            // Start eBPF push notification hooks to bypass DBus polling
            crate::sys::ebpf::start_ebpf_dbus_listener(event_bus).await;

            let controllers = vec![audio, network, bluetooth, display, power, mpris];
            system_proxies::init_system_controller(controllers);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::ipc::types::{MockState, SystemEventBus};

    #[tokio::test]
    async fn test_system_controller_state_updates() {
        let bus = SystemEventBus::new();
        let state = Arc::new(Mutex::new(MockState::default_mock()));
        let network = NetworkController::new_mock(state.clone(), bus.clone());
        let bluetooth = BluetoothController::new_mock(state.clone(), bus.clone());
        let audio = AudioController::new_mock(state.clone(), bus.clone());
        let display = DisplayController::new_mock(state.clone(), bus.clone());
        let mpris = MprisController::new_mock(state.clone(), bus.clone());

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
        let network = NetworkController::new_mock(state.clone(), bus.clone());
        let bluetooth = BluetoothController::new_mock(state.clone(), bus.clone());

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
        let power = PowerController::new_mock(state.clone(), bus.clone());
        let mpris = MprisController::new_mock(state.clone(), bus.clone());
        let network = NetworkController::new_mock(state.clone(), bus.clone());

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
        let network = NetworkController::new_mock(state.clone(), bus.clone());
        let mpris = MprisController::new_mock(state.clone(), bus.clone());
        
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
