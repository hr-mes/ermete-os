use gtk4::prelude::*;
use gtk4::{Adjustment, Box, ComboBoxText, Label, Orientation, Scale};

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
        .halign(gtk4::Align::Start)
        .build();
    title.add_css_class("title-1");
    container.append(&title);

    let content_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();
    container.append(&content_box);

    // Layout
    let layout_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
        
    let layout_label = Label::builder()
        .label("Layout")
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
        
    let layout_combo = ComboBoxText::new();
    layout_combo.append(Some("it"), "Italiano");
    layout_combo.append(Some("us"), "English (US)");
    layout_combo.append(Some("uk"), "English (UK)");
    layout_combo.set_active_id(Some("it"));

    layout_combo.connect_changed(|combo| {
        if let Some(idx) = combo.active() {
            crate::niri_client::set_keyboard_layout_by_index(idx as usize);
        }
    });

    layout_box.append(&layout_label);
    layout_box.append(&layout_combo);
    content_box.append(&layout_box);

    // Repeat Rate
    let rate_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
        
    let rate_label = Label::builder()
        .label("Velocità Ripetizione Tasti")
        .halign(gtk4::Align::Start)
        .build();
        
    let rate_adj = Adjustment::new(25.0, 10.0, 100.0, 1.0, 5.0, 0.0);
    let rate_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&rate_adj)
        .digits(0)
        .draw_value(true)
        .build();
        
    rate_box.append(&rate_label);
    rate_box.append(&rate_scale);
    content_box.append(&rate_box);

    container
}
