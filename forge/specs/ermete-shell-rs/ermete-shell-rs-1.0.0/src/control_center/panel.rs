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

#[zbus::proxy(
    interface = "os.ermete.MeshSync",
    default_service = "os.ermete.MeshSync",
    default_path = "/os/ermete/MeshSync"
)]
trait MeshSync {
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn ip_addr(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn signal_strength(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn wifi_enabled(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn eth_active(&self) -> zbus::Result<bool>;
}

#[zbus::proxy(
    interface = "os.ermete.AiDaemon",
    default_service = "os.ermete.AiDaemon",
    default_path = "/os/ermete/AiDaemon"
)]
trait AiDaemon {
    #[zbus(property)]
    fn current_mode(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn ring0_tracepoints_active(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn latency_reduction_pct(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn context_switches_saved(&self) -> zbus::Result<u64>;
    #[zbus(property)]
    fn status_text(&self) -> zbus::Result<String>;
}

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
            add_css_class: "glassmorphism",
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

        let sender_mesh = sender.clone();
        glib::spawn_future_local(async move {
            let conn = match zbus::Connection::system().await {
                Ok(c) => c,
                Err(_) => return,
            };
            
            if let Ok(mesh_proxy) = MeshSyncProxy::new(&conn).await {
                macro_rules! update_mesh_macro { () => {
                    if let (Ok(connected), Ok(ssid), Ok(ip_addr), Ok(signal), Ok(wifi), Ok(eth)) = 
                        (mesh_proxy.connected().await, mesh_proxy.ssid().await, mesh_proxy.ip_addr().await, mesh_proxy.signal_strength().await, mesh_proxy.wifi_enabled().await, mesh_proxy.eth_active().await) {
                        let net_data = NetworkModuleData {
                            connected,
                            ssid,
                            ip_addr,
                            signal_strength: signal,
                            wifi_enabled: wifi,
                            eth_active: eth,
                        };
                        sender_mesh.input(CcPanelInput::UpdateNetwork(net_data));
                    }
                } }
                
                update_mesh_macro!();
                
                let mut changes = mesh_proxy.receive_ssid_changed().await; {
                    use futures_util::stream::StreamExt;
                    while let Some(_) = changes.next().await {
                        update_mesh_macro!();
                    }
                }
            }
        });

        let sender_ai = sender.clone();
        glib::spawn_future_local(async move {
            let conn = match zbus::Connection::system().await {
                Ok(c) => c,
                Err(_) => return,
            };
            
            if let Ok(ai_proxy) = AiDaemonProxy::new(&conn).await {
                macro_rules! update_ai_macro { () => {
                    if let (Ok(mode_str), Ok(tracepoints), Ok(latency), Ok(cs_saved), Ok(status)) = 
                        (ai_proxy.current_mode().await, ai_proxy.ring0_tracepoints_active().await, ai_proxy.latency_reduction_pct().await, ai_proxy.context_switches_saved().await, ai_proxy.status_text().await) {
                        
                        let mode = match mode_str.as_str() {
                            "GamingLowLatency" => crate::control_center::ebpf::EbpfMode::GamingLowLatency,
                            "EcoSaver" => crate::control_center::ebpf::EbpfMode::EcoSaver,
                            "MaxThroughput" => crate::control_center::ebpf::EbpfMode::MaxThroughput,
                            _ => crate::control_center::ebpf::EbpfMode::AiInferred,
                        };
                        
                        let ebpf_data = EbpfModuleData {
                            current_mode: mode,
                            ring0_tracepoints_active: tracepoints as usize,
                            latency_reduction_pct: latency,
                            context_switches_saved: cs_saved,
                            status_text: status,
                        };
                        sender_ai.input(CcPanelInput::UpdateEbpf(ebpf_data));
                    }
                } }
                
                update_ai_macro!();
                
                let mut changes = ai_proxy.receive_current_mode_changed().await; {
                    use futures_util::stream::StreamExt;
                    while let Some(_) = changes.next().await {
                        update_ai_macro!();
                    }
                }
            }
        });

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






