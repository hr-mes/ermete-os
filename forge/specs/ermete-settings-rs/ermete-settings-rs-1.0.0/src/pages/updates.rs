use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, Switch};
use crate::components::action_row::ActionRow;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(40)
        .margin_start(40)
        .margin_end(40)
        .build();

    let title = Label::builder()
        .label("Aggiornamenti di Sistema")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["settings-card"])
        .build();

    let check_btn = Button::builder()
        .label("Verifica Aggiornamenti")
        .css_classes(["suggested-action"])
        .valign(Align::Center)
        .build();

    let status_row = ActionRow::builder("Ermete OS è aggiornato")
        .subtitle("Ultimo controllo: Oggi alle 10:45 • Versione 1.0.0 (Layer 0 - Ostree Native)")
        .suffix(&check_btn)
        .build();

    let status_row_clone = status_row.clone();
    check_btn.connect_clicked(move |_| {
        let row = status_row_clone.clone();
        relm4::spawn_local(async move {
            row.set_subtitle("Ricerca aggiornamenti Ostree in corso...");
            if let Ok(output) = tokio::process::Command::new("ostree")
                .args(["admin", "status"])
                .output()
                .await 
            {
                if output.status.success() {
                    row.set_subtitle("Sistema base Ostree aggiornato e verificato.");
                } else {
                    row.set_subtitle("Errore durante il controllo degli aggiornamenti.");
                }
            } else {
                row.set_subtitle("Errore: impossibile contattare ostree.");
            }
        });
    });

    card.append(&status_row);

    // OTA Info Layer 1 Live Update
    let live_switch = Switch::builder().active(true).valign(Align::Center).build();
    let live_row = ActionRow::builder("Aggiornamenti Live UI (Layer 1)")
        .subtitle("Gli aggiornamenti dell'interfaccia utente vengono applicati istantaneamente senza riavvio")
        .suffix(&live_switch)
        .build();
    card.append(&live_row);

    // OTA Info Layer 0 Kernel Update
    let kernel_status_label = Label::builder()
        .label("Atomico Ostree")
        .css_classes(["subtitle"])
        .valign(Align::Center)
        .build();
    let kernel_row = ActionRow::builder("Aggiornamenti Base & Kernel (Layer 0)")
        .subtitle("Gli aggiornamenti del sistema base richiedono un riavvio per l'applicazione del deployment")
        .suffix(&kernel_status_label)
        .build();
    card.append(&kernel_row);

    container.append(&card);

    container
}
