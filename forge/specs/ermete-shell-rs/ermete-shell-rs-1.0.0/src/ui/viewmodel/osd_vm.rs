#[derive(Debug, Clone)]
pub enum OsdEvent {
    Volume(f64),
    Brightness(f64),
    CapsLock(bool),
}

pub struct OsdViewModel;

impl OsdViewModel {
    #[allow(dead_code)]
    pub fn emit(event: OsdEvent) {
        match event {
            OsdEvent::Volume(v) => {
                crate::ipc::system_proxies::get_audio_bus().emit(crate::ipc::types::AudioEvent::VolumeChanged(v));
            }
            OsdEvent::Brightness(b) => {
                crate::ipc::system_proxies::get_hardware_bus().emit(crate::ipc::types::HardwareEvent::BrightnessChanged(b));
            }
            OsdEvent::CapsLock(active) => {
                crate::ipc::system_proxies::get_hardware_bus().emit(crate::ipc::types::HardwareEvent::CapsLockToggled(active));
            }
        }
    }

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
                match ev {
                    crate::ipc::types::HardwareEvent::BrightnessChanged(b) => {
                        on_event_hw(OsdEvent::Brightness(b));
                    }
                    crate::ipc::types::HardwareEvent::CapsLockToggled(active) => {
                        on_event_hw(OsdEvent::CapsLock(active));
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osd_events() {
        let vol_ev = OsdEvent::Volume(0.75);
        let bright_ev = OsdEvent::Brightness(50.0);
        let caps_ev = OsdEvent::CapsLock(true);

        match vol_ev {
            OsdEvent::Volume(v) => assert_eq!(v, 0.75),
            _ => panic!("Expected Volume"),
        }
        match bright_ev {
            OsdEvent::Brightness(b) => assert_eq!(b, 50.0),
            _ => panic!("Expected Brightness"),
        }
        match caps_ev {
            OsdEvent::CapsLock(c) => assert!(c),
            _ => panic!("Expected CapsLock"),
        }
    }
}
