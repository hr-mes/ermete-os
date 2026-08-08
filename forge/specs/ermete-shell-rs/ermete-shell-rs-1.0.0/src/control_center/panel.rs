use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use relm4::factory::FactoryVecDeque;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use super::module_item::{CcModuleItem, ModuleContent};
use super::network::NetworkModuleData;
use super::audio::AudioModuleData;
use super::display::DisplayModuleData;
use super::ebpf::EbpfModuleData;
use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};

pub struct ControlCenterPanel {
    pub app: gtk::Application,
    pub visible: bool,
    pub modules: FactoryVecDeque<CcModuleItem>,
}

#[derive(Debug)]
pub enum CcPanelInput {
    ToggleVisible,
    ClosePanel,
    UpdateNetwork(NetworkModuleData),
    UpdateAudio(AudioModuleData),
    UpdateDisplay(DisplayModuleData),
    UpdateEbpf(EbpfModuleData),
}

#[relm4::component(pub)]
impl SimpleComponent for ControlCenterPanel {
    type Input = CcPanelInput;
    type Output = ();
    type Init = gtk::Application;

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Ermete OS - Unified Control Center"),
            add_css_class: "popup-window",
            add_css_class: "cc-slideover-panel",
            set_default_width: 380,
            #[watch]
            set_visible: model.visible,

            gtk::Revealer {
                set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                set_transition_duration: 250,
                set_reveal_child: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_top: 14,
                    set_margin_bottom: 14,
                    set_margin_start: 12,
                    set_margin_end: 12,

                    // Panel Top Header Bar
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        set_valign: gtk::Align::Center,

                        gtk::Label {
                            set_label: "❖ Unified Control Center",
                            add_css_class: "cc-label-title",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                        },

                        gtk::Button {
                            set_label: "⚙ Settings",
                            add_css_class: "cc-quick-btn",
                            connect_clicked => move |_| {
                                ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchSettings(String::new()));
                            }
                        },

                        gtk::Button {
                            set_label: "✕",
                            add_css_class: "cc-quick-btn",
                            connect_clicked => CcPanelInput::ClosePanel,
                        }
                    },

                    // Modular Factory View Window
                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vexpand: true,

                        #[local_ref]
                        modules_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 10,
                        }
                    },

                    // Quick System Actions Row
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_homogeneous: true,

                        gtk::Button {
                            set_label: "🔒 Lock",
                            add_css_class: "cc-quick-btn",
                            connect_clicked => move |_| {
                                ControlCenterViewModel::execute_intent(ControlCenterIntent::TriggerLock);
                            }
                        },
                        gtk::Button {
                            set_label: "🖥 Standby",
                            add_css_class: "cc-quick-btn",
                            connect_clicked => move |_| {
                                ControlCenterViewModel::execute_intent(ControlCenterIntent::TriggerStandby);
                            }
                        },
                        gtk::Button {
                            set_label: " Shell",
                            add_css_class: "cc-quick-btn",
                            connect_clicked => move |_| {
                                ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchTerminal);
                            }
                        },
                        gtk::Button {
                            set_label: "📷 Snap",
                            add_css_class: "cc-quick-btn",
                            connect_clicked => move |_| {
                                ControlCenterViewModel::execute_intent(ControlCenterIntent::TriggerScreenshot);
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        app: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.set_application(Some(&app));
        root.init_layer_shell();
        root.set_layer(Layer::Overlay);
        root.set_namespace("control-center");

        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Right, true);
        root.set_anchor(Edge::Bottom, true);

        root.set_margin(Edge::Top, 34);
        root.set_margin(Edge::Right, 12);
        root.set_margin(Edge::Bottom, 12);

        crate::ui::popup_manager::setup_popup_autoclose(&root, "control-center");

        let mut modules = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let mut guard = modules.guard();

        // 1. Network Module
        guard.push_back(CcModuleItem {
            id: "net".to_string(),
            title: "Network & Connectivity".to_string(),
            icon: "󰤨".to_string(),
            content: ModuleContent::Network(NetworkModuleData::default()),
        });

        // 2. Audio Module (PipeWire Proxy)
        guard.push_back(CcModuleItem {
            id: "audio".to_string(),
            title: "Audio (PipeWire Proxy)".to_string(),
            icon: "🔊".to_string(),
            content: ModuleContent::Audio(AudioModuleData::default()),
        });

        // 3. Display Module (Brightness & Mica Tinting)
        guard.push_back(CcModuleItem {
            id: "display".to_string(),
            title: "Display & Mica Glass".to_string(),
            icon: "☀".to_string(),
            content: ModuleContent::Display(DisplayModuleData::default()),
        });

        // 4. eBPF Performance Modes Module
        guard.push_back(CcModuleItem {
            id: "ebpf".to_string(),
            title: "eBPF Autonomous Nervous System".to_string(),
            icon: "⚡".to_string(),
            content: ModuleContent::Ebpf(EbpfModuleData::default()),
        });

        drop(guard);

        let model = ControlCenterPanel {
            app: app.clone(),
            visible: true,
            modules,
        };

        let modules_box = model.modules.widget();
        let widgets = view_output!();

        // Esc key controller
        let key_ctrl = gtk::EventControllerKey::new();
        let sender_esc = sender.clone();
        key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gtk::gdk::Key::Escape {
                sender_esc.input(CcPanelInput::ClosePanel);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        root.add_controller(key_ctrl);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            CcPanelInput::ToggleVisible => {
                self.visible = !self.visible;
            }
            CcPanelInput::ClosePanel => {
                self.visible = false;
            }
            CcPanelInput::UpdateNetwork(data) => {
                let mut guard = self.modules.guard();
                if let Some(item) = guard.get_mut(0) {
                    item.content = ModuleContent::Network(data);
                }
            }
            CcPanelInput::UpdateAudio(data) => {
                let mut guard = self.modules.guard();
                if let Some(item) = guard.get_mut(1) {
                    item.content = ModuleContent::Audio(data);
                }
            }
            CcPanelInput::UpdateDisplay(data) => {
                let mut guard = self.modules.guard();
                if let Some(item) = guard.get_mut(2) {
                    item.content = ModuleContent::Display(data);
                }
            }
            CcPanelInput::UpdateEbpf(data) => {
                let mut guard = self.modules.guard();
                if let Some(item) = guard.get_mut(3) {
                    item.content = ModuleContent::Ebpf(data);
                }
            }
        }
    }
}
