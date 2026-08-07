use std::process::Command;
use gtk4::gio::prelude::SettingsExt;

#[derive(Debug, Clone)]
pub struct ControlCenterState {
    pub network_icon: String,
    pub network_title: String,
    pub network_sub: String,
    pub is_network_connected: bool,
    pub is_bluetooth_active: bool,
    pub bluetooth_sub: String,
    pub brightness: f64,
    pub volume: f64,
    pub true_tone: bool,
    pub focus_time: bool,
    pub mpris_title: String,
    pub mpris_artist: String,
    pub is_playing: bool,
}

impl Default for ControlCenterState {
    fn default() -> Self {
        Self {
            network_icon: "󰤨".to_string(),
            network_title: "Wi-Fi".to_string(),
            network_sub: "Disattivato".to_string(),
            is_network_connected: false,
            is_bluetooth_active: false,
            bluetooth_sub: "Disattivato".to_string(),
            brightness: 75.0,
            volume: 80.0,
            true_tone: false,
            focus_time: false,
            mpris_title: "Nessun media in riproduzione".to_string(),
            mpris_artist: "-".to_string(),
            is_playing: false,
        }
    }
}

pub enum ControlCenterIntent {
    ToggleWifi(bool),
    ToggleBluetooth(bool),
    SetBrightness(f64),
    SetVolume(f64),
    ToggleTrueTone(bool),
    ToggleFocusTime(bool),
    MediaPrevious,
    MediaPlayPause,
    MediaNext,
    SetDarkMode,
    TriggerStandby,
    LaunchTerminal,
    LaunchSettings(String),
    TriggerScreenshot,
    TriggerLock,
    OptimizeKernel,
}

pub struct ControlCenterViewModel;

impl ControlCenterViewModel {
    pub fn get_initial_state() -> ControlCenterState {
        let (net_icon, net_title, net_sub) = crate::core::get_network_controller().get_cached_network_status();
        let is_network_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
        
        let init_brightness = crate::core::live_state::get_live_state().brightness;
        let brightness = if init_brightness > 0.0 { init_brightness } else { 75.0 };
        
        let init_volume = crate::core::get_audio_controller().get_cached_volume() * 100.0;
        let volume = if init_volume > 0.0 { init_volume } else { 80.0 };

        let focus_time = crate::ipc::notifications::DND_ACTIVE.load(std::sync::atomic::Ordering::SeqCst);
        
        let initial_mpris = crate::core::get_mpris_controller().get_cached_mpris_state();
        let (mpris_title, mpris_artist, is_playing) = match initial_mpris {
            Some(m) => (m.title, m.artist, m.status.contains("Playing")),
            None => ("Nessun media in riproduzione".to_string(), "-".to_string(), false),
        };

        ControlCenterState {
            network_icon: net_icon,
            network_title: net_title,
            network_sub: net_sub,
            is_network_connected,
            is_bluetooth_active: false,
            bluetooth_sub: "Dispositivi".to_string(),
            brightness,
            volume,
            true_tone: false,
            focus_time,
            mpris_title,
            mpris_artist,
            is_playing,
        }
    }

