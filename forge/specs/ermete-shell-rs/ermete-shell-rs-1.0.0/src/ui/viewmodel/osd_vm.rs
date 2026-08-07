pub enum OsdEvent {
    Volume(f64),
    Brightness(f64),
}

pub struct OsdViewModel;

impl OsdViewModel {
    pub fn subscribe<F: Fn(OsdEvent) + 'static>(on_event: F) {
        let on_event_rc = std::rc::Rc::new(on_event);

        let on_event_audio = on_event_rc.clone();
        let mut audio_rx = crate::ipc::system_proxies::get_audio_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(ev) = audio_rx.recv().await {
                if let crate::ipc::types::AudioEvent::VolumeChanged(v) = ev {
                    on_event_audio(OsdEvent::Volume(v));
                }
            }
        });

        let on_event_hw = on_event_rc;
        let mut hw_rx = crate::ipc::system_proxies::get_hardware_bus().subscribe();
        gtk4::glib::MainContext::default().spawn_local(async move {
            while let Ok(ev) = hw_rx.recv().await {
                let crate::ipc::types::HardwareEvent::BrightnessChanged(b) = ev;
                on_event_hw(OsdEvent::Brightness(b));
            }
        });
    }
}
