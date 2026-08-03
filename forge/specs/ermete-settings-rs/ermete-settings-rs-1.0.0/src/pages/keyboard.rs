use gtk4::prelude::*;
use gtk4::{Adjustment, Align, Box, ComboBoxText, Label, Orientation, Scale};
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
        .label("Tastiera")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Layout
    let layout_combo = ComboBoxText::new();
    layout_combo.append(Some("it"), "Italiano");
    layout_combo.append(Some("us"), "English (US)");
    layout_combo.append(Some("uk"), "English (UK)");
    layout_combo.set_active_id(Some("it"));

    layout_combo.connect_changed(|combo| {
        if let Some(idx) = combo.active() {
            relm4::spawn_local(async move {
                ermete_niri_ipc::async_client::set_keyboard_layout_by_index(idx as usize).await;
            });
        }
    });

    let layout_row = ActionRow::builder("Layout Tastiera")
        .subtitle("Mappatura e disposizione dei tasti per la digitazione")
        .suffix(&layout_combo)
        .build();
    container.append(&layout_row);

    // Repeat Rate
    let rate_adj = Adjustment::new(25.0, 10.0, 100.0, 1.0, 5.0, 0.0);
    let rate_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&rate_adj)
        .digits(0)
        .draw_value(true)
        .hexpand(true)
        .build();

    rate_scale.connect_value_changed(|scale| {
        let val = scale.value() as u32;
        let val_str = val.to_string();
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::update_niri_kdl_setting("repeat-rate", &val_str).await;
        });
    });

    let rate_row = ActionRow::builder("Velocità Ripetizione Tasti")
        .subtitle("Frequenza dei caratteri generati quando un tasto è tenuto premuto")
        .suffix(&rate_scale)
        .build();
    container.append(&rate_row);

    // Repeat Delay
    let delay_adj = Adjustment::new(600.0, 200.0, 1000.0, 50.0, 100.0, 0.0);
    let delay_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&delay_adj)
        .digits(0)
        .draw_value(true)
        .hexpand(true)
        .build();

    delay_scale.connect_value_changed(|scale| {
        let val = scale.value() as u32;
        let val_str = val.to_string();
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::update_niri_kdl_setting("repeat-delay", &val_str).await;
        });
    });

    let delay_row = ActionRow::builder("Ritardo di Ripetizione (ms)")
        .subtitle("Intervallo di attesa iniziale prima dell'avvio della ripetizione")
        .suffix(&delay_scale)
        .build();
    container.append(&delay_row);

    container
}
