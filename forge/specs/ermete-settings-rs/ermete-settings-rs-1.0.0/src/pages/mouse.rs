use gtk4::prelude::*;
use gtk4::{Align, Box, ComboBoxText, Label, Orientation, Scale, Switch};
use crate::components::action_row::ActionRow;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    let title = Label::builder()
        .label("Mouse &amp; Trackpad (libinput)")
        .use_markup(true)
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Natural Scroll
    let switch1 = Switch::builder()
        .valign(Align::Center)
        .build();

    switch1.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::update_niri_kdl_setting("natural-scroll", val).await;
        });
        glib::Propagation::Proceed
    });

    let row1 = ActionRow::builder("Scrolling Naturale")
        .subtitle("Inverti la direzione dello scorrimento (stile macOS)")
        .suffix(&switch1)
        .build();
    container.append(&row1);

    // Tap-to-click
    let switch2 = Switch::builder()
        .valign(Align::Center)
        .build();

    switch2.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::update_niri_kdl_setting("tap-to-click", val).await;
        });
        glib::Propagation::Proceed
    });

    let row2 = ActionRow::builder("Tap-to-click")
        .subtitle("Tocco leggero per simulare il clic del mouse sul Trackpad")
        .suffix(&switch2)
        .build();
    container.append(&row2);

    // Pointer Acceleration Profile
    let combo_accel = ComboBoxText::new();
    combo_accel.append_text("Piatto (Flat - Gaming/Preciso)");
    combo_accel.append_text("Adattivo (Adaptive - Standard libinput)");
    combo_accel.append_text("Personalizzato");
    combo_accel.set_active(Some(1));

    combo_accel.connect_changed(|combo| {
        if let Some(txt) = combo.active_text() {
            let prof = if txt.contains("Piatto") { "flat" } else { "adaptive" };
            relm4::spawn_local(async move {
                ermete_niri_ipc::async_client::update_niri_kdl_setting("accel-profile", prof).await;
            });
        }
    });

    let row3 = ActionRow::builder("Profilo Accelerazione Puntatore")
        .subtitle("Seleziona la risposta della velocità del cursore")
        .suffix(&combo_accel)
        .build();
    container.append(&row3);

    // Scroll Factor / Sensitivity
    let scale_scroll = Scale::with_range(Orientation::Horizontal, 0.2, 3.0, 0.1);
    scale_scroll.set_value(1.0);
    scale_scroll.set_draw_value(true);
    scale_scroll.set_hexpand(true);

    scale_scroll.connect_value_changed(|s| {
        let val = format!("{:.1}", s.value());
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::update_niri_kdl_setting("scroll-factor", &val).await;
        });
    });

    let row4 = ActionRow::builder("Sensibilità e Velocità di Scorrevolezza")
        .subtitle("Regola il moltiplicatore della rotellina e del trackpad (Scroll Factor)")
        .suffix(&scale_scroll)
        .build();
    container.append(&row4);

    // Trackpad Multi-finger Gestures
    let switch5 = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();

    switch5.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::update_niri_kdl_setting("enable-gestures", val).await;
        });
        glib::Propagation::Proceed
    });

    let row5 = ActionRow::builder("Gesture Multi-Touch Trackpad")
        .subtitle("3 dita per cambio workspace, 4 dita per la panoramica")
        .suffix(&switch5)
        .build();
    container.append(&row5);

    container
}
