use gtk4::prelude::*;
use gtk4::{Align, Box, ComboBoxText, Label, ListBox, ListBoxRow, Orientation, Scale, Switch};

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
        .label("Mouse & Trackpad (libinput)")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let list_box = ListBox::builder()
        .css_classes(["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    // Natural Scroll
    let row1 = ListBoxRow::new();
    let hbox1 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label1 = Label::builder()
        .label("Scrolling Naturale (Invertito come macOS)")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let switch1 = Switch::builder()
        .valign(Align::Center)
        .build();

    switch1.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        crate::niri_client::update_niri_kdl_setting("natural-scroll", val);
        glib::Propagation::Proceed
    });

    hbox1.append(&label1);
    hbox1.append(&switch1);
    row1.set_child(Some(&hbox1));
    list_box.append(&row1);

    // Tap-to-click
    let row2 = ListBoxRow::new();
    let hbox2 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label2 = Label::builder()
        .label("Tap-to-click (Tocco per fare clic sul Trackpad)")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let switch2 = Switch::builder()
        .valign(Align::Center)
        .build();

    switch2.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        crate::niri_client::update_niri_kdl_setting("tap-to-click", val);
        glib::Propagation::Proceed
    });

    hbox2.append(&label2);
    hbox2.append(&switch2);
    row2.set_child(Some(&hbox2));
    list_box.append(&row2);

    // Pointer Acceleration Profile
    let row3 = ListBoxRow::new();
    let hbox3 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label3 = Label::builder()
        .label("Profilo Accelerazione Puntatore:")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let combo_accel = ComboBoxText::new();
    combo_accel.append_text("Piatto (Flat - Gaming/Preciso)");
    combo_accel.append_text("Adattivo (Adaptive - Standard libinput)");
    combo_accel.append_text("Personalizzato");
    combo_accel.set_active(Some(1));

    combo_accel.connect_changed(|combo| {
        if let Some(txt) = combo.active_text() {
            let prof = if txt.contains("Piatto") { "flat" } else { "adaptive" };
            crate::niri_client::update_niri_kdl_setting("accel-profile", prof);
        }
    });

    hbox3.append(&label3);
    hbox3.append(&combo_accel);
    row3.set_child(Some(&hbox3));
    list_box.append(&row3);

    // Scroll Factor / Sensitivity
    let row4 = ListBoxRow::new();
    let hbox4 = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label4 = Label::builder()
        .label("Sensibilità e Velocità di Scorrevolezza (Scroll Factor):")
        .halign(Align::Start)
        .build();

    let scale_scroll = Scale::with_range(Orientation::Horizontal, 0.2, 3.0, 0.1);
    scale_scroll.set_value(1.0);
    scale_scroll.set_draw_value(true);

    scale_scroll.connect_value_changed(|s| {
        let val = format!("{:.1}", s.value());
        crate::niri_client::update_niri_kdl_setting("scroll-factor", &val);
    });

    hbox4.append(&label4);
    hbox4.append(&scale_scroll);
    row4.set_child(Some(&hbox4));
    list_box.append(&row4);

    // Trackpad Multi-finger Gestures
    let row5 = ListBoxRow::new();
    let hbox5 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label5 = Label::builder()
        .label("Gesture Multi-Touch Trackpad (3 dita cambio workspace, 4 dita panoramica)")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let switch5 = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();

    switch5.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        crate::niri_client::update_niri_kdl_setting("enable-gestures", val);
        glib::Propagation::Proceed
    });

    hbox5.append(&label5);
    hbox5.append(&switch5);
    row5.set_child(Some(&hbox5));
    list_box.append(&row5);

    container.append(&list_box);
    container
}
