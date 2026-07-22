use notify::{Watcher, RecursiveMode};
use crate::core::*;
use crate::ui::spotlight::*;
use crate::ui::notifications::*;
use crate::ui::control_center::*;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CenterBox, CssProvider,
    Label, Orientation,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub const TOPBAR_CSS: &str = r#"
window.topbar-window {
    background-color: transparent;
}

window.bg-overlay-window {
    background-color: rgba(0, 0, 0, 0.01);
}

.topbar-container {
    background: @shell_bg;
    border-bottom: 1px solid @shell_border;
    color: @shell_fg;
    font-family: 'Inter', 'SF Pro Text', 'Roboto', sans-serif;
    font-size: 13px;
    font-weight: 500;
    padding: 0 10px;
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
    background: rgba(30, 30, 32, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 16px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6);
    animation: pop-in-fade 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
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
    background: rgba(30, 30, 32, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 16px;
    padding: 14px;
    color: #f8fafc;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6);
    animation: slide-down-fade 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-tile {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 10px;
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.cc-tile:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: scale(1.02);
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

fn load_css() {
    let home = std::env::var("HOME").unwrap();
    let colors_path = format!("{}/.config/ermete-shell/colors.css", home);
    let colors_css = std::fs::read_to_string(&colors_path).unwrap_or_default();
    
    // In caso il file di matugen non definisca nulla o sia vuoto, diamo dei fallback di sicurezza.
    let fallback = if colors_css.is_empty() {
        r#"
        @define-color shell_bg rgba(28, 28, 30, 0.65);
        @define-color shell_fg #f5f5f7;
        @define-color shell_border rgba(255, 255, 255, 0.08);
        @define-color shell_hover rgba(255, 255, 255, 0.1);
        @define-color shell_primary #ffffff;
        @define-color popup_bg rgba(30, 30, 32, 0.75);
        @define-color popup_border rgba(255, 255, 255, 0.08);
        @define-color btn_bg rgba(255, 255, 255, 0.05);
        @define-color btn_fg #ffffff;
        @define-color btn_hover rgba(255, 255, 255, 0.1);
        "#
    } else {
        ""
    };

    let full_css = format!("{}\n{}\n{}", colors_css, fallback, TOPBAR_CSS);

    CSS_PROVIDER.with(|p| {
        let mut provider_opt = p.borrow_mut();
        let display = gtk4::gdk::Display::default().unwrap();
        
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
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let path = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".config/ermete-shell");
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

fn build_left_island(app: &Application) -> (GtkBox, Button) {
    let box_left = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .valign(Align::Center)
        .build();

    let apple_logo = Button::builder()
        .label("◈")
        .css_classes(["macos-menu-item", "macos-apple-logo"])
        .build();
    let app_clone = app.clone();
    apple_logo.connect_clicked(move |_| {
        show_start_menu_popover(&app_clone);
    });
    crate::core::attach_voiceover_hover(&apple_logo, "Menu di avvio e sistema");
    box_left.append(&apple_logo);

    let app_title = Button::builder()
        .label("Ermete OS")
        .css_classes(["macos-menu-item", "macos-app-title"])
        .build();
    crate::core::attach_voiceover_hover(&app_title, "Nome dell'applicazione corrente");
    box_left.append(&app_title);

    (box_left, app_title)
}

fn build_center_island(_app: &Application) -> GtkBox {
    let workspace_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
        .build();

    let scroll_ctrl = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll_ctrl.connect_scroll(|_, _dx, dy| {
        if dy > 0.0 {
            crate::core::niri_client::focus_workspace_down();
        } else if dy < 0.0 {
            crate::core::niri_client::focus_workspace_up();
        }
        glib::Propagation::Stop
    });
    workspace_box.add_controller(scroll_ctrl);

    let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
    spawn_niri_workspace_watcher(sender);

    let workspace_box_clone = workspace_box.clone();
    receiver.attach(None, move |workspaces| {
        while let Some(child) = workspace_box_clone.first_child() {
            workspace_box_clone.remove(&child);
        }

        let active_output = workspaces.iter()
            .find(|w| w.is_focused)
            .or_else(|| workspaces.iter().find(|w| w.is_active))
            .map(|w| w.output.clone())
            .unwrap_or_default();

        let mut filtered_ws: Vec<_> = workspaces.into_iter().filter(|w| w.output == active_output).collect();
        filtered_ws.sort_by_key(|w| w.idx);

        for ws in filtered_ws {
            let label = if ws.is_active { "●" } else { "○" };
            let ws_btn = Button::builder()
                .label(label)
                .css_classes(["macos-menu-item"])
                .build();
            
            if ws.is_focused {
                ws_btn.add_css_class("workspace-focused");
            } else if ws.is_active {
                ws_btn.add_css_class("workspace-active");
            }

            let ws_id = ws.id;
            ws_btn.connect_clicked(move |_| {
                crate::core::niri_client::focus_workspace_by_id(ws_id);
            });

            workspace_box_clone.append(&ws_btn);
        }
        glib::ControlFlow::Continue
    });

    workspace_box
}

fn build_right_island(app: &Application, clock_label: &Label) -> (GtkBox, Button, Button) {
    let box_right = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .valign(Align::Center)
        .build();

    // 1. Battery / Power Dongle (macOS style)
    let batt_item = Button::builder()
        .label("100% 󰁹")
        .css_classes(["macos-status-item"])
        .build();
    crate::core::attach_voiceover_hover(&batt_item, "Stato batteria e alimentazione");

    // 2. Dynamic Network Dongle (macOS style: Ethernet/Wi-Fi/Off)
    let (init_icon, _, _) = get_network_status();
    let net_item = Button::builder()
        .label(&init_icon)
        .css_classes(["macos-status-item"])
        .build();
    let app_net = app.clone();
    net_item.connect_clicked(move |_| {
        show_wifi_popover(&app_net);
    });
    crate::core::attach_voiceover_hover(&net_item, "Connettività Wi-Fi e Rete");

    // 3. Spotlight Dongle (macOS style)
    let spot_item = Button::builder()
        .label("🔍")
        .css_classes(["macos-status-item"])
        .build();
    let app_clone1 = app.clone();
    spot_item.connect_clicked(move |_| {
        show_spotlight_modal(&app_clone1);
    });
    crate::core::attach_voiceover_hover(&spot_item, "Ricerca globale Spotlight");

    // 4. Control Center Dongle (macOS style)
    let cc_item = Button::builder()
        .label("❖")
        .css_classes(["macos-status-item"])
        .build();
    let app_clone2 = app.clone();
    cc_item.connect_clicked(move |_| {
        show_control_center_popover(&app_clone2);
    });
    crate::core::attach_voiceover_hover(&cc_item, "Centro di Controllo di sistema");

    // 5. Clock Dongle (macOS style)
    let clock_item = Button::builder()
        .css_classes(["macos-status-item", "macos-clock"])
        .build();
    clock_item.set_child(Some(clock_label));
    let app_clone3 = app.clone();
    clock_item.connect_clicked(move |_| {
        show_calendar_popover(&app_clone3);
    });
    crate::core::attach_voiceover_hover(&clock_item, "Orologio e Calendario");

    // 6. Notification Center Bell
    let notif_item = Button::builder()
        .label("󰂚")
        .css_classes(["macos-status-item"])
        .build();
    let app_clone_notif = app.clone();
    notif_item.connect_clicked(move |_| {
        toggle_or_open_popup("notifications", || crate::ui::notifications::show_notification_center(&app_clone_notif));
    });
    crate::core::attach_voiceover_hover(&notif_item, "Centro Notifiche");

    box_right.append(&batt_item);
    box_right.append(&net_item);
    box_right.append(&spot_item);
    box_right.append(&cc_item);
    box_right.append(&notif_item);
    box_right.append(&clock_item);
    (box_right, net_item, batt_item)
}

pub fn build_ui(app: &Application) {
    if UI_BUILT.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    load_css();
    spawn_css_watcher();
    spawn_notification_daemon(app);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Shell")
        .css_classes(["topbar-window"])
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace("bar");
    window.auto_exclusive_zone_enable();

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // macOS Sonoma / Sequoia height = 28px exactly
    window.set_height_request(28);

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["topbar-container"])
        .hexpand(true)
        .build();

    let clock_label = Label::new(Some(&macos_clock_string()));

    let center_box = CenterBox::new();
    let (left_island, app_title) = build_left_island(app);
    center_box.set_start_widget(Some(&left_island));
    center_box.set_center_widget(Some(&build_center_island(app)));
    let (right_island, net_btn, batt_btn) = build_right_island(app, &clock_label);
    center_box.set_end_widget(Some(&right_island));
    center_box.set_hexpand(true);

    container.append(&center_box);
    window.set_child(Some(&container));

    glib::timeout_add_seconds_local(
        5,
        clone!(@weak clock_label, @weak net_btn, @weak batt_btn => @default-return glib::ControlFlow::Break, move || {
            clock_label.set_label(&macos_clock_string());
            let (net_icon, _, _) = get_network_status();
            net_btn.set_label(&net_icon);
            
            let live = crate::core::live_state::get_live_state();
            if live.has_battery {
                batt_btn.set_visible(true);
                let batt_icon = if live.battery_percent < 20.0 {
                    "󰁺"
                } else if live.battery_percent < 50.0 {
                    "󰁼"
                } else {
                    "󰁹"
                };
                batt_btn.set_label(&format!("{}% {}", live.battery_percent.round() as i32, batt_icon));
            } else {
                batt_btn.set_visible(false);
            }
            
            glib::ControlFlow::Continue
        }),
    );

    // Fast Polling for Niri Window Focus (Every 100ms for snappiness)
    glib::timeout_add_local(
        std::time::Duration::from_millis(200),
        clone!(@weak app_title => @default-return glib::ControlFlow::Break, move || {
            let niri = crate::core::niri_state::get_niri_state();
            if let Some(title) = niri.focused_window_title {
                app_title.set_label(&title);
            } else {
                app_title.set_label("Ermete OS");
            }
            glib::ControlFlow::Continue
        }),
    );

    window.present();
}

#[allow(dead_code)]
const APP_ID: &str = "os.ermete.Shell";

#[allow(dead_code)]
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
        win.close();
    }

    if !already_open {
        open_fn();
    }
}

#[allow(dead_code)]
static UI_BUILT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[allow(dead_code)]
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
