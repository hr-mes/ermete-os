use gtk4::prelude::*;
use std::fs;

pub fn build_page() -> gtk4::Box {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = gtk4::Label::builder()
        .label("<span size='x-large' weight='bold'>Generali</span>")
        .use_markup(true)
        .halign(gtk4::Align::Start)
        .build();
    
    container.append(&title);

    let os_name = "Ermete OS";
    
    let kernel_version = fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|o| o.trim().to_string())
        .unwrap_or_else(|_| "6.12.0-chimera".to_string());
        
    let arch = std::env::consts::ARCH.to_string();

    let info_text = format!(
        "<b>Sistema Operativo:</b> {}\n<b>Versione Kernel:</b> {}\n<b>Architettura:</b> {}",
        os_name, kernel_version, arch
    );

    let info_label = gtk4::Label::builder()
        .label(&info_text)
        .use_markup(true)
        .halign(gtk4::Align::Start)
        .build();
        
    container.append(&info_label);

    let update_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();

    let update_button = gtk4::Button::builder()
        .label("Controlla Aggiornamenti")
        .halign(gtk4::Align::Start)
        .build();
        
    let update_status = gtk4::Label::builder()
        .label("")
        .halign(gtk4::Align::Start)
        .build();

    update_box.append(&update_button);
    update_box.append(&update_status);
    container.append(&update_box);

    update_button.connect_clicked(move |_| {
        update_status.set_label("Sistema Aggiornato");
    });

    // Accessibilità (VoiceOver)
    let a11y_title = gtk4::Label::builder()
        .label("<span size='large' weight='bold'>Accessibilità</span>")
        .use_markup(true)
        .halign(gtk4::Align::Start)
        .margin_top(16)
        .build();
    container.append(&a11y_title);

    let vo_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .build();
    let vo_lbl = gtk4::Label::builder()
        .label("VoiceOver (Screen Reader Nativo)")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    let vo_switch = gtk4::Switch::builder().valign(gtk4::Align::Center).build();
    let vo_sw_clone = vo_switch.clone();

    vo_switch.connect_state_set(move |_, state| {
        glib::MainContext::default().spawn_local(async move {
            if let Ok(connection) = zbus::Connection::session().await {
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

    glib::MainContext::default().spawn_local(async move {
        if let Ok(connection) = zbus::Connection::session().await {
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

    vo_box.append(&vo_lbl);
    vo_box.append(&vo_switch);
    container.append(&vo_box);

    container
}
