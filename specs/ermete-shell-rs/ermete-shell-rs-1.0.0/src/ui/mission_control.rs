// Mission Control spatial overlay
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, FlowBox, Image, Label, Orientation, ScrolledWindow, Widget};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::core::dock_watcher::{fetch_current_niri_windows, fetch_current_workspaces};
use crate::ui::topbar::setup_popup_autoclose;

fn get_icon_for_app_id(app_id: &str) -> String {
    let clean = app_id.trim_end_matches(".desktop").to_lowercase();
    if gtk4::gio::Icon::for_string(&clean).is_ok() {
        return clean;
    }
    // Fallback attempts
    if clean.contains("firefox") { return "firefox".to_string(); }
    if clean.contains("terminal") { return "utilities-terminal".to_string(); }
    if clean.contains("nautilus") || clean.contains("files") { return "system-file-manager".to_string(); }
    "application-x-executable".to_string()
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mission Control")
        .css_classes(["mission-control-window"])
        .build();

    window.init_layer_shell();
    window.set_namespace("overview");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    window.auto_exclusive_zone_enable();
    
    // Full screen
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Bottom, 0);
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);
    
    setup_popup_autoclose(&window, "overview");

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(32)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(48)
        .margin_end(48)
        .build();

    // 1. Workspaces Strip (Top)
    let workspaces = fetch_current_workspaces();
    let ws_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .halign(Align::Center)
        .build();

    for ws in &workspaces {
        let ws_container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .build();
            
        let ws_name = ws.name.clone().unwrap_or_else(|| format!("Desktop {}", ws.idx));
        
        let ws_btn = Button::builder()
            .label(&ws_name)
            .css_classes(if ws.is_active { vec!["mc-workspace-btn", "active"] } else { vec!["mc-workspace-btn"] })
            .build();
            
        let controls_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .halign(Align::Center)
            .build();
            
        let rename_btn = Button::builder().label("Rename").css_classes(["mc-workspace-action"]).build();
        let wallpaper_btn = Button::builder().label("Wallpaper").css_classes(["mc-workspace-action"]).build();
        let rules_btn = Button::builder().label("Rules").css_classes(["mc-workspace-action"]).build();
        
        controls_box.append(&rename_btn);
        controls_box.append(&wallpaper_btn);
        controls_box.append(&rules_btn);
        
        ws_container.append(&ws_btn);
        ws_container.append(&controls_box);
        
        let ws_id = ws.id;
        let win_clone = window.clone();
        ws_btn.connect_clicked(move |_| {
            let _ = std::process::Command::new("niri")
                .arg("msg")
                .arg("action")
                .arg("focus-workspace")
                .arg("--id")
                .arg(&ws_id.to_string())
                .spawn();
            win_clone.close();
        });
        ws_box.append(&ws_container);
    }
    main_box.append(&ws_box);

    // 2. Windows Grid
    let windows = fetch_current_niri_windows();
    
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .build();

    let flowbox = FlowBox::builder()
        .valign(Align::Start)
        .halign(Align::Center)
        .max_children_per_line(5)
        .min_children_per_line(1)
        .row_spacing(32)
        .column_spacing(32)
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    for win_info in windows {
        let app_id = win_info.app_id.unwrap_or_else(|| "unknown".to_string());
        let title = win_info.title.unwrap_or_else(|| "Unknown Window".to_string());
        let icon_name = get_icon_for_app_id(&app_id);

        let card_btn = Button::builder()
            .css_classes(["mc-window-card"])
            .build();

        let card_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        let img = Image::builder()
            .icon_name(&icon_name)
            .pixel_size(128)
            .build();

        let lbl = Label::builder()
            .label(&title)
            .css_classes(["mc-window-title"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(25)
            .justify(gtk4::Justification::Center)
            .build();

        card_box.append(&img);
        card_box.append(&lbl);
        card_btn.set_child(Some(&card_box));

        let win_id = win_info.id;
        let win_clone = window.clone();
        card_btn.connect_clicked(move |_| {
            let _ = std::process::Command::new("niri")
                .arg("msg")
                .arg("action")
                .arg("focus-window")
                .arg("--id")
                .arg(&win_id.to_string())
                .spawn();
            win_clone.close();
        });

        flowbox.insert(&card_btn, -1);
    }

    scroll.set_child(Some(&flowbox));
    main_box.append(&scroll);

    window.set_child(Some(&main_box));

    // Basic CSS
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        .mission-control-window { background-color: rgba(17, 17, 27, 0.85); }
        .mc-ws-btn { padding: 8px 24px; border-radius: 12px; font-weight: bold; background-color: rgba(255, 255, 255, 0.1); color: #cdd6f4; }
        .mc-ws-btn.active { background-color: rgba(10, 132, 255, 0.8); color: white; border: 2px solid #89b4fa; }
        .mc-ws-btn:hover { background-color: rgba(255, 255, 255, 0.2); }
        .mc-window-card { background-color: rgba(30, 30, 46, 0.7); border-radius: 24px; border: 1px solid #313244; transition: all 200ms ease; }
        .mc-window-card:hover { background-color: rgba(49, 50, 68, 0.9); border: 1px solid #89b4fa; }
        .mc-window-title { font-size: 16px; font-weight: bold; color: #cdd6f4; margin-top: 8px; }
    ");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}
