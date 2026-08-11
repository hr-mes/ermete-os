use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, ProgressBar};
use crate::components::action_row::ActionRow;

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

    let settings_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["liquid-surface"])
        .build();

    // Battery progress bar
    let progress_bar = ProgressBar::builder()
        .fraction(0.0)
        .show_text(true)
        .text("Caricamento...")
        .width_request(200)
        .valign(Align::Center)
        .build();

    let battery_row = ActionRow::builder("Livello Batteria")
        .subtitle("Stato di carica e alimentazione")
        .suffix(&progress_bar)
        .build();

    settings_card.append(&battery_row);

    // Asynchronously fetch capacity via UPower DBus proxy (Zero-Trust compliant)
    let progress_bar_clone = progress_bar.clone();
    relm4::spawn_local(async move {
        let mut capacity: Option<u32> = None;
        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(msg) = conn.call_method(
                Some("org.freedesktop.UPower"),
                "/org/freedesktop/UPower/devices/DisplayDevice",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.UPower.Device", "Percentage"),
            ).await {
                if let Ok(val) = msg.body().deserialize::<zbus::zvariant::OwnedValue>() {
                    if let Ok(pct) = f64::try_from(val) {
                        capacity = Some(pct.round() as u32);
                    }
                }
            }
        }

        let (fraction, progress_text) = match capacity {
            Some(cap) => {
                let cap = cap.min(100);
                (cap as f64 / 100.0, format!("{}%", cap))
            }
            None => (0.0, "N/D (AC)".to_string()),
        };

        progress_bar_clone.set_fraction(fraction);
        progress_bar_clone.set_text(Some(&progress_text));
    });

    let separator = gtk4::Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    settings_card.append(&separator);

    // Power Profiles Buttons
    let profiles_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
        .build();

    let btn_performance = Button::with_label("Prestazioni");
    btn_performance.connect_clicked(|_| {
        relm4::spawn_local(async move {
            let _ = tokio::process::Command::new("powerprofilesctl")
                .args(["set", "performance"])
                .status()
                .await;
        });
    });

    let btn_balanced = Button::with_label("Bilanciato");
    btn_balanced.connect_clicked(|_| {
        relm4::spawn_local(async move {
            let _ = tokio::process::Command::new("powerprofilesctl")
                .args(["set", "balanced"])
                .status()
                .await;
        });
    });

    let btn_power_saver = Button::with_label("Risparmio Energetico");
    btn_power_saver.connect_clicked(|_| {
        relm4::spawn_local(async move {
            let _ = tokio::process::Command::new("powerprofilesctl")
                .args(["set", "power-saver"])
                .status()
                .await;
        });
    });

    profiles_box.append(&btn_performance);
    profiles_box.append(&btn_balanced);
    profiles_box.append(&btn_power_saver);

    let profiles_row = ActionRow::builder("Profili Energetici")
        .subtitle("Seleziona la modalità di gestione delle prestazioni e dei consumi")
        .suffix(&profiles_box)
        .build();

    settings_card.append(&profiles_row);

    container.append(&settings_card);

    container
}
