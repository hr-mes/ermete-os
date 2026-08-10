use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Grid, Label, Orientation, ScrolledWindow, Switch};
use std::path::{Path, PathBuf};
use crate::components::action_row::ActionRow;

async fn scan_dir_async(dir: &Path, wallpapers: &mut Vec<PathBuf>, depth: usize) {
    if depth > 3 {
        return;
    }
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                std::boxed::Box::pin(scan_dir_async(&path, wallpapers, depth + 1)).await;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ["png", "jpg", "jpeg", "webp", "gif"].contains(&ext_lower.as_str()) {
                    wallpapers.push(path);
                }
            }
        }
    }
}

async fn scan_wallpapers_async() -> Vec<PathBuf> {
    let mut wallpapers = Vec::new();
    let dirs = ["/usr/share/backgrounds", "/usr/share/wallpapers"];
    for d in dirs {
        scan_dir_async(Path::new(d), &mut wallpapers, 0).await;
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

    // Dock Section inside Card
    let dock_card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["card"])
        .build();

    let dock_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    let dock_switch_clone = dock_switch.clone();
    relm4::spawn_local(async move {
        let is_active = tokio::process::Command::new("systemctl")
            .args(["--user", "is-active", "--quiet", "ermete-dock.service"])
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false);
        dock_switch_clone.set_active(is_active);
    });

    dock_switch.connect_state_set(|_, state| {
        relm4::spawn_local(async move {
            let action = if state { "start" } else { "stop" };
            let _ = tokio::process::Command::new("systemctl")
                .args(["--user", action, "ermete-dock.service"])
                .status()
                .await;
            let _ = crate::crdt_store::update_setting_crdt("dock_enabled", &state.to_string()).await;
        });
        gtk4::glib::Propagation::Proceed
    });

    let dock_row = ActionRow::builder("Mostra Dock in basso")
        .subtitle("Abilita la barra delle applicazioni Ermete Dock")
        .suffix(&dock_switch)
        .build();

    dock_card.append(&dock_row);
    container.append(&dock_card);

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

    let wallpaper_grid_clone = wallpaper_grid.clone();
    relm4::spawn_local(async move {
        let wallpapers = scan_wallpapers_async().await;
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
                let abs_path = path_clone.to_string_lossy().into_owned();
                let abs_path_clone = abs_path.clone();
                let abs_path_crdt = abs_path.clone();
                relm4::spawn_local(async move {
                    if let Ok(conn) = crate::get_connection().await {
                        if let Ok(proxy) = crate::settings_proxy::SettingsProxy::new(&conn).await {
                            let _ = proxy.set_wallpaper(&abs_path_clone).await;
                        }
                    }
                    crate::crdt_store::update_wallpaper_crdt(&abs_path_crdt).await;
                });
            });

            wallpaper_grid_clone.attach(&btn, col, row, 1, 1);
        }
    });

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .child(&wallpaper_grid)
        .build();

    container.append(&scroll);

    container
}
