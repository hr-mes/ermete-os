use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};

pub(crate) fn build_cc_row(badge_class: &str, icon_glyph: &str, title: &str, sub: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-tile-row"]).build();
    let row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(14)
        .valign(Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(8)
        .margin_end(8)
        .build();

    let badge = Label::builder()
        .label(icon_glyph)
        .css_classes([badge_class])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let text_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(1)
        .valign(Align::Center)
        .build();

    let lbl_title = Label::builder()
        .label(title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();
    let lbl_sub = Label::builder()
        .label(sub)
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    text_box.append(&lbl_title);
    text_box.append(&lbl_sub);

    row_box.append(&badge);
    row_box.append(&text_box);
    btn.set_child(Some(&row_box));
    btn
}

pub(crate) fn build_cc_compact_tile(badge_class: &str, icon_glyph: &str, title: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-tile"]).hexpand(true).build();
    let row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .valign(Align::Center)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let badge = Label::builder()
        .label(icon_glyph)
        .css_classes([badge_class])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let lbl = Label::builder()
        .label(title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();

    row_box.append(&badge);
    row_box.append(&lbl);
    btn.set_child(Some(&row_box));
    btn
}
