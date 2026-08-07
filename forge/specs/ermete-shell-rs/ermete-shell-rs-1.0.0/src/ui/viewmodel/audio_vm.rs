pub enum AudioIntent {
    ToggleOutputMute,
    SetOutputVolume(f64),
    ToggleInputMute,
    SetInputVolume(f64),
    LaunchAudioSettings,
}

pub struct AudioViewModel;

impl AudioViewModel {
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
            AudioIntent::LaunchAudioSettings => {
                let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs --page audio");
            }
        }
    }
}
