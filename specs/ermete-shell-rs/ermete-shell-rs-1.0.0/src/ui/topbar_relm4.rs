use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use relm4::factory::{FactoryComponent, FactoryVecDeque, FactorySender};
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use crate::core::*;
use crate::ui::spotlight::*;
use crate::ui::notifications::*;
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
                crate::core::niri_client::focus_workspace_by_id(self.ws.id);
            }
        }
    }
}

thread_local! {
    static CSS_PROVIDER: std::cell::RefCell<Option<gtk4::CssProvider>> = std::cell::RefCell::new(None);
}

fn load_css() {
    let home = std::env::var("HOME").unwrap();
    let colors_path = format!("{}/.config/ermete-shell/colors.css", home);
    let colors_css = std::fs::read_to_string(&colors_path).unwrap_or_default();
    
    let fallback = if colors_css.is_empty() {
        r#"
        @define-color shell_bg rgba(28, 28, 30, 0.65);
        @define-color shell_fg #f5f5f7;
        @define-color shell_border rgba(255, 255, 255, 0.08);
        @define-color shell_hover rgba(255, 255, 255, 0.1);
        @define-color shell_primary #ffffff;
        
        /* Seelen UI Global Variables */
        @define-color spacing_2xs 4px;
        @define-color spacing_xs 8px;
        @define-color spacing_s 12px;
        @define-color spacing_m 16px;
        @define-color spacing_l 20px;
        @define-color spacing_xl 24px;
        @define-color spacing_2xl 32px;

        @define-color border_radius 10px;
        @define-color border_width 3px;

        @define-color shadow_color rgba(0, 0, 0, 1.0);
        @define-color shadow_s rgba(0, 0, 0, 0.08);
        @define-color shadow_m rgba(0, 0, 0, 0.16);
        @define-color shadow_l rgba(0, 0, 0, 0.24);

        @define-color color_white rgb(255, 255, 255);
        @define-color color_black rgb(0, 0, 0);
        "#
    } else {
        ""
    };

    // Preleviamo il CSS originale da topbar
    let topbar_css = r#"
    window.topbar-window { background-color: transparent; }
    .topbar-container {
        background: @shell_bg;
        border-bottom: 1px solid @shell_border;
        color: @shell_fg;
        font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
        font-size: 13px; font-weight: 500; padding: 0 10px;
    }
    .macos-menu-item {
        background: transparent; border: none; border-radius: 5px; padding: 4px 12px;
        color: @shell_fg; font-weight: 500; transition: all 0.25s;
    }
    .macos-menu-item:hover { background: @shell_hover; color: @shell_primary; }
    .workspace-focused { color: #0a84ff; font-weight: 900; }
    .workspace-active { color: #f5f5f7; font-weight: 700; }
    .macos-status-item {
        background: transparent; border: none; border-radius: 5px; padding: 4px 12px;
        color: @shell_fg; font-size: 14px; transition: all 0.25s;
    }
    .macos-status-item:hover { background: @shell_hover; color: @shell_primary; }
    .macos-clock { font-weight: 700; }
    "#;

    let full_css = format!("{}\n{}\n{}", colors_css, fallback, topbar_css);

    CSS_PROVIDER.with(|p| {
        let mut provider_opt = p.borrow_mut();
        let display = gtk4::gdk::Display::default().unwrap();
        if let Some(old_provider) = provider_opt.as_ref() {
            gtk4::style_context_remove_provider_for_display(&display, old_provider);
        }
        let new_provider = gtk4::CssProvider::new();
        new_provider.load_from_data(&full_css);
        gtk4::style_context_add_provider_for_display(
            &display,
            &new_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        *provider_opt = Some(new_provider);
    });
}

fn spawn_css_watcher() {
    let (sender, receiver) = glib::MainContext::channel::<()>(glib::Priority::DEFAULT);
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let path = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".config/ermete-shell");
        let _ = notify::Watcher::watch(&mut watcher, &path, notify::RecursiveMode::NonRecursive);
        while let Ok(event) = rx.recv() {
            if let Ok(ev) = event {
                if ev.kind.is_modify() { let _ = sender.send(()); }
            }
        }
    });
    receiver.attach(None, move |_| {
        load_css();
        glib::ControlFlow::Continue
    });
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
}

#[relm4::component(pub)]
impl SimpleComponent for TopbarModel {
    type Input = TopbarInput;
    type Output = ();
    type Init = gtk::Application;

    view! {
        gtk::Window {
            set_title: Some("Ermete Shell - Topbar (Relm4)"),
            add_css_class: "topbar-window",
            
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
        // Init di sistema (staccate dal vecchio topbar)
        load_css();
        spawn_css_watcher();
        crate::ui::notifications::spawn_notification_daemon(&app);

        root.set_application(Some(&app));
        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_namespace("bar-relm4");
        root.auto_exclusive_zone_enable();
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.set_height_request(28);

        // Factory dei Workspace
        let workspaces = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        // Modello Iniziale
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

        // Avvio dei Timer Asincroni (Tick) per aggiornare lo stato dichiarativamente
        let sender_slow = sender.clone();
        glib::timeout_add_seconds_local(5, move || {
            sender_slow.input(TopbarInput::TickSecond);
            glib::ControlFlow::Continue
        });

        let sender_fast = sender.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            sender_fast.input(TopbarInput::TickFast);
            glib::ControlFlow::Continue
        });

        // Watcher nativo per i workspace di Niri via DBus/Socket
        let (niri_tx, niri_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
        crate::core::spawn_niri_workspace_watcher(niri_tx);
        
        let sender_ws = sender.clone();
        niri_rx.attach(None, move |workspaces_data| {
            sender_ws.input(TopbarInput::UpdateWorkspaces(workspaces_data));
            glib::ControlFlow::Continue
        });

        // Forza un tick immediato
        sender.input(TopbarInput::TickSecond);
        sender.input(TopbarInput::TickFast);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            TopbarInput::TickSecond => {
                // Aggiornamento dichiarativo di Orologio, Rete e Batteria
                self.clock_text = crate::core::macos_clock_string();
                
                let (net_icon, _, _) = crate::core::get_network_status();
                self.network_icon = net_icon;
                
                let live = crate::core::live_state::get_live_state();
                self.has_battery = live.has_battery;
                self.battery_percent = live.battery_percent;
            }
            TopbarInput::TickFast => {
                // Aggiornamento veloce del titolo app focalizzata via Niri
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
            // Eventi UI gestiti delegando le chiamate esistenti
            TopbarInput::ToggleStartMenu => {
                crate::ui::topbar::toggle_or_open_popup("launcher", || crate::ui::control_center::show_start_menu_popover(&self.app));
            }
            TopbarInput::ToggleControlCenter => {
                crate::ui::topbar::toggle_or_open_popup("control-center", || crate::ui::control_center::show_control_center_popover(&self.app));
            }
            TopbarInput::ToggleSpotlight => {
                crate::ui::topbar::toggle_or_open_popup("spotlight", || crate::ui::spotlight::show_spotlight_modal(&self.app));
            }
            TopbarInput::ToggleCalendar => {
                crate::ui::topbar::toggle_or_open_popup("calendar", || crate::ui::control_center::show_calendar_popover(&self.app));
            }
            TopbarInput::ToggleWifi => {
                crate::ui::topbar::toggle_or_open_popup("wifi", || crate::ui::control_center::show_wifi_popover(&self.app));
            }
            TopbarInput::ToggleNotifications => {
                crate::ui::topbar::toggle_or_open_popup("notifications", || crate::ui::notifications::show_notification_center(&self.app));
            }
        }
    }
}
