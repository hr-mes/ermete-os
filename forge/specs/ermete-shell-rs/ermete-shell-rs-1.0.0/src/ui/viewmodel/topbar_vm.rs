#[derive(Debug, Clone)]
pub struct TopbarState {
    pub clock_text: String,
    pub battery_percent: f64,
    pub has_battery: bool,
    pub network_icon: String,
    pub focused_app_title: String,
}

pub struct TopbarViewModel;

impl TopbarViewModel {
    pub fn get_clock_string() -> String {
        crate::core::macos_clock_string()
    }

    pub fn get_network_status() -> (String, String, String) {
        crate::core::get_network_controller().get_cached_network_status()
    }

    pub fn get_live_state() -> (bool, f64) {
        let live = crate::core::live_state::get_live_state();
        (live.has_battery, live.battery_percent)
    }

    pub fn get_focused_title() -> String {
        let niri = crate::core::niri_state::get_niri_state();
        niri.focused_window_title.unwrap_or_else(|| "Ermete OS".to_string())
    }

    pub fn subscribe_events<F: Fn() + 'static>(on_event: F) {
        let on_event_rc = std::rc::Rc::new(on_event);

        let on_event_net = on_event_rc.clone();
        let mut net_rx = crate::ipc::system_proxies::get_net_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(_) = net_rx.recv().await {
                on_event_net();
            }
        });

        let on_event_audio = on_event_rc;
        let mut audio_rx = crate::ipc::system_proxies::get_audio_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(_) = audio_rx.recv().await {
                on_event_audio();
            }
        });
    }
}
