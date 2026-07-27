use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Switch};
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
        .label("Ecosistema Continuity")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let desc = Label::builder()
        .label("I tuoi dispositivi sulla rete locale comunicano tramite protocolli peer-to-peer cifrati (Ermete Cloud).")
        .css_classes(["subtitle"])
        .halign(Align::Start)
        .wrap(true)
        .build();
    container.append(&desc);

    let switch1 = Switch::builder().valign(Align::Center).active(true).build();
    switch1.connect_state_set(move |_, state| {
        relm4::spawn_local(async move {
            if let Ok(connection) = crate::get_connection().await {
                let _ = connection.call_method(
                    Some("org.ermete.Settings"),
                    "/org/ermete/Settings",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.ermete.Settings", "ClipboardSyncEnabled", zbus::zvariant::Value::from(state))
                ).await;
            }
        });
        glib::Propagation::Proceed
    });

    let row1 = ActionRow::builder("Appunti Universali (Clipboard Sync)")
        .subtitle("Copia testo o immagini su questo computer e incollali istantaneamente su un altro dispositivo Ermete.")
        .suffix(&switch1)
        .build();
    container.append(&row1);

    let devices_title = Label::builder()
        .label("Dispositivi Scoperti")
        .css_classes(["heading"])
        .halign(Align::Start)
        .margin_top(16)
        .build();
    container.append(&devices_title);

    let dev_row = ActionRow::builder("Ricerca dispositivi Ermete")
        .subtitle("Scansione automatica in corso sulla rete locale via mDNS...")
        .build();
    container.append(&dev_row);

    container
}
