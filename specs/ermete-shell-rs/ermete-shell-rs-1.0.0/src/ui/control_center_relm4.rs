use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use gtk::prelude::*;

// Draft Architecture for Control Center using Relm4 (Model-View-Update pattern)

pub struct ControlCenterModel {
    pub volume: f64,
    pub brightness: f64,
    pub is_wifi_enabled: bool,
    pub is_bluetooth_enabled: bool,
    pub is_do_not_disturb: bool,
    pub is_dark_mode: bool,
}

#[derive(Debug)]
pub enum ControlCenterInput {
    ToggleWifi,
    ToggleBluetooth,
    ToggleDoNotDisturb,
    ToggleDarkMode,
    SetVolume(f64),
    SetBrightness(f64),
}

#[relm4::component(pub)]
impl SimpleComponent for ControlCenterModel {
    type Input = ControlCenterInput;
    type Output = ();
    type Init = ();

    view! {
        gtk::Window {
            set_title: Some("Control Center"),
            set_default_size: (350, 500),
            set_css_classes: &["control-center-window"],
            
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_margin_all: 16,
                
                // Quick Toggles
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_homogeneous: true,
                    
                    gtk::ToggleButton {
                        set_label: "Wi-Fi",
                        set_active: model.is_wifi_enabled,
                        connect_toggled => ControlCenterInput::ToggleWifi,
                    },
                    
                    gtk::ToggleButton {
                        set_label: "Bluetooth",
                        set_active: model.is_bluetooth_enabled,
                        connect_toggled => ControlCenterInput::ToggleBluetooth,
                    },
                    
                    gtk::ToggleButton {
                        set_label: "Dark Mode",
                        set_active: model.is_dark_mode,
                        connect_toggled => ControlCenterInput::ToggleDarkMode,
                    },
                },
                
                // Sliders
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    
                    gtk::Scale {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_range: (0.0, 100.0),
                        set_value: model.volume,
                        connect_value_changed[sender] => move |scale| {
                            sender.input(ControlCenterInput::SetVolume(scale.value()));
                        }
                    },
                    
                    gtk::Scale {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_range: (0.0, 100.0),
                        set_value: model.brightness,
                        connect_value_changed[sender] => move |scale| {
                            sender.input(ControlCenterInput::SetBrightness(scale.value()));
                        }
                    }
                },
                
                // DND Toggle
                gtk::Switch {
                    set_active: model.is_do_not_disturb,
                    connect_state_set[sender] => move |_, _| {
                        sender.input(ControlCenterInput::ToggleDoNotDisturb);
                        gtk::glib::Propagation::Proceed
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ControlCenterModel {
            volume: 50.0,
            brightness: 80.0,
            is_wifi_enabled: true,
            is_bluetooth_enabled: false,
            is_do_not_disturb: false,
            is_dark_mode: true,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ControlCenterInput::ToggleWifi => {
                self.is_wifi_enabled = !self.is_wifi_enabled;
            }
            ControlCenterInput::ToggleBluetooth => {
                self.is_bluetooth_enabled = !self.is_bluetooth_enabled;
            }
            ControlCenterInput::ToggleDoNotDisturb => {
                self.is_do_not_disturb = !self.is_do_not_disturb;
            }
            ControlCenterInput::ToggleDarkMode => {
                self.is_dark_mode = !self.is_dark_mode;
            }
            ControlCenterInput::SetVolume(val) => {
                self.volume = val;
            }
            ControlCenterInput::SetBrightness(val) => {
                self.brightness = val;
            }
        }
    }
}
