use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, Switch};
use crate::components::action_row::ActionRow;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("<span size='x-large' weight='bold'>Generali</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&title);

    let kernel_version = match std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        Ok(o) => o.trim().to_string(),
        Err(_) => "6.12.0-chimera".to_string(),
    };
    let arch = std::env::consts::ARCH.to_string();

    let row_os = ActionRow::builder("Sistema Operativo")
        .subtitle("Ermete OS")
        .build();
    container.append(&row_os);

    let row_kernel = ActionRow::builder("Versione Kernel")
        .subtitle(&kernel_version)
        .build();
    container.append(&row_kernel);

    let row_arch = ActionRow::builder("Architettura")
        .subtitle(&arch)
        .build();
    container.append(&row_arch);

    // Updates
    let update_button = Button::builder()
        .label("Controlla Aggiornamenti")
        .halign(Align::Start)
        .build();

    let update_status = Label::builder()
        .label("")
        .halign(Align::Start)
        .build();

    let update_status_clone = update_status.clone();
    update_button.connect_clicked(move |_| {
        let status_c = update_status_clone.clone();
        relm4::spawn_local(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            status_c.set_label("Sistema Aggiornato");
        });
    });

    let row_updates = ActionRow::builder("Aggiornamenti di Sistema")
        .subtitle("Verifica la disponibilità di nuove versioni per Ermete OS")
        .suffix(&update_button)
        .build();
    container.append(&row_updates);
    container.append(&update_status);

    // Accessibility (VoiceOver)
    let a11y_title = Label::builder()
        .label("<span size='large' weight='bold'>Accessibilità</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(16)
        .build();
    container.append(&a11y_title);

    let vo_switch = Switch::builder().valign(Align::Center).build();
    let vo_sw_clone = vo_switch.clone();

    vo_switch.connect_state_set(move |_, state| {
        relm4::spawn_local(async move {
            if let Ok(connection) = crate::get_connection().await {
                let _ = connection.call_method(
                    Some("org.ermete.Settings"),
                    "/org/ermete/Settings",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.ermete.Settings", "VoiceOverEnabled", zbus::zvariant::Value::from(state))
                ).await;
            }
        });
        glib::Propagation::Proceed
    });

    relm4::spawn_local(async move {
        if let Ok(connection) = crate::get_connection().await {
            if let Ok(msg) = connection.call_method(
                Some("org.ermete.Settings"),
                "/org/ermete/Settings",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.ermete.Settings", "VoiceOverEnabled")
            ).await {
                if let Ok(val) = msg.body::<zbus::zvariant::OwnedValue>() {
                    if let Ok(enabled) = bool::try_from(val) {
                        vo_sw_clone.set_active(enabled);
                    }
                }
            }
        }
    });

    let row_vo = ActionRow::builder("VoiceOver")
        .subtitle("Screen Reader Nativo per l'accessibilità")
        .suffix(&vo_switch)
        .build();
    container.append(&row_vo);

    container
}
