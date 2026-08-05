use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use relm4::factory::{FactoryComponent, FactoryVecDeque, FactorySender};
use gtk::prelude::*;
use gtk4::Application;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::ui::control_center::*;

pub struct WorkspaceItem {
    pub ws: crate::core::NiriWorkspace,
}

#[derive(Debug)]
pub enum WorkspaceMsg {
    Focus,
}

#[relm4::factory(pub)]
impl FactoryComponent for WorkspaceItem {
    type Init = crate::core::NiriWorkspace;
    type Input = WorkspaceMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            #[watch]
            set_css_classes: &[
                "macos-menu-item",
                if self.ws.is_focused { "workspace-focused" } 
                else if self.ws.is_active { "workspace-active" } 
                else { "" }
            ],
            
            #[watch]
            set_label: if self.ws.is_active { "●" } else { "○" },
            
            connect_clicked => WorkspaceMsg::Focus,
        }
    }

    fn init_model(init: Self::Init, _index: &relm4::factory::DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { ws: init }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            WorkspaceMsg::Focus => {
                let id = self.ws.id;
                glib::MainContext::default().spawn_local(async move {
                    ermete_niri_ipc::async_client::focus_workspace_by_id(id).await;
                });
            }
        }
    }
}

pub struct TopbarModel {
    pub app: gtk::Application,
    pub clock_text: String,
    pub battery_percent: f64,
    pub has_battery: bool,
    pub network_icon: String,
    pub focused_app_title: String,
    pub workspaces: FactoryVecDeque<WorkspaceItem>,
}

#[derive(Debug)]
pub enum TopbarInput {
    TickSecond,          // Aggiorna orologio e stato base
    TickFast,            // Aggiorna titolo app
    UpdateWorkspaces(Vec<crate::core::NiriWorkspace>),
    ToggleStartMenu,
    ToggleControlCenter,
    ToggleSpotlight,
    ToggleCalendar,
    ToggleWifi,
    ToggleNotifications,
    ToggleDesktopWidgets,
    ToggleLiveTheming,
}

#[relm4::component(pub)]
impl SimpleComponent for TopbarModel {
    type Input = TopbarInput;
    type Output = ();
    type Init = gtk::Application;

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Ermete Shell - Topbar"),
            add_css_class: "topbar-window",
            set_visible: true,
            
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                add_css_class: "topbar-container",
                set_hexpand: true,
                
                gtk::CenterBox {
                    set_hexpand: true,
                    
                    // --- ISOLA SINISTRA ---
                    #[wrap(Some)]
                    set_start_widget = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 2,
                        set_valign: gtk::Align::Center,
                        
                        gtk::Button {
                            set_label: "◈",
                            add_css_class: "macos-menu-item",
                            add_css_class: "macos-apple-logo",
                            connect_clicked => TopbarInput::ToggleStartMenu,
                        },
                        
                        gtk::Button {
                            #[watch]
                            set_label: &model.focused_app_title,
                            add_css_class: "macos-menu-item",
                            add_css_class: "macos-app-title",
                        }
                    },
                    
                    // --- ISOLA CENTRALE (Workspaces Factory) ---
                    #[wrap(Some)]
                    set_center_widget = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_valign: gtk::Align::Center,
                        
