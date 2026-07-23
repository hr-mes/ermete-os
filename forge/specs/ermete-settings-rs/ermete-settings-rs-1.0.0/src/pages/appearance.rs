use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ToggleButton};
use crate::settings_proxy::SettingsProxy;

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
        .label("<b>Aspetto e Temi</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    title.add_css_class("title-1");

    container.append(&title);

    // Color Scheme Section
    let scheme_label = Label::builder()
        .label("Tema Colore")
        .halign(Align::Start)
        .build();
    scheme_label.add_css_class("heading");
    container.append(&scheme_label);

    let scheme_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .build();

    let btn_light = ToggleButton::with_label("Chiaro");
    let btn_dark = ToggleButton::with_label("Scuro");
    let btn_auto = ToggleButton::with_label("Auto");

    btn_light.set_size_request(120, 80);
    btn_dark.set_size_request(120, 80);
    btn_auto.set_size_request(120, 80);

    btn_dark.set_group(Some(&btn_light));
    btn_auto.set_group(Some(&btn_light));

    btn_light.connect_toggled(|btn| {
        if btn.is_active() {
            let ctx = gtk4::glib::MainContext::default();
            ctx.spawn_local(async move {
                if let Ok(conn) = crate::get_connection().await {
                    if let Ok(proxy) = SettingsProxy::new(&conn).await {
                        let _ = proxy.set_color_scheme("prefer-light").await;
                    }
                }
            });
        }
    });

    btn_dark.connect_toggled(|btn| {
        if btn.is_active() {
            let ctx = gtk4::glib::MainContext::default();
            ctx.spawn_local(async move {
                if let Ok(conn) = crate::get_connection().await {
                    if let Ok(proxy) = SettingsProxy::new(&conn).await {
                        let _ = proxy.set_color_scheme("prefer-dark").await;
                    }
                }
            });
        }
    });

    btn_auto.connect_toggled(|btn| {
        if btn.is_active() {
            let ctx = gtk4::glib::MainContext::default();
            ctx.spawn_local(async move {
                if let Ok(conn) = crate::get_connection().await {
                    if let Ok(proxy) = SettingsProxy::new(&conn).await {
                        let _ = proxy.set_color_scheme("default").await;
                    }
                }
            });
        }
    });

    scheme_box.append(&btn_light);
    scheme_box.append(&btn_dark);
    scheme_box.append(&btn_auto);
    container.append(&scheme_box);

    // Accent Color Section
    let accent_label = Label::builder()
        .label("Colore Accento")
        .halign(Align::Start)
        .build();
    accent_label.add_css_class("heading");
    container.append(&accent_label);

    let accent_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .build();

    let accents = [
        ("Blu", "blue", "#89b4fa"),
        ("Rosso", "red", "#f38ba8"),
        ("Verde", "green", "#a6e3a1"),
        ("Arancione", "orange", "#fab387"),
        ("Viola", "purple", "#cba6f7"),
        ("Rosa", "pink", "#f5c2e7"),
    ];

    for (name, _gnome_val, hex_val) in accents {
        let btn = Button::with_label(name);
        btn.set_size_request(80, 40);
        let hex_clone = hex_val.to_string();
        btn.connect_clicked(move |_| {
            let hex_c = hex_clone.clone();
            let ctx = gtk4::glib::MainContext::default();
            ctx.spawn_local(async move {
                if let Ok(conn) = crate::get_connection().await {
                    if let Ok(proxy) = SettingsProxy::new(&conn).await {
                        let _ = proxy.set_accent_color(&hex_c).await;
                    }
                }
            });
        });
        accent_box.append(&btn);
    }

    container.append(&accent_box);

    // Load current state from D-Bus on page initialization
    let bl = btn_light.clone();
    let bd = btn_dark.clone();
    let ba = btn_auto.clone();
    let ctx = gtk4::glib::MainContext::default();
    ctx.spawn_local(async move {
        if let Ok(conn) = crate::get_connection().await {
            if let Ok(proxy) = SettingsProxy::new(&conn).await {
                if let Ok(scheme) = proxy.color_scheme().await {
                    match scheme.as_str() {
                        "prefer-dark" => bd.set_active(true),
                        "prefer-light" => bl.set_active(true),
                        _ => ba.set_active(true),
                    }
                }
            }
        }
    });

    container
}
