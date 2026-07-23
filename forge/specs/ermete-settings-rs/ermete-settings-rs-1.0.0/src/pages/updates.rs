use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Button, Spinner};

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(40)
        .margin_start(40)
        .margin_end(40)
        .build();

    let title = Label::builder()
        .label("Aggiornamenti di Sistema")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["settings-card"])
        .build();

    let hbox = Box::builder().orientation(Orientation::Horizontal).spacing(16).build();
    
    let icon = gtk4::Image::builder().icon_name("software-update-available-symbolic").pixel_size(64).build();
    hbox.append(&icon);

    let vbox = Box::builder().orientation(Orientation::Vertical).spacing(4).hexpand(true).valign(Align::Center).build();
    let status_title = Label::builder().label("Ermete OS è aggiornato").css_classes(["heading"]).halign(Align::Start).build();
    let status_desc = Label::builder().label("Ultimo controllo: Oggi alle 10:45\nVersione corrente: Ermete OS 1.0.0 (Layer 0) - Ostree Native").css_classes(["subtitle"]).halign(Align::Start).build();
    vbox.append(&status_title);
    vbox.append(&status_desc);
    hbox.append(&vbox);

    let check_btn = Button::builder().label("Verifica Aggiornamenti").css_classes(["suggested-action"]).valign(Align::Center).build();
    hbox.append(&check_btn);

    card.append(&hbox);

    // OTA Info
    let info_box = Box::builder().orientation(Orientation::Horizontal).spacing(8).margin_top(16).build();
    let info_icon = gtk4::Image::builder().icon_name("dialog-information-symbolic").build();
    let info_text = Label::builder().label("Gli aggiornamenti della UI (Layer 1) vengono applicati istantaneamente senza riavvio (Live Update). Gli aggiornamenti del Kernel richiedono un riavvio.").css_classes(["subtitle"]).wrap(true).build();
    info_box.append(&info_icon);
    info_box.append(&info_text);
    card.append(&info_box);

    container.append(&card);

    container
}