                        #[local_ref]
                        workspaces_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                        }
                    },
                    
                    // --- ISOLA DESTRA ---
                    #[wrap(Some)]
                    set_end_widget = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 2,
                        set_valign: gtk::Align::Center,
                        
                        #[name = "morphic_pill"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "macos-status-item",
                            add_css_class: "morphic-pill",
                            
                            gtk::Label {
                                set_label: "✨",
                                set_margin_start: 4,
                                set_margin_end: 4,
                            },
                            
                            #[name = "pill_revealer"]
                            gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                                set_transition_duration: 300,
                                
                                gtk::Label {
                                    set_label: "AI Tasks bg...",
                                    set_margin_end: 8,
                                    add_css_class: "pill-text",
                                }
                            },
                            
                            add_controller = gtk::EventControllerMotion {
                                connect_enter [pill_revealer] => move |_, _, _| {
                                    pill_revealer.set_reveal_child(true);
                                },
                                connect_leave [pill_revealer] => move |_| {
                                    pill_revealer.set_reveal_child(false);
                                }
                            }
                        },
                        
                        gtk::Button {
                            #[watch]
                            set_visible: model.has_battery,
                            #[watch]
                            set_label: &format!("{}% 󰁹", model.battery_percent.round() as i32),
                            add_css_class: "macos-status-item",
                        },
                        
                        gtk::Button {
                            #[watch]
                            set_label: &model.network_icon,
                            add_css_class: "macos-status-item",
                            connect_clicked => TopbarInput::ToggleWifi,
                        },
                        
                        gtk::Button {
                            set_label: "🔍",
                            add_css_class: "macos-status-item",
                            connect_clicked => TopbarInput::ToggleSpotlight,
                        },
                        
                        gtk::Button {
                            set_label: "❖",
                            add_css_class: "macos-status-item",
                            connect_clicked => TopbarInput::ToggleControlCenter,
                        },
                        
                        gtk::Button {
                            set_label: "🧩",
                            add_css_class: "macos-status-item",
                            set_tooltip_text: Some("Desktop Widgets"),
                            connect_clicked => TopbarInput::ToggleDesktopWidgets,
                        },
                        
                        gtk::Button {
                            set_label: "🎨",
                            add_css_class: "macos-status-item",
                            set_tooltip_text: Some("Live Theming & Dynamic Accent"),
                            connect_clicked => TopbarInput::ToggleLiveTheming,
                        },
                        
                        gtk::Button {
                            set_label: "󰂚",
                            add_css_class: "macos-status-item",
                            connect_clicked => TopbarInput::ToggleNotifications,
                        },
                        
                        gtk::Button {
                            #[watch]
                            set_label: &model.clock_text,
                            add_css_class: "macos-status-item",
                            add_css_class: "macos-clock",
                            connect_clicked => TopbarInput::ToggleCalendar,
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
        crate::ui::notifications::spawn_notification_daemon(&app);

        root.set_application(Some(&app));
        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_namespace("bar");
        root.auto_exclusive_zone_enable();
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.set_height_request(28);

        let workspaces = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = TopbarModel {
            app: app.clone(),
            clock_text: "Caricamento...".to_string(),
            battery_percent: 100.0,
            has_battery: true,
            network_icon: "󰤨".to_string(),
            focused_app_title: "Ermete OS".to_string(),
            workspaces,
        };

        let workspaces_box = model.workspaces.widget();
        let widgets = view_output!();

        let sender_slow = sender.clone();
        glib::timeout_add_seconds_local(5, move || {
            sender_slow.input(TopbarInput::TickSecond);
            glib::ControlFlow::Continue
        });

        let sender_ws = sender.clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let event_bus = crate::core::system_proxies::get_event_bus();
        event_bus.subscribe(move |ev| {
            let _ = tx.send(ev.clone());
        });

        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    crate::core::system_proxies::SystemEvent::NetworkUpdated(_) |
                    crate::core::system_proxies::SystemEvent::VolumeChanged(_) |
                    crate::core::system_proxies::SystemEvent::WifiToggled(_) |
                    crate::core::system_proxies::SystemEvent::BluetoothToggled(_) => {
                        sender_ws.input(TopbarInput::TickSecond);
                    },
                    _ => {}
                }
            }
        });

        let (niri_tx, niri_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
        crate::core::spawn_niri_workspace_watcher(niri_tx);
        
        let sender_ws_niri = sender.clone();
        niri_rx.attach(None, move |workspaces_data| {
            sender_ws_niri.input(TopbarInput::TickFast);
            sender_ws_niri.input(TopbarInput::UpdateWorkspaces(workspaces_data));
            glib::ControlFlow::Continue
        });

        sender.input(TopbarInput::TickSecond);
        sender.input(TopbarInput::TickFast);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            TopbarInput::TickSecond => {
                self.clock_text = crate::core::macos_clock_string();
                
                let (net_icon, _, _) = crate::core::system_proxies::get_network_controller().get_cached_network_status();
                self.network_icon = net_icon;
                
                let live = crate::core::live_state::get_live_state();
                self.has_battery = live.has_battery;
                self.battery_percent = live.battery_percent;
            }
            TopbarInput::TickFast => {
                let niri = crate::core::niri_state::get_niri_state();
                self.focused_app_title = niri.focused_window_title.unwrap_or_else(|| "Ermete OS".to_string());
            }
            TopbarInput::UpdateWorkspaces(workspaces_data) => {
                let active_output = workspaces_data.iter()
                    .find(|w| w.is_focused)
                    .or_else(|| workspaces_data.iter().find(|w| w.is_active))
                    .map(|w| w.output.clone())
                    .unwrap_or_default();

                let mut filtered_ws: Vec<_> = workspaces_data.into_iter().filter(|w| w.output == active_output).collect();
                filtered_ws.sort_by_key(|w| w.idx);

                let mut ws_guard = self.workspaces.guard();
                ws_guard.clear();
                for ws in filtered_ws {
                    ws_guard.push_back(ws);
                }
            }
            TopbarInput::ToggleStartMenu => {
                toggle_or_open_popup("launcher", || crate::ui::control_center::show_start_menu_popover(&self.app));
            }
            TopbarInput::ToggleControlCenter => {
                toggle_or_open_popup("control-center", || crate::ui::control_center::show_control_center_popover(&self.app));
            }
            TopbarInput::ToggleSpotlight => {
                toggle_or_open_popup("spotlight", || crate::ui::spotlight::show_spotlight_modal(&self.app));
            }
            TopbarInput::ToggleCalendar => {
                toggle_or_open_popup("calendar", || crate::ui::control_center::show_calendar_popover(&self.app));
            }
            TopbarInput::ToggleWifi => {
                toggle_or_open_popup("wifi", || crate::ui::control_center::show_wifi_popover(&self.app));
            }
            TopbarInput::ToggleNotifications => {
                toggle_or_open_popup("notifications", || crate::ui::notifications::show_notification_center(&self.app));
            }
            TopbarInput::ToggleDesktopWidgets => {
                let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs --page desktop");
            }
            TopbarInput::ToggleLiveTheming => {
                let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs --page appearance");
            }
        }
    }
}

