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

use system_proxies::ControllerBackend;

#[tracing::instrument]
pub fn init_system_controller() {
    tracing::info!("Initializing IPC system controllers and event bus listeners");
    glib::MainContext::default().spawn_local(async {
        let session_res = match zbus::connection::Builder::session() {
            Ok(b) => b.max_queued(1024).build().await,
            Err(e) => Err(e),
        };
        let system_res = match zbus::connection::Builder::system() {
            Ok(b) => b.max_queued(1024).build().await,
            Err(e) => Err(e),
        };

        if let (Ok(session), Ok(system)) = (session_res, system_res) {
            tracing::info!("Connected to Session and System D-Bus buses cleanly with max_queued limits");
            
            let backend = types::IpcBackend::Dbus { session: session.clone(), system };
            
            let appearance_engine = std::sync::Arc::new(crate::appearance_engine::AppearanceEngine::new());
            if let Err(err) = crate::appearance_engine::register_layout_dbus(&session, appearance_engine).await {
                tracing::warn!(error = %err, "Failed registering org.ermete.Shell.Layout DBus interface");
            }
            
            let audio: Box<dyn ControllerBackend> = Box::new(AudioController::new(backend.clone(), system_proxies::get_audio_bus()));
            let network_ctrl = NetworkController::new(backend.clone(), system_proxies::get_net_bus());
            let bluetooth: Box<dyn ControllerBackend> = Box::new(BluetoothController::new(backend.clone(), system_proxies::get_net_bus()));
            let display: Box<dyn ControllerBackend> = Box::new(DisplayController::new(backend.clone(), system_proxies::get_hardware_bus()));
            let power: Box<dyn ControllerBackend> = Box::new(PowerController::new(backend.clone(), system_proxies::get_hardware_bus()));
            let mpris_ctrl = MprisController::new(backend, system_proxies::get_mpris_bus());

            let _ = mpris_ctrl.refresh_mpris().await;
            let _ = network_ctrl.refresh_network_status().await;

            let network: Box<dyn ControllerBackend> = Box::new(network_ctrl);
            let mpris: Box<dyn ControllerBackend> = Box::new(mpris_ctrl);

            // Start eBPF push notification hooks to bypass DBus polling
            crate::sys::ebpf::start_ebpf_dbus_listener(system_proxies::get_net_bus()).await;

            let controllers = vec![audio, network, bluetooth, display, power, mpris];
            system_proxies::init_system_controller(controllers);
            tracing::info!("All IPC controllers successfully initialized and registered");
        } else {
            tracing::warn!("Failed to establish Session or System D-Bus connections for IPC controllers");
        }
    });
}

