use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ToggleButton};
use crate::components::action_row::ActionRow;
use crate::settings_proxy::with_settings_proxy;

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

    let settings_card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["liquid-surface"])
        .build();

    // Color Scheme Section
    let scheme_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .valign(Align::Center)
        .build();

    let btn_light = ToggleButton::with_label("Chiaro");
    let btn_dark = ToggleButton::with_label("Scuro");
    let btn_auto = ToggleButton::with_label("Auto");

    btn_light.set_size_request(100, 40);
    btn_dark.set_size_request(100, 40);
    btn_auto.set_size_request(100, 40);

    btn_dark.set_group(Some(&btn_light));
    btn_auto.set_group(Some(&btn_light));

    btn_light.connect_toggled(|btn| {
        if btn.is_active() {
            relm4::spawn_local(async move {
                with_settings_proxy(|proxy| async move {
                    let _ = proxy.set_color_scheme("prefer-light").await;
                }).await;
                crate::crdt_store::update_theme_crdt("prefer-light").await;
            });
        }
    });

    btn_dark.connect_toggled(|btn| {
        if btn.is_active() {
            relm4::spawn_local(async move {
                with_settings_proxy(|proxy| async move {
                    let _ = proxy.set_color_scheme("prefer-dark").await;
                }).await;
                crate::crdt_store::update_theme_crdt("prefer-dark").await;
            });
        }
    });

    btn_auto.connect_toggled(|btn| {
        if btn.is_active() {
            relm4::spawn_local(async move {
                with_settings_proxy(|proxy| async move {
                    let _ = proxy.set_color_scheme("default").await;
                }).await;
                crate::crdt_store::update_theme_crdt("default").await;
            });
        }
    });

    scheme_box.append(&btn_light);
    scheme_box.append(&btn_dark);
    scheme_box.append(&btn_auto);

    let scheme_row = ActionRow::builder("Tema Colore")
        .subtitle("Seleziona la modalità di visualizzazione dell'interfaccia")
        .suffix(&scheme_box)
        .build();

    settings_card.append(&scheme_row);

    let separator = gtk4::Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    settings_card.append(&separator);

    // Accent Color Section
    let accent_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
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
        btn.set_size_request(70, 36);
        let hex_clone = hex_val.to_string();
        btn.connect_clicked(move |_| {
            let hex_c = hex_clone.clone();
            let _ = crate::accent_engine::apply_accent_color(&hex_c);
            let hex_c2 = hex_c.clone();
            relm4::spawn_local(async move {
                crate::settings_proxy::with_appearance_proxy(move |proxy| async move {
                    let _ = proxy.set_accent_color(&hex_c).await;
                }).await;
                crate::crdt_store::update_accent_color_crdt(&hex_c2).await;
            });
        });
        accent_box.append(&btn);
    }

    let accent_row = ActionRow::builder("Colore Accento")
        .subtitle("Personalizza il colore di evidenziazione dell'interfaccia")
        .suffix(&accent_box)
        .build();

    settings_card.append(&accent_row);

    container.append(&settings_card);

    // Zorin-Style System Layout Switcher Section
    let layout_switcher = crate::pages::layout_switcher::build_switcher_section();
    container.append(&layout_switcher);


    // Load current state from D-Bus on page initialization
    let bl = btn_light.clone();
    let bd = btn_dark.clone();
    let ba = btn_auto.clone();
    relm4::spawn_local(async move {
        with_settings_proxy(move |proxy| async move {
            if let Ok(scheme) = proxy.color_scheme().await {
                match scheme.as_str() {
                    "prefer-dark" => bd.set_active(true),
                    "prefer-light" => bl.set_active(true),
                    _ => ba.set_active(true),
                }
            }
        }).await;
    });

    container
}
