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

    let kernel_subtitle = gtk4::Label::builder()
        .label("Caricamento...")
        .halign(Align::End)
        .build();
    let row_kernel = ActionRow::builder("Versione Kernel")
        .suffix(&kernel_subtitle)
        .build();
    container.append(&row_kernel);

    let kernel_sub_clone = kernel_subtitle.clone();
    relm4::spawn_local(async move {
        let mut ver = "6.12.0-ermete".to_string();
        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(msg) = conn.call_method(
                Some("org.freedesktop.hostname1"),
                "/org/freedesktop/hostname1",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.hostname1", "KernelRelease"),
            ).await {
                if let Ok(val) = msg.body().deserialize::<zbus::zvariant::OwnedValue>() {
                    if let Ok(kver) = String::try_from(val) {
                        ver = kver;
                    }
                }
            }
        }
        kernel_sub_clone.set_label(&ver);
    });

    let arch = std::env::consts::ARCH.to_string();

    let row_os = ActionRow::builder("Sistema Operativo")
        .subtitle("Ermete OS")
        .build();
    container.append(&row_os);

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
            status_c.set_label("Controllo in corso...");
            if let Ok(output) = tokio::process::Command::new("ostree").args(["admin", "status"]).output().await {
                if output.status.success() {
                    status_c.set_label("Sistema Aggiornato");
                } else {
                    status_c.set_label("Errore controllo");
                }
            } else {
                status_c.set_label("Comando ostree assente");
            }
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
            let _ = crate::crdt_store::update_setting_crdt("voice_over_enabled", &state.to_string()).await;
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
                if let Ok(val) = msg.body().deserialize::<zbus::zvariant::OwnedValue>() {
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
