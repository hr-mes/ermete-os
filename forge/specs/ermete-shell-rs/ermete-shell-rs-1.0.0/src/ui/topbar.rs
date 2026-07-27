use notify::{Watcher, RecursiveMode};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use relm4::factory::{FactoryComponent, FactoryVecDeque, FactorySender};
use gtk::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use crate::core::*;
use crate::ui::spotlight::*;
use crate::ui::notifications::*;
use crate::ui::control_center::*;

pub const TOPBAR_CSS: &str = r#"
window.topbar-window {
    background-color: transparent;
}

window.bg-overlay-window {
    background-color: rgba(0, 0, 0, 0.01);
}

.topbar-container {
    background: rgba(28, 28, 30, 0.45);
    backdrop-filter: blur(20px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    color: @shell_fg;
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 13px;
    font-weight: 500;
    padding: 0 10px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
}

.macos-menu-item {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 4px 12px;
    color: @shell_fg;
    font-size: 13px;
    font-weight: 500;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.macos-menu-item:hover {
    background: @shell_hover;
    color: @shell_primary;
}

.macos-apple-logo {
    font-size: 16px;
    font-weight: 700;
}

.macos-app-title {
    font-weight: 700;
    color: @shell_primary;
}

.macos-status-item {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 4px 12px;
    color: @shell_fg;
    font-size: 14px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.macos-status-item:hover {
    background: @shell_hover;
    color: @shell_primary;
}

.macos-clock {
    font-weight: 700;
}

/* ==========================================
   ANIMATIONS & KEYFRAMES
   ========================================== */
@keyframes slide-down-fade {
    0% {
        opacity: 0;
        transform: translateY(-20px) scale(0.98);
    }
    100% {
        opacity: 1;
        transform: translateY(0) scale(1.0);
    }
}

@keyframes pop-in-fade {
    0% {
        opacity: 0;
        transform: scale(0.95);
    }
    100% {
        opacity: 1;
        transform: scale(1.0);
    }
}

/* ==========================================
   macOS SPOTLIGHT MODAL (Win+D)
   ========================================== */
window.spotlight-window {
    background-color: transparent;
}

.spotlight-card {
    background: rgba(28, 28, 30, 0.45);
    backdrop-filter: blur(25px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 20px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
    animation: pop-in-fade 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.spotlight-input {
    background: transparent;
    border: none;
    box-shadow: none;
    color: #ffffff;
    font-size: 32px;
    font-weight: 300;
    padding: 16px 20px;
}

.spotlight-input:focus {
    border: none;
    background: transparent;
    box-shadow: none;
    outline: none;
}

.spotlight-item {
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 12px 16px;
    color: #f5f5f7;
    transition: all 0.15s ease-in-out;
}

.spotlight-item:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
}

.spotlight-item-title {
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 16px;
    font-weight: 500;
    color: #ffffff;
}

.spotlight-item-desc {
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 13px;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.5);
}

/* ==========================================
   macOS CONTROL CENTER POPOVER
   ========================================== */
window.popup-window {
    background-color: transparent;
}

.cc-card {
    background: rgba(28, 28, 30, 0.45);
    backdrop-filter: blur(25px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 20px;
    padding: 16px;
    color: #f8fafc;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
    animation: slide-down-fade 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.cc-tile {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 10px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-tile:hover {
    background: rgba(255, 255, 255, 0.12);
    transform: scale(1.02);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.cc-tile:active {
    transform: scale(0.96);
}

.cc-tile-row {
    background: transparent;
    border: none;
    border-radius: 10px;
    padding: 6px 8px;
    color: #f5f5f7;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-tile-row:hover {
    background: rgba(255, 255, 255, 0.08);
    transform: translateX(4px);
}

.cc-tile-row:active {
    background: rgba(255, 255, 255, 0.04);
    transform: translateX(0px);
}

.cc-circle-blue {
    background: #0a84ff;
    border-radius: 999px;
    min-width: 28px;
    min-height: 28px;
    color: #ffffff;
    font-weight: 700;
}

.cc-circle-indigo {
    background: #5e5ce6;
    border-radius: 999px;
    min-width: 28px;
    min-height: 28px;
    color: #ffffff;
    font-weight: 700;
}

.cc-circle-gray {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 999px;
    min-width: 28px;
    min-height: 28px;
    color: #ffffff;
    font-weight: 700;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-label-main {
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
}

.cc-label-sub {
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 11px;
    font-weight: 500;
    color: #94a3b8;
}

.cc-tile-slider {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 10px 14px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-tile-slider:hover {
    background: rgba(255, 255, 255, 0.08);
}

.cc-btn-active {
    background-color: rgba(10, 132, 255, 0.8);
    border: 1px solid rgba(10, 132, 255, 1.0);
}
.cc-btn-active .cc-label-main {
    color: #ffffff;
}

.cc-slider-icon {
    font-size: 15px;
    color: #f5f5f7;
}

.cc-quick-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 10px 6px;
    color: #f5f5f7;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-quick-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
    transform: translateY(-2px);
}

.cc-quick-btn:active {
    transform: translateY(1px);
}

.cc-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 8px 12px;
    color: #e2e8f0;
    font-weight: 500;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
}

.cc-btn-danger {
    background: rgba(255, 69, 58, 0.15);
    border: 1px solid rgba(255, 69, 58, 0.3);
    border-radius: 8px;
    padding: 8px 12px;
    color: #ff8a80;
    font-weight: 600;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-btn-danger:hover {
    background: rgba(255, 69, 58, 0.25);
    color: #ffffff;
}

progressbar.cc-progress-blue trough {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    min-height: 8px;
}
progressbar.cc-progress-blue progress {
    background: #0a84ff;
    border-radius: 6px;
    min-height: 8px;
}
progressbar.cc-progress-indigo trough {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    min-height: 8px;
}
progressbar.cc-progress-indigo progress {
    background: #5e5ce6;
    border-radius: 6px;
    min-height: 8px;
}
.applet-item {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 8px 12px;
    color: #f8fafc;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.metric-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 14px 16px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
.metric-value {
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 26px;
    font-weight: 800;
    color: #ffffff;
}
.pro-applet-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 10px 14px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
.applet-header-card {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 12px 16px;
}
.pro-applet-card-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 10px 14px;
    color: #ffffff;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
.pro-applet-card-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.15);
}
.wifi-pwd-entry {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 8px 12px;
    color: #ffffff;
    min-height: 38px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}
.wifi-pwd-entry:focus {
    border-color: rgba(255, 255, 255, 0.2);
    background: rgba(0, 0, 0, 0.4);
}
"#;

thread_local! {
    static CSS_PROVIDER: std::cell::RefCell<Option<CssProvider>> = std::cell::RefCell::new(None);
}

fn load_css() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let colors_path = format!("{}/.config/ermete-shell/colors.css", home);
    let colors_css = std::fs::read_to_string(&colors_path).unwrap_or_default();
    
    let fallback = if colors_css.is_empty() {
        r#"
        @define-color shell_bg alpha(#1c1c1e, 0.65);
        @define-color shell_fg #f5f5f7;
        @define-color shell_border alpha(white, 0.08);
        @define-color shell_hover alpha(white, 0.1);
        @define-color shell_primary #ffffff;
        @define-color popup_bg alpha(#1e1e20, 0.75);
        @define-color popup_border alpha(white, 0.08);
        @define-color btn_bg alpha(white, 0.05);
        @define-color btn_fg #ffffff;
        @define-color btn_hover alpha(white, 0.1);
        "#
    } else {
        ""
    };

    let full_css = format!("{}\n{}\n{}", colors_css, fallback, TOPBAR_CSS);

    CSS_PROVIDER.with(|p| {
        let mut provider_opt = p.borrow_mut();
        let display = gtk4::gdk::Display::default().unwrap_or_else(|| panic!("No display available"));
        
        if let Some(old_provider) = provider_opt.as_ref() {
            gtk4::style_context_remove_provider_for_display(&display, old_provider);
        }
        
        let new_provider = CssProvider::new();
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
        let mut watcher = match notify::recommended_watcher(tx) { Ok(w) => w, Err(e) => { eprintln!("Watcher error: {}", e); return; } };
        let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())).join(".config/ermete-shell");
        let _ = watcher.watch(&path, RecursiveMode::NonRecursive);
        
        while let Ok(event) = rx.recv() {
            if let Ok(ev) = event {
                if ev.kind.is_modify() {
                    let _ = sender.send(());
                }
            }
        }
    });

    receiver.attach(None, move |_| {
        load_css();
        glib::ControlFlow::Continue
    });
}

thread_local! {
    static ACTIVE_POPUP: std::cell::RefCell<Option<(String, glib::WeakRef<ApplicationWindow>)>> = std::cell::RefCell::new(None);
}

pub fn setup_popup_autoclose(pop: &ApplicationWindow, tag: &str) {
    let mut to_close = None;
    ACTIVE_POPUP.with(|p| {
        if let Some((_, old_weak)) = p.borrow().as_ref() {
            if let Some(old_win) = old_weak.upgrade() {
                if old_win != *pop && old_win.is_visible() {
                    to_close = Some(old_win);
                }
            }
        }
        *p.borrow_mut() = Some((tag.to_string(), pop.downgrade()));
    });

    if let Some(win) = to_close {
        win.close();
    }

    pop.set_keyboard_mode(KeyboardMode::OnDemand);
    pop.set_namespace(tag);

    if let Some(app) = pop.application() {
        let bg_win = ApplicationWindow::builder()
            .application(&app)
            .css_classes(["bg-overlay-window"])
            .build();
            
        bg_win.init_layer_shell();
        bg_win.set_namespace("bg-overlay");
        bg_win.set_layer(Layer::Top);
        bg_win.set_anchor(Edge::Top, true);
        bg_win.set_anchor(Edge::Bottom, true);
        bg_win.set_anchor(Edge::Left, true);
        bg_win.set_anchor(Edge::Right, true);
        bg_win.set_exclusive_zone(-1);
        bg_win.set_keyboard_mode(KeyboardMode::None);
        
        let empty_box = gtk4::Box::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        bg_win.set_child(Some(&empty_box));
        
        let click = gtk4::GestureClick::new();
        click.set_button(0); // Tutti i bottoni
        let pop_close_clone = pop.clone();
        click.connect_pressed(move |_, _, _, _| {
            pop_close_clone.close();
        });
        empty_box.add_controller(click);
        
        let bg_clone = bg_win.clone();
        pop.connect_close_request(move |win| {
            bg_clone.close();
            ACTIVE_POPUP.with(|p| {
                let mut clear = false;
                if let Some((_, old_weak)) = p.borrow().as_ref() {
                    if let Some(old_win) = old_weak.upgrade() {
                        if old_win == *win {
                            clear = true;
                        }
                    }
                }
                if clear {
                    *p.borrow_mut() = None;
                }
            });
            glib::Propagation::Proceed
        });
        
        bg_win.present();
    } else {
        pop.connect_close_request(move |win| {
            ACTIVE_POPUP.with(|p| {
                let mut clear = false;
                if let Some((_, old_weak)) = p.borrow().as_ref() {
                    if let Some(old_win) = old_weak.upgrade() {
                        if old_win == *win {
                            clear = true;
                        }
                    }
                }
                if clear {
                    *p.borrow_mut() = None;
                }
            });
            glib::Propagation::Proceed
        });
    }

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);
}

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
        load_css();
        spawn_css_watcher();
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

        let sender_fast = sender.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            sender_fast.input(TopbarInput::TickFast);
            glib::ControlFlow::Continue
        });

        let (niri_tx, niri_rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
        crate::core::spawn_niri_workspace_watcher(niri_tx);
        
        let sender_ws = sender.clone();
        niri_rx.attach(None, move |workspaces_data| {
            sender_ws.input(TopbarInput::UpdateWorkspaces(workspaces_data));
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
                
                let (net_icon, _, _) = crate::core::get_network_status();
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
    ACTIVE_POPUP.with(|p| {
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

