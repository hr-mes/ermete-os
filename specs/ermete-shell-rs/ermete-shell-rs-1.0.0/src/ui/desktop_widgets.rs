use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, Fixed, Box, Label, Orientation, Align, GestureDrag};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::cell::Cell;
use std::rc::Rc;

// --- Config Models ---

#[derive(Debug, Deserialize, Serialize)]
pub struct WidgetConfig {
    pub id: String,
    pub widget_type: String,
    pub x: f64,
    pub y: f64,
    pub settings: Option<serde_json::Value>,
    pub custom_css: Option<String>,
    pub plugin_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DesktopConfig {
    pub widgets: Vec<WidgetConfig>,
}

// --- Configuration Manager ---

fn get_config_path() -> PathBuf {
    let mut path = glib::user_config_dir();
    path.push("ermete");
    path.push("widgets.json");
    path
}

fn load_config() -> DesktopConfig {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_else(|_| default_config())
    } else {
        let default = default_config();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&default) {
            let _ = fs::write(&path, content);
        }
        default
    }
}

fn default_config() -> DesktopConfig {
    DesktopConfig {
        widgets: vec![
            WidgetConfig {
                id: "main_clock".to_string(),
                widget_type: "clock".to_string(),
                x: 80.0,
                y: 80.0,
                settings: None,
            },
            WidgetConfig {
                id: "main_system".to_string(),
                widget_type: "system".to_string(),
                x: 80.0,
                y: 250.0,
                settings: None,
            }
        ],
    }
}

fn update_widget_position(target_id: &str, new_x: f64, new_y: f64) {
    let mut config = load_config();
    let mut updated = false;
    for w in config.widgets.iter_mut() {
        if w.id == target_id {
            w.x = new_x;
            w.y = new_y;
            updated = true;
            break;
        }
    }
    if updated {
        let path = get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(&config) {
            let _ = fs::write(&path, content);
        }
    }
}

// --- OS Stats Readers ---

fn get_memory_usage() -> (u64, u64) {
    let mut total: u64 = 0;
    let mut available: u64 = 0;
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                total = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
            } else if line.starts_with("MemAvailable:") || line.starts_with("MemFree:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                available = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
            }
        }
    }
    let used = total.saturating_sub(available) / 1024;
    (used, total / 1024)
}

fn get_cpu_times() -> (u64, u64) {
    if let Ok(content) = fs::read_to_string("/proc/stat") {
        if let Some(line) = content.lines().next() {
            if line.starts_with("cpu ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut total = 0;
                let mut idle = 0;
                for (i, part) in parts.iter().skip(1).enumerate() {
                    if let Ok(val) = part.parse::<u64>() {
                        total += val;
                        if i == 3 || i == 4 { // idle or iowait
                            idle += val;
                        }
                    }
                }
                return (idle, total);
            }
        }
    }
    (0, 0)
}

// --- Widgets ---

fn build_system_widget() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Center)
        .css_classes(vec!["desktop-widget"])
        .build();

    let title = Label::builder()
        .label("Hardware")
        .css_classes(vec!["subtitle"])
        .halign(Align::Start)
        .build();

    let cpu_label = Label::builder()
        .css_classes(vec!["title-1"])
        .halign(Align::Start)
        .build();

    let ram_label = Label::builder()
        .css_classes(vec!["subtitle"])
        .halign(Align::Start)
        .build();

    container.append(&title);
    container.append(&cpu_label);
    container.append(&ram_label);

    let prev_cpu = Rc::new(Cell::new(get_cpu_times()));
    
    let update_sys = {
        let cpu_label = cpu_label.clone();
        let ram_label = ram_label.clone();
        let prev_cpu = prev_cpu.clone();
        move || {
            let (mem_used, mem_total) = get_memory_usage();
            let mem_percent = if mem_total > 0 { (mem_used as f64 / mem_total as f64) * 100.0 } else { 0.0 };
            ram_label.set_label(&format!("RAM: {:.1}% ({}/{} MB)", mem_percent, mem_used, mem_total));

            let (idle, total) = get_cpu_times();
            let (prev_idle, prev_total) = prev_cpu.get();
            let total_diff = total.saturating_sub(prev_total);
            let idle_diff = idle.saturating_sub(prev_idle);
            let cpu_usage = if total_diff > 0 {
                100.0 * (total_diff as f64 - idle_diff as f64) / total_diff as f64
            } else {
                0.0
            };
            cpu_label.set_label(&format!("CPU: {:.1}%", cpu_usage));
            prev_cpu.set((idle, total));

            glib::ControlFlow::Continue
        }
    };

    update_sys();
    glib::timeout_add_seconds_local(2, update_sys);

    container
}