    pub fn subscribe_network<F: Fn(String, String, String, bool, Option<bool>) + 'static>(on_update: F) {
        let mut net_rx = crate::ipc::system_proxies::get_net_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(event) = net_rx.recv().await {
                match event {
                    crate::ipc::types::NetEvent::NetworkUpdated(_) | crate::ipc::types::NetEvent::WifiToggled(_) => {
                        let (net_icon, net_title, net_sub) = crate::core::get_network_controller().get_cached_network_status();
                        let net_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
                        on_update(net_icon, net_title, net_sub, net_connected, None);
                    }
                    crate::ipc::types::NetEvent::BluetoothToggled(enabled) => {
                        let (net_icon, net_title, net_sub) = crate::core::get_network_controller().get_cached_network_status();
                        let net_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
                        on_update(net_icon, net_title, net_sub, net_connected, Some(enabled));
                    }
                }
            }
        });
    }

    pub fn subscribe_hardware<F: Fn(f64) + 'static>(on_brightness_change: F) {
        let mut hw_rx = crate::ipc::system_proxies::get_hardware_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(event) = hw_rx.recv().await {
                if let crate::ipc::types::HardwareEvent::BrightnessChanged(val) = event {
                    on_brightness_change(val);
                }
            }
        });
    }

    pub fn subscribe_audio<F: Fn(f64) + 'static>(on_volume_change: F) {
        let mut audio_rx = crate::ipc::system_proxies::get_audio_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(event) = audio_rx.recv().await {
                if let crate::ipc::types::AudioEvent::VolumeChanged(val) = event {
                    on_volume_change(val);
                }
            }
        });
    }

    pub fn subscribe_mpris<F: Fn(String, String, bool) + 'static>(on_mpris_change: F) {
        let mut mpris_rx = crate::ipc::system_proxies::get_mpris_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(event) = mpris_rx.recv().await {
                let crate::ipc::types::MprisEvent::MprisUpdated(mpris_opt) = event;
                if let Some(mpris) = mpris_opt {
                    on_mpris_change(mpris.title, mpris.artist, mpris.status.contains("Playing"));
                } else {
                    on_mpris_change("Nessun media in riproduzione".to_string(), "-".to_string(), false);
                }
            }
        });
    }

    pub fn execute_intent(intent: ControlCenterIntent) {
        match intent {
            ControlCenterIntent::ToggleWifi(state) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let _ = ctrl.set_wifi_powered(state).await;
                });
            }
            ControlCenterIntent::ToggleBluetooth(state) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_bluetooth_controller();
                    let _ = ctrl.set_bluetooth_powered(state).await;
                });
            }
            ControlCenterIntent::SetBrightness(val) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_display_controller();
                    let _ = ctrl.set_brightness(val).await;
                });
            }
            ControlCenterIntent::SetVolume(val) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_audio_controller();
                    let _ = ctrl.set_volume(val).await;
                });
            }
            ControlCenterIntent::ToggleTrueTone(enabled) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    if let Ok(connection) = zbus::Connection::session().await {
                        let _ = connection.call_method(
                            Some("org.ermete.Settings"),
                            "/org/ermete/Settings",
                            Some("org.freedesktop.DBus.Properties"),
                            "Set",
                            &("org.ermete.Settings", "TrueToneEnabled", zbus::zvariant::Value::from(enabled))
                        ).await;
                    }
                });
            }
            ControlCenterIntent::ToggleFocusTime(enabled) => {
                crate::ipc::notifications::DND_ACTIVE.store(enabled, std::sync::atomic::Ordering::SeqCst);
            }
            ControlCenterIntent::MediaPrevious => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_mpris_controller();
                    let _ = ctrl.player_command("previous").await;
                });
            }
            ControlCenterIntent::MediaPlayPause => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_mpris_controller();
                    let _ = ctrl.player_command("play-pause").await;
                });
            }
            ControlCenterIntent::MediaNext => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_mpris_controller();
                    let _ = ctrl.player_command("next").await;
                });
            }
            ControlCenterIntent::SetDarkMode => {
                let settings = gtk4::gio::Settings::new("org.gnome.desktop.interface");
                let _ = settings.set_string("color-scheme", "prefer-dark");
            }
            ControlCenterIntent::TriggerStandby => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    ermete_niri_ipc::async_client::power_off_monitors().await;
                    let ctrl = crate::core::get_power_controller();
                    let _ = ctrl.suspend().await;
                });
            }
            ControlCenterIntent::LaunchTerminal => {
                let _ = Command::new("foot").spawn();
            }
            ControlCenterIntent::LaunchSettings(page) => {
                let cmd = if page.is_empty() {
                    "ermete-settings-rs".to_string()
                } else {
                    format!("ermete-settings-rs --page {}", page)
                };
                let _ = gtk4::glib::spawn_command_line_async(cmd);
            }
            ControlCenterIntent::TriggerScreenshot => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    ermete_niri_ipc::async_client::screenshot().await;
                });
            }
            ControlCenterIntent::TriggerLock => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_power_controller();
                    let _ = ctrl.lock_screen().await;
                });
            }
            ControlCenterIntent::OptimizeKernel => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let conn = match zbus::Connection::session().await {
                        Ok(c) => Ok(c),
                        Err(_) => zbus::Connection::system().await,
                    };
                    if let Ok(connection) = conn {
                        let res = connection.call_method(
                            Some("org.ermete.KernelForge"),
                            "/org/ermete/KernelForge",
                            Some("org.ermete.KernelForge"),
                            "ForgeHardwareTailoredKernel",
                            &(),
                        ).await;
                        match res {
                            Ok(msg) => {
                                let body_str: Result<String, _> = msg.body().deserialize();
                                match body_str {
                                    Ok(text) => tracing::info!("Kernel Forge successfully completed: {}", text),
                                    Err(_) => tracing::info!("Kernel Forge successfully triggered"),
                                }
                            }
                            Err(e) => tracing::error!("Kernel Forge D-Bus call failed: {}", e),
                        }
                    }
                });
            }
        }
    }
}

