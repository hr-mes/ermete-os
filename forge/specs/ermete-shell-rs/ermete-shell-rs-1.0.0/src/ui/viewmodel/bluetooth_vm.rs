#[derive(Debug, Clone)]
pub struct BluetoothDeviceInfo {
    pub name: String,
    pub connected: bool,
}

pub enum BluetoothIntent {
    TogglePowered(bool),
    LaunchBluetoothSettings,
}

pub struct BluetoothViewModel;

impl BluetoothViewModel {
    pub fn fetch_initial_state<F: Fn(bool) + 'static>(on_powered: F) {
        gtk4::glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_bluetooth_controller();
            if let Ok(enabled) = ctrl.is_bluetooth_enabled().await {
                on_powered(enabled);
            }
        });
    }

    pub fn fetch_devices<F: Fn(Vec<BluetoothDeviceInfo>) + 'static>(on_devices: F) {
        gtk4::glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_bluetooth_controller();
            if let Ok(devices) = ctrl.list_bluetooth_devices().await {
                let items = devices.into_iter().map(|d| BluetoothDeviceInfo {
                    name: d.name,
                    connected: d.connected,
                }).collect();
                on_devices(items);
            } else {
                on_devices(vec![]);
            }
        });
    }

    pub fn execute_intent(intent: BluetoothIntent) {
        match intent {
            BluetoothIntent::TogglePowered(state) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_bluetooth_controller();
                    let _ = ctrl.toggle_bluetooth().await;
                    let _ = ctrl.set_bluetooth_powered(state).await;
                });
            }
            BluetoothIntent::LaunchBluetoothSettings => {
                let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs --page bluetooth");
            }
        }
    }
}