pub fn handle_command(app: &Application, arg: &str) {
    match arg {
        "spotlight" | "launcher" => toggle_or_open_popup("spotlight", || crate::ui::spotlight::show_spotlight_modal(app)),
        "control-center" => toggle_or_open_popup("control-center", || show_control_center_popover(app)),
        "notifications" | "notification-center" => toggle_or_open_popup("notifications", || crate::ui::notifications::show_notification_center(app)),
        "sys-monitor" | "monitor" => toggle_or_open_popup("sys-monitor", || show_system_monitor_modal(app)),
        "calendar" => toggle_or_open_popup("calendar", || show_calendar_popover(app)),
        "media-player" | "mixer" | "audio" => toggle_or_open_popup("media-player", || show_audio_mixer_popover(app)),
        "wifi" => toggle_or_open_popup("wifi", || show_wifi_popover(app)),
        "bluetooth" => toggle_or_open_popup("bluetooth", || show_bluetooth_popover(app)),
        "start-menu" | "menu" => toggle_or_open_popup("launcher", || show_start_menu_popover(app)),
        "powermenu" => toggle_or_open_popup("powermenu", || crate::ui::powermenu::show_powermenu_modal(app)),
        "clipboard" => toggle_or_open_popup("clipboard", || crate::ui::clipboard::show_clipboard_modal(app)),
        "store" => toggle_or_open_popup("store", || crate::ui::store::show_store_modal(app)),
        "dock" => crate::ui::dock::toggle_dock_visibility(),
        _ => {}
    }
}

pub fn toggle_or_open_popup(tag: &str, open_fn: impl FnOnce()) {
    let mut to_close = None;
    let mut already_open = false;
    crate::wayland::popup::ACTIVE_POPUP.with(|p| {
        if let Some((old_tag, old_weak)) = p.borrow().as_ref() {
            if let Some(old_win) = old_weak.upgrade() {
                if old_win.is_visible() {
                    to_close = Some(old_win);
                    if old_tag == tag {
                        already_open = true;
                    }
                }
            }
        }
        *p.borrow_mut() = None;
    });

    if let Some(win) = to_close {
        use gtk4::prelude::WidgetExt;
        win.set_visible(false);
    }
    
    if !already_open {
        open_fn();
    }
}