fn build_clock_widget() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Center)
        .css_classes(vec!["desktop-widget"])
        .build();

    let time_label = Label::builder()
        .css_classes(vec!["title-1"])
        .halign(Align::Start)
        .build();

    let date_label = Label::builder()
        .css_classes(vec!["subtitle"])
        .halign(Align::Start)
        .build();

    let update_time = {
        let time_label = time_label.clone();
        let date_label = date_label.clone();
        move || {
            let now = Local::now();
            time_label.set_label(&now.format("%H:%M").to_string());
            date_label.set_label(&now.format("%A, %d %B").to_string());
            glib::ControlFlow::Continue
        }
    };

    update_time();
    glib::timeout_add_seconds_local(1, update_time);

    container.append(&time_label);
    container.append(&date_label);

    container
}

// --- Interactivity ---

fn make_widget_draggable(widget: &Box, canvas: &Fixed, widget_id: String, initial_x: f64, initial_y: f64) {
    let gesture = GestureDrag::new();
    let start_pos = Rc::new(Cell::new((initial_x, initial_y)));
    
    let widget_clone = widget.clone();
    let canvas_clone = canvas.clone();
    let start_pos_clone = start_pos.clone();
    
    gesture.connect_drag_update(move |_, offset_x, offset_y| {
        let (sx, sy) = start_pos_clone.get();
        let new_x = sx + offset_x;
        let new_y = sy + offset_y;
        canvas_clone.move_(&widget_clone, new_x, new_y);
    });

    let widget_id_clone = widget_id.clone();
    gesture.connect_drag_end(move |_, offset_x, offset_y| {
        let (sx, sy) = start_pos.get();
        let final_x = sx + offset_x;
        let final_y = sy + offset_y;
        start_pos.set((final_x, final_y));
        update_widget_position(&widget_id_clone, final_x, final_y);
    });

    widget.add_controller(gesture);
}

// --- Main Engine ---

fn reload_widgets(canvas: &Fixed) {
    while let Some(child) = canvas.first_child() {
        canvas.remove(&child);
    }

    let config = load_config();
    for widget_cfg in config.widgets {
        match widget_cfg.widget_type.as_str() {
            "clock" => {
                let widget = build_clock_widget();
                make_widget_draggable(&widget, canvas, widget_cfg.id.clone(), widget_cfg.x, widget_cfg.y);
                canvas.put(&widget, widget_cfg.x, widget_cfg.y);
            }
            "system" => {
                let widget = build_system_widget();
                make_widget_draggable(&widget, canvas, widget_cfg.id.clone(), widget_cfg.x, widget_cfg.y);
                canvas.put(&widget, widget_cfg.x, widget_cfg.y);
            }
            other => {
                eprintln!("Unknown widget type requested: {}", other);
            }
        }
    }
}

pub fn build_desktop_widgets(app: &Application) {
    let provider = CssProvider::new();
    provider.load_from_data(
        "
        window.desktop-overlay {
            background-color: transparent;
        }
        .desktop-widget {
            background-color: rgba(20, 20, 20, 0.45);
            border-radius: 24px;
            border: 1px solid rgba(255, 255, 255, 0.12);
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
            padding: 24px;
            color: white;
            font-family: 'Inter', sans-serif;
        }
        .desktop-widget .title-1 {
            font-size: 48px;
            font-weight: 800;
        }
        .desktop-widget .subtitle {
            font-size: 18px;
            font-weight: 500;
            color: rgba(255, 255, 255, 0.8);
        }
        "
    );
    
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desktop Widgets")
        .css_classes(vec!["desktop-overlay"])
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Bottom);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    let canvas = Fixed::new();

    reload_widgets(&canvas);

    let file = gtk4::gio::File::for_path(get_config_path());
    if let Ok(monitor) = file.monitor_file(gtk4::gio::FileMonitorFlags::NONE, gtk4::gio::Cancellable::NONE) {
        let canvas_clone = canvas.clone();
        monitor.connect_changed(move |_, _, _, event_type| {
            if event_type == gtk4::gio::FileMonitorEvent::Changed || event_type == gtk4::gio::FileMonitorEvent::Created {
                reload_widgets(&canvas_clone);
            }
        });
        std::boxed::Box::leak(std::boxed::Box::new(monitor));
    }

    window.set_child(Some(&canvas));
    window.present();
}
