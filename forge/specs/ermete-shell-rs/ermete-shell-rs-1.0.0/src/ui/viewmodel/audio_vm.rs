use std::process::Command;

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
            let streams = tokio::task::spawn_blocking(Self::parse_pactl_sink_inputs)
                .await
                .unwrap_or_default();
            on_streams(streams);
        });
    }

    fn parse_pactl_sink_inputs() -> Vec<AppAudioStream> {
        let output = Command::new("pactl")
            .arg("list")
            .arg("sink-inputs")
            .output();

        let mut streams = Vec::new();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            let blocks = text.split("\nSink ");
            for block in blocks {
                if block.trim().is_empty() {
                    continue;
                }
                let mut id = String::new();
                let mut app_name = String::new();
                let mut volume = 0.8;
                let mut muted = false;

                for line in block.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("d'ingresso #") || trimmed.starts_with("Input #") {
                        if let Some(idx) = trimmed.find('#') {
                            id = trimmed[idx + 1..].trim().to_string();
                        }
                    } else if trimmed.starts_with("application.name =") {
                        let parts: Vec<&str> = trimmed.split('=').collect();
                        if parts.len() > 1 {
                            app_name = parts[1].trim().trim_matches('"').to_string();
                        }
                    } else if trimmed.starts_with("Muto:") || trimmed.starts_with("Mute:") {
                        muted = trimmed.contains("sì") || trimmed.contains("yes") || trimmed.contains("true");
                    } else if trimmed.starts_with("Volume:") {
                        // Extract percentage e.g., "50%"
                        if let Some(pct_idx) = trimmed.find('%') {
                            let sub = &trimmed[..pct_idx];
                            if let Some(space_idx) = sub.rfind('/') {
                                let val_str = sub[space_idx + 1..].trim();
                                if let Ok(val) = val_str.parse::<f64>() {
                                    volume = val / 100.0;
                                }
                            }
                        }
                    }
                }

                if !id.is_empty() {
                    let display_name = if app_name.is_empty() {
                        format!("Applicazione #{}", id)
                    } else {
                        app_name.clone()
                    };

                    let icon = match display_name.to_lowercase().as_str() {
                        s if s.contains("firefox") || s.contains("chrome") || s.contains("browser") => "🌐",
                        s if s.contains("spotify") || s.contains("media") || s.contains("mpv") || s.contains("vlc") => "🎵",
                        s if s.contains("discord") || s.contains("telegram") || s.contains("slack") => "💬",
                        _ => "🎛️",
                    }.to_string();

                    streams.push(AppAudioStream {
                        id,
                        name: display_name,
                        volume,
                        muted,
                        icon,
                    });
                }
            }
        }

        if streams.is_empty() {
            // Demo/Fallback streams if no active audio playing
            vec![
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
            ]
        } else {
            streams
        }
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
                let vol_pct = (volume * 100.0).round() as u32;
                let cmd = format!("pactl set-sink-input-volume {} {}%", id, vol_pct);
                let _ = gtk4::glib::spawn_command_line_async(cmd);
            }
            AudioIntent::ToggleAppMute { id } => {
                let cmd = format!("pactl set-sink-input-mute {} toggle", id);
                let _ = gtk4::glib::spawn_command_line_async(cmd);
            }
            AudioIntent::LaunchAudioSettings => {
                let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs --page audio");
            }
        }
    }
}

