// Mission Control spatial overlay
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, FlowBox, Label, Orientation, Picture, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::core::dock_watcher::{fetch_current_niri_windows, fetch_current_workspaces};
use crate::ui::popup_manager::setup_popup_autoclose;

const MISSION_CONTROL_CSS: &str = r#"
window.mission-control-window {
    background-color: rgba(12, 14, 22, 0.88);
}

button.mc-window-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 16px;
    padding: 8px;
    transition: all 200ms cubic-bezier(0.25, 1, 0.5, 1);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}

button.mc-window-card:hover {
    background: rgba(255, 255, 255, 0.10);
    border-color: rgba(99, 102, 241, 0.6);
    box-shadow: 0 12px 32px rgba(99, 102, 241, 0.25);
}

button.mc-window-card:active {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(129, 140, 248, 0.9);
}

picture.mc-window-preview {
    border-radius: 10px;
    background-color: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

label.mc-window-title {
    color: #f3f4f6;
    font-weight: 600;
    font-size: 14px;
}

label.mc-app-id {
    color: #9ca3af;
    font-size: 11px;
    font-weight: 500;
}

button.mc-workspace-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    color: #d1d5db;
    padding: 6px 16px;
    font-weight: 500;
    transition: all 150ms ease;
}

button.mc-workspace-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

button.mc-workspace-btn.active {
    background: linear-gradient(135deg, #4f46e5, #7c3aed);
    color: #ffffff;
    border-color: rgba(167, 139, 250, 0.6);
    box-shadow: 0 4px 14px rgba(79, 70, 229, 0.4);
}

button.mc-workspace-action {
    background: transparent;
    border: none;
    color: #9ca3af;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 6px;
}

button.mc-workspace-action:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #e5e7eb;
}
"#;

/// Asynchronous dummy abstraction to fetch window GPU preview thumbnails.
/// In production, this can be wired to Screencopy / DMA-BUF buffers.
/// Currently renders a high quality 16:9 procedural gradient preview buffer.
pub async fn fetch_window_thumbnail(app_id: &str, window_id: u64) -> Option<gdk::Texture> {
    let width = 320;
    let height = 180;
    let mut pixels = Vec::with_capacity(width * height * 4);

    // Compute base colors based on app_id and window_id hash
    let mut hash = window_id.wrapping_add(14695981039346656037);
    for b in app_id.bytes() {
        hash = hash.wrapping_mul(1099511628211) ^ (b as u64);
    }

    let r1 = ((hash & 0xFF) as f32 / 255.0) * 0.7 + 0.15;
    let g1 = (((hash >> 8) & 0xFF) as f32 / 255.0) * 0.7 + 0.15;
    let b1 = (((hash >> 16) & 0xFF) as f32 / 255.0) * 0.7 + 0.15;

    let r2 = (((hash >> 24) & 0xFF) as f32 / 255.0) * 0.7 + 0.2;
    let g2 = (((hash >> 32) & 0xFF) as f32 / 255.0) * 0.7 + 0.2;
    let b2 = (((hash >> 40) & 0xFF) as f32 / 255.0) * 0.7 + 0.2;

    for y in 0..height {
        let vy = y as f32 / height as f32;
        for x in 0..width {
            let vx = x as f32 / width as f32;

            // Simulated window titlebar (top 22px dark bar with window frame border)
            if y < 22 {
                if y < 20 {
                    pixels.push(30);
                    pixels.push(33);
                    pixels.push(42);
                    pixels.push(255);
                } else {
                    pixels.push(50);
                    pixels.push(55);
                    pixels.push(70);
                    pixels.push(255);
                }
                continue;
            }

            // High-quality smooth spatial gradient
            let blend_x = (vx * std::f32::consts::PI).sin();
            let blend_y = (vy * std::f32::consts::PI).sin();
            let factor = (blend_x * 0.5 + blend_y * 0.5).clamp(0.0, 1.0);

            let r = (r1 * (1.0 - factor) + r2 * factor) * 255.0;
            let g = (g1 * (1.0 - factor) + g2 * factor) * 255.0;
            let b = (b1 * (1.0 - factor) + b2 * factor) * 255.0;

            pixels.push(r as u8);
            pixels.push(g as u8);
            pixels.push(b as u8);
            pixels.push(255);
        }
    }

    let bytes = glib::Bytes::from_owned(pixels);
    let stride = width * 4;
    let memory_texture = gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        gdk::MemoryFormat::R8g8b8a8Premultiplied,
        &bytes,
        stride as usize,
    );

    Some(memory_texture.upcast())
}

pub fn build_ui(app: &Application) {
    if let Some(display) = gdk::Display::default() {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(MISSION_CONTROL_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

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

    // Full screen overlay margins
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
                .arg(ws_id.to_string())
                .spawn();
            win_clone.close();
        });
        ws_box.append(&ws_container);
    }
    main_box.append(&ws_box);

    // 2. Spatial Exposé Windows Grid with Live GPU Previews (16:9 Layout)
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
        .max_children_per_line(4)
        .min_children_per_line(1)
        .row_spacing(32)
        .column_spacing(32)
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    for win_info in windows {
        let app_id = win_info.app_id.unwrap_or_else(|| "unknown".to_string());
        let title = win_info.title.unwrap_or_else(|| "Unknown Window".to_string());

        let card_btn = Button::builder()
            .css_classes(["mc-window-card"])
            .build();

        let card_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        // 16:9 Picture preview widget for live GPU texture buffers
        let picture = Picture::builder()
            .can_shrink(true)
            .keep_aspect_ratio(true)
            .width_request(280)
            .height_request(158)
            .css_classes(["mc-window-preview"])
            .build();

        // Header info row
        let header_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let lbl = Label::builder()
            .label(&title)
            .css_classes(["mc-window-title"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(22)
            .halign(Align::Start)
            .hexpand(true)
            .build();

        let app_lbl = Label::builder()
            .label(&app_id)
            .css_classes(["mc-app-id"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(14)
            .halign(Align::End)
            .build();

        header_box.append(&lbl);
        header_box.append(&app_lbl);

        card_box.append(&header_box);
        card_box.append(&picture);
        card_btn.set_child(Some(&card_box));

        // Asynchronously fetch thumbnail and bind GPU buffer texture to Picture
        let app_id_clone = app_id.clone();
        let win_id = win_info.id;
        let picture_clone = picture.clone();

        glib::MainContext::default().spawn_local(async move {
            if let Some(texture) = fetch_window_thumbnail(&app_id_clone, win_id).await {
                picture_clone.set_paintable(Some(&texture));
            }
        });

        let win_clone = window.clone();
        card_btn.connect_clicked(move |_| {
            let _ = std::process::Command::new("niri")
                .arg("msg")
                .arg("action")
                .arg("focus-window")
                .arg("--id")
                .arg(win_id.to_string())
                .spawn();
            win_clone.close();
        });

        flowbox.insert(&card_btn, -1);
    }

    scroll.set_child(Some(&flowbox));
    main_box.append(&scroll);

    window.set_child(Some(&main_box));
    window.present();
}

