use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Button};

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(40)
        .margin_start(40)
        .margin_end(40)
        .build();

    let title = Label::builder()
        .label("Ecosistema Continuity")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let desc = Label::builder()
        .label("I tuoi dispositivi sulla rete locale comunicano tramite protocolli peer-to-peer cifrati (Ermete Cloud).")
        .css_classes(["subtitle"])
        .halign(Align::Start)
        .wrap(true)
        .build();
    container.append(&desc);

    let list_box = gtk4::ListBox::builder()
        .css_classes(["settings-card"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    let row1 = gtk4::ListBoxRow::builder().build();
    let row1_box = Box::builder().orientation(Orientation::Horizontal).spacing(16).margin_top(12).margin_bottom(12).build();
    let icon1 = gtk4::Image::builder().icon_name("computer-symbolic").pixel_size(32).build();
    let text_box1 = Box::builder().orientation(Orientation::Vertical).hexpand(true).build();
    text_box1.append(&Label::builder().label("Appunti Universali (Clipboard Sync)").halign(Align::Start).css_classes(["heading"]).build());
    text_box1.append(&Label::builder().label("Copia testo o immagini su questo computer e incollali istantaneamente su un altro dispositivo Ermete.").halign(Align::Start).css_classes(["subtitle"]).wrap(true).build());
    let switch1 = gtk4::Switch::builder().valign(Align::Center).active(true).build();
    row1_box.append(&icon1);
    row1_box.append(&text_box1);
    row1_box.append(&switch1);
    row1.set_child(Some(&row1_box));
    list_box.append(&row1);

    container.append(&list_box);

    let devices_title = Label::builder()
        .label("Dispositivi Scoperti")
        .css_classes(["heading"])
        .halign(Align::Start)
        .margin_top(16)
        .build();
    container.append(&devices_title);

    let dev_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["settings-card"])
        .build();

    let dev_empty = Label::builder()
        .label("Ricerca dispositivi Ermete in corso sulla rete locale (mDNS)...")
        .css_classes(["subtitle"])
        .halign(Align::Center)
        .build();
    dev_box.append(&dev_empty);

    container.append(&dev_box);

    container
}
