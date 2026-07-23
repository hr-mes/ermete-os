use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation};
use std::fs;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    let title = Label::builder()
        .label("Rete Cablata")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();
    container.append(&title);

    let status_str = get_ethernet_status();
    let status_label = Label::builder()
        .label(&status_str)
        .halign(Align::Start)
        .css_classes(["heading"])
        .build();
    container.append(&status_label);

    let proxy_button = Button::builder()
        .label("Configura Proxy")
        .halign(Align::Start)
        .build();
    proxy_button.connect_clicked(|_| {
        println!("Azione dummy: Configura Proxy cliccato");
    });
    container.append(&proxy_button);

    container
}

fn get_ethernet_status() -> String {
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if (name.starts_with("eth") || name.starts_with("en")) && !name.starts_with("enx") && !name.starts_with("lo") {
                let state_path = format!("/sys/class/net/{}/operstate", name);
                if let Ok(state) = fs::read_to_string(state_path) {
                    let st = state.trim();
                    let st_label = match st {
                        "up" => "Connesso",
                        "down" => "Scollegato",
                        "unknown" => "Stato sconosciuto",
                        other => other,
                    };
                    return format!("{} - {}", name, st_label);
                }
            }
        }
    }
    "Nessuna rete cablata rilevata".to_string()
}
