use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Grid, Label, Orientation, ScrolledWindow, Switch};
use std::path::{Path, PathBuf};
use std::process::Command;

fn is_dock_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "ermete-dock.service"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn scan_dir(dir: &Path, wallpapers: &mut Vec<PathBuf>, depth: usize) {
    if depth > 3 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, wallpapers, depth + 1);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ["png", "jpg", "jpeg", "webp", "gif"].contains(&ext_lower.as_str()) {
                    wallpapers.push(path);
                }
            }
        }
    }
}

fn scan_wallpapers() -> Vec<PathBuf> {
    let mut wallpapers = Vec::new();
    let dirs = ["/usr/share/backgrounds", "/usr/share/wallpapers"];
    for d in dirs {
        scan_dir(Path::new(d), &mut wallpapers, 0);
    }
    wallpapers.sort();
    wallpapers.dedup();

    if wallpapers.is_empty() {
        wallpapers.push(PathBuf::from("/usr/share/backgrounds/default.png"));
        wallpapers.push(PathBuf::from("/usr/share/backgrounds/ermete-wallpaper-1.jpg"));
        wallpapers.push(PathBuf::from("/usr/share/backgrounds/ermete-wallpaper-2.jpg"));
    }

    wallpapers
}

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // Title
    let title = Label::builder()
        .label("Desktop & Dock")
        .halign(Align::Start)
        .css_classes(["title-1", "large-title"])
        .build();

    container.append(&title);

    // Dock Section
    let dock_label_heading = Label::builder()
        .label("Dock")
        .halign(Align::Start)
        .css_classes(["heading"])
        .build();
    container.append(&dock_label_heading);

    let dock_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let dock_label = Label::builder()
        .label("Mostra Dock in basso")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let dock_switch = Switch::builder()
        .valign(Align::Center)
        .active(is_dock_active())
        .build();

    dock_switch.connect_state_set(|_, state| {
        if state {
            let _ = Command::new("systemctl")
                .args(["--user", "start", "ermete-dock.service"])
                .spawn();
            println!("Dock enabled (ermete-dock.service started)");
        } else {
            let _ = Command::new("systemctl")
                .args(["--user", "stop", "ermete-dock.service"])
                .spawn();
            println!("Dock disabled (ermete-dock.service stopped)");
        }
        gtk4::glib::Propagation::Proceed
    });

    dock_box.append(&dock_label);
    dock_box.append(&dock_switch);
    container.append(&dock_box);

    // Wallpaper Section
    let wallpaper_label = Label::builder()
        .label("Wallpaper")
        .halign(Align::Start)
        .css_classes(["heading"])
        .build();
    container.append(&wallpaper_label);

    let wallpaper_grid = Grid::builder()
        .column_spacing(12)
        .row_spacing(12)
        .build();

    let wallpapers = scan_wallpapers();
    let columns = 3;

    for (i, path) in wallpapers.iter().enumerate() {
        let col = (i % columns) as i32;
        let row = (i / columns) as i32;

        let label_text = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Wallpaper")
            .to_string();

        let btn = Button::builder()
            .label(&label_text)
            .width_request(180)
            .height_request(100)
            .tooltip_text(path.to_string_lossy().as_ref())
            .build();

        let path_clone = path.clone();
        btn.connect_clicked(move |_| {
            let abs_path = path_clone.to_string_lossy().to_string();
            let abs_path_clone = abs_path.clone();
            let ctx = gtk4::glib::MainContext::default();
            ctx.spawn_local(async move {
                if let Ok(conn) = crate::get_connection().await {
                    if let Ok(proxy) = crate::settings_proxy::SettingsProxy::new(&conn).await {
                        let _ = proxy.set_wallpaper(&abs_path_clone).await;
                    }
                }
            });
            println!("Wallpaper selected via D-Bus: {}", abs_path);
        });

        wallpaper_grid.attach(&btn, col, row, 1, 1);
    }

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&wallpaper_grid)
        .build();

    container.append(&scroll);

    container
}

