use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, ProgressBar};
use std::process::Command;

fn get_battery_capacity() -> Option<u32> {
    std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // Title
    let title = Label::builder()
        .label("Batteria ed Energia")
        .halign(Align::Start)
        .build();
    title.add_css_class("title-1");
    container.append(&title);

    let (label_text, fraction, progress_text) = match get_battery_capacity() {
        Some(cap) => {
            let cap = cap.min(100);
            (
                format!("Livello Batteria Attuale: {}%", cap),
                cap as f64 / 100.0,
                format!("{}%", cap),
            )
        }
        None => (
            "Livello Batteria Attuale: N/D (Alimentazione AC)".to_string(),
            0.0,
            "N/D".to_string(),
        ),
    };

    // Battery progress
    let battery_label = Label::builder()
        .label(&label_text)
        .halign(Align::Start)
        .margin_top(12)
        .build();
    container.append(&battery_label);

    let progress_bar = ProgressBar::builder()
        .fraction(fraction)
        .show_text(true)
        .text(&progress_text)
        .build();
    container.append(&progress_bar);

    // Power Profiles Label
    let profiles_label = Label::builder()
        .label("Profili Energetici")
        .halign(Align::Start)
        .margin_top(24)
        .build();
    profiles_label.add_css_class("title-2");
    container.append(&profiles_label);

    // Power Profiles Buttons
    let profiles_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();

    let btn_performance = Button::with_label("Prestazioni");
    btn_performance.connect_clicked(|_| {
        Command::new("powerprofilesctl")
            .arg("set")
            .arg("performance")
            .spawn()
            .ok();
    });

    let btn_balanced = Button::with_label("Bilanciato");
    btn_balanced.connect_clicked(|_| {
        Command::new("powerprofilesctl")
            .arg("set")
            .arg("balanced")
            .spawn()
            .ok();
    });

    let btn_power_saver = Button::with_label("Risparmio Energetico");
    btn_power_saver.connect_clicked(|_| {
        Command::new("powerprofilesctl")
            .arg("set")
            .arg("power-saver")
            .spawn()
            .ok();
    });

    profiles_box.append(&btn_performance);
    profiles_box.append(&btn_balanced);
    profiles_box.append(&btn_power_saver);

    container.append(&profiles_box);

    container
}
