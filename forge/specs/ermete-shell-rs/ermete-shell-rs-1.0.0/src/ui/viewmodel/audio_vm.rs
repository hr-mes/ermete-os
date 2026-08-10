use zbus::proxy;

#[proxy(
    interface = "org.ermete.AudioBus",
    default_service = "org.ermete.AudioBus",
    default_path = "/org/ermete/AudioBus"
)]
pub trait AudioBusDbus {
    fn get_sinks(&self) -> zbus::Result<String>;
    fn set_node_volume(&self, node_id: u32, volume: f32) -> zbus::Result<String>;
    fn set_node_mute(&self, node_id: u32, mute: bool) -> zbus::Result<String>;
}

#[derive(Debug, Clone)]
pub struct AppAudioStream {
    pub id: String,
    pub name: String,
    pub volume: f64, // 0.0 .. 1.0
    pub muted: bool,
    pub icon: String,
}

pub enum AudioIntent {
    ToggleOutputMute,
    SetOutputVolume(f64),
    ToggleInputMute,
    SetInputVolume(f64),
    SetAppVolume { id: String, volume: f64 },
    ToggleAppMute { id: String },
    LaunchAudioSettings,
}

pub struct AudioViewModel;

impl AudioViewModel {
    pub fn fetch_app_streams<F: Fn(Vec<AppAudioStream>) + 'static>(on_streams: F) {
        gtk4::glib::MainContext::default().spawn_local(async move {
            let mut streams = Vec::new();
            if let Ok(conn) = zbus::Connection::session().await {
                if let Ok(proxy) = AudioBusDbusProxy::new(&conn).await {
                    if let Ok(json_str) = proxy.get_sinks().await {
                        if let Ok(parsed) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                            for item in parsed {
                                let id = item.get("id").and_then(|v| v.as_u64()).map(|v| v.to_string()).unwrap_or_default();
                                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("Flusso Audio").to_string();
                                let volume = item.get("volume").and_then(|v| v.as_f64()).unwrap_or(0.8);
                                let muted = item.get("muted").and_then(|v| v.as_bool()).unwrap_or(false);
                                if !id.is_empty() {
                                    let icon = match name.to_lowercase().as_str() {
                                        s if s.contains("firefox") || s.contains("chrome") || s.contains("browser") => "🌐",
                                        s if s.contains("spotify") || s.contains("media") || s.contains("mpv") || s.contains("vlc") => "🎵",
                                        s if s.contains("discord") || s.contains("telegram") || s.contains("slack") => "💬",
                                        _ => "🎛️",
                                    }.to_string();
                                    streams.push(AppAudioStream { id, name, volume, muted, icon });
                                }
                            }
                        }
                    }
                }
            }

            if streams.is_empty() {
                // Demo/Fallback streams if no active audio playing
                streams = vec![
                    AppAudioStream {
                        id: "demo-1".to_string(),
                        name: "Firefox / Browser Web".to_string(),
                        volume: 0.75,
                        muted: false,
                        icon: "🌐".to_string(),
                    },
                    AppAudioStream {
                        id: "demo-2".to_string(),
                        name: "Lettore Multimediale".to_string(),
                        volume: 0.85,
                        muted: false,
                        icon: "🎵".to_string(),
                    },
                    AppAudioStream {
                        id: "demo-3".to_string(),
                        name: "Suoni di Sistema".to_string(),
                        volume: 0.60,
                        muted: false,
                        icon: "🔔".to_string(),
                    },
                ];
            }
            on_streams(streams);
        });
    }

    pub fn execute_intent(intent: AudioIntent) {
        match intent {
            AudioIntent::ToggleOutputMute => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_audio_controller();
                    let _ = ctrl.toggle_mute().await;
                });
            }
            AudioIntent::SetOutputVolume(val) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_audio_controller();
                    let _ = ctrl.set_volume(val).await;
                });
            }
            AudioIntent::ToggleInputMute => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_audio_controller();
                    let _ = ctrl.toggle_source_mute().await;
                });
            }
            AudioIntent::SetInputVolume(val) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_audio_controller();
                    let _ = ctrl.set_source_volume(val).await;
                });
            }
            AudioIntent::SetAppVolume { id, volume } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    if let Ok(node_id) = id.parse::<u32>() {
                        if let Ok(conn) = zbus::Connection::session().await {
                            if let Ok(proxy) = AudioBusDbusProxy::new(&conn).await {
                                let _ = proxy.set_node_volume(node_id, volume as f32).await;
                            }
                        }
                    }
                });
            }
            AudioIntent::ToggleAppMute { id } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    if let Ok(node_id) = id.parse::<u32>() {
                        if let Ok(conn) = zbus::Connection::session().await {
                            if let Ok(proxy) = AudioBusDbusProxy::new(&conn).await {
                                let _ = proxy.set_node_mute(node_id, true).await;
                            }
                        }
                    }
                });
            }
            AudioIntent::LaunchAudioSettings => {
                let _ = std::process::Command::new("ermete-settings-rs").arg("--page").arg("audio").spawn();
            }
        }
    }
}
