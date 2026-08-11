use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Image, Label, Orientation, Separator, Switch};
use crate::components::action_row::ActionRow;

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // Page Title & Subtitle
    let title = Label::builder()
        .label("<b>Continuity &amp; Handoff</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    title.add_css_class("title-1");
    container.append(&title);

    let desc = Label::builder()
        .label("Sincronizza gli appunti e il lavoro in corso tra il tuo PC e i dispositivi Ermete sulla rete locale via Ermete Cloud P2P.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    desc.add_css_class("subtitle");
    container.append(&desc);

    // Main Features Settings Card
    let settings_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["liquid-surface"])
        .build();

    // 1. Universal Clipboard Switch
    let clipboard_switch = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();

    clipboard_switch.connect_state_set(move |_, state| {
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("universal_clipboard", if state { "true" } else { "false" }).await;
            if let Ok(connection) = crate::get_connection().await {
                let _ = connection
                    .call_method(
                        Some("org.ermete.Settings"),
                        "/org/ermete/Settings",
                        Some("org.freedesktop.DBus.Properties"),
                        "Set",
                        &("org.ermete.Settings", "ClipboardSyncEnabled", zbus::zvariant::Value::from(state)),
                    )
                    .await;
            }
        });
        glib::Propagation::Proceed
    });

    let clipboard_row = ActionRow::builder("Universal Clipboard (Appunti Universali)")
        .subtitle("Copia su PC, Incolla su telefono. Condividi istantaneamente testo e elementi negli appunti tra tutti i dispositivi connessi.")
        .suffix(&clipboard_switch)
        .build();

    settings_card.append(&clipboard_row);

    let separator = Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    settings_card.append(&separator);

    // 2. Handoff Switch
    let handoff_switch = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();

    handoff_switch.connect_state_set(move |_, state| {
        relm4::spawn_local(async move {
            let _ = crate::crdt_store::update_setting_crdt("handoff_enabled", if state { "true" } else { "false" }).await;
            if let Ok(connection) = crate::get_connection().await {
                let _ = connection
                    .call_method(
                        Some("org.ermete.Settings"),
                        "/org/ermete/Settings",
                        Some("org.freedesktop.DBus.Properties"),
                        "Set",
                        &("org.ermete.Settings", "HandoffEnabled", zbus::zvariant::Value::from(state)),
                    )
                    .await;
            }
        });
        glib::Propagation::Proceed
    });

    let handoff_row = ActionRow::builder("Handoff (Continuità Applicazioni)")
        .subtitle("Continua l'app su un altro schermo. Trasferisci la sessione di lavoro attiva su un dispositivo Ermete nelle vicinanze.")
        .suffix(&handoff_switch)
        .build();

    settings_card.append(&handoff_row);
    container.append(&settings_card);

    // 3. Connected Devices Section
    let devices_title = Label::builder()
        .label("<b>Dispositivi Connessi</b>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(16)
        .build();
    devices_title.add_css_class("heading");
    container.append(&devices_title);

    let devices_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["liquid-surface"])
        .build();

    // Helper to build a device row with Image icon, title/subtitle and status/action button
    let create_device_row = |icon_name: &str, device_name: &str, details: &str, status: &str, is_active: bool| -> GtkBox {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(16)
            .css_classes(["action-row"])
            .build();

        let icon = Image::builder()
            .icon_name(icon_name)
            .pixel_size(36)
            .valign(Align::Center)
            .build();

        let text_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(Align::Center)
            .hexpand(true)
            .build();

        let name_lbl = Label::builder()
            .label(device_name)
            .halign(Align::Start)
            .css_classes(["action-row-title"])
            .build();

        let sub_lbl = Label::builder()
            .label(details)
            .halign(Align::Start)
            .css_classes(["action-row-subtitle"])
            .build();

        text_box.append(&name_lbl);
        text_box.append(&sub_lbl);

        let action_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(Align::Center)
            .build();

        let status_lbl = Label::builder()
            .label(status)
            .valign(Align::Center)
            .css_classes(["dim-label"])
            .build();
        action_box.append(&status_lbl);

        if is_active {
            let handoff_btn = Button::builder()
                .label("Invia Handoff")
                .valign(Align::Center)
                .build();

            let d_name = device_name.to_string();
            handoff_btn.connect_clicked(move |_| {
                let target_device = d_name.clone();
                relm4::spawn_local(async move {
                    let _ = crate::crdt_store::update_setting_crdt("active_handoff_target", &target_device).await;
                });
            });

            action_box.append(&handoff_btn);
        }

        row.append(&icon);
        row.append(&text_box);
        row.append(&action_box);

        row
    };

    let dev1 = create_device_row(
        "phone-symbolic",
        "Ermete Phone",
        "[No Real Bluetooth/Wifi-Direct devices detected]",
        "Connesso",
        true,
    );
    devices_card.append(&dev1);

    let dev_sep1 = Separator::builder().orientation(Orientation::Horizontal).build();
    devices_card.append(&dev_sep1);

    let dev2 = create_device_row(
        "tablet-symbolic",
        "Ermete Tab Pro",
        "",
        "Connesso",
        true,
    );
    devices_card.append(&dev2);

    let dev_sep2 = Separator::builder().orientation(Orientation::Horizontal).build();
    devices_card.append(&dev_sep2);

    let dev3 = create_device_row(
        "computer-symbolic",
        "Ermete Book Ultra",
        "Laptop Ermete OS • Standby Rete Locale",
        "Disponibile",
        false,
    );
    devices_card.append(&dev3);

    container.append(&devices_card);

    container
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuity_page_builds() {
        gtk4::init().unwrap_or_default();
        let _page = build_page();
    }
}
