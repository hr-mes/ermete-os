use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Switch};

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(40)
        .margin_start(40)
        .margin_end(40)
        .build();

    let title = Label::builder()
        .label("Ermete Labs (Sperimentale)")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let desc = Label::builder()
        .label("Funzionalità avanzate basate su Intelligenza Artificiale e Spatial Computing.\nQueste funzioni richiedono hardware specifico o molta RAM e sono disabilitate di default.")
        .css_classes(["subtitle"])
        .halign(Align::Start)
        .wrap(true)
        .build();
    container.append(&desc);

    let list_box = gtk4::ListBox::builder()
        .css_classes(["settings-card"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    // Gaze Tracking
    let row1 = gtk4::ListBoxRow::builder().build();
    let row1_box = Box::builder().orientation(Orientation::Horizontal).spacing(16).margin_top(12).margin_bottom(12).build();
    let icon1 = gtk4::Image::builder().icon_name("camera-web-symbolic").pixel_size(32).build();
    let text_box1 = Box::builder().orientation(Orientation::Vertical).hexpand(true).build();
    text_box1.append(&Label::builder().label("Gaze-Tracking Navigation").halign(Align::Start).css_classes(["heading"]).build());
    text_box1.append(&Label::builder().label("Controlla il puntatore e lo scroll usando il movimento oculare tramite webcam.").halign(Align::Start).css_classes(["subtitle"]).wrap(true).build());
    let switch1 = Switch::builder().valign(Align::Center).build();
    row1_box.append(&icon1);
    row1_box.append(&text_box1);
    row1_box.append(&switch1);
    row1.set_child(Some(&row1_box));
    list_box.append(&row1);

    // Generative UI
    let row2 = gtk4::ListBoxRow::builder().build();
    let row2_box = Box::builder().orientation(Orientation::Horizontal).spacing(16).margin_top(12).margin_bottom(12).build();
    let icon2 = gtk4::Image::builder().icon_name("applications-engineering-symbolic").pixel_size(32).build();
    let text_box2 = Box::builder().orientation(Orientation::Vertical).hexpand(true).build();
    text_box2.append(&Label::builder().label("Generative UI Engine").halign(Align::Start).css_classes(["heading"]).build());
    text_box2.append(&Label::builder().label("Le interfacce si riadattano in tempo reale basandosi sui tuoi pattern d'uso tramite LLM locale.").halign(Align::Start).css_classes(["subtitle"]).wrap(true).build());
    let switch2 = Switch::builder().valign(Align::Center).build();
    row2_box.append(&icon2);
    row2_box.append(&text_box2);
    row2_box.append(&switch2);
    row2.set_child(Some(&row2_box));
    list_box.append(&row2);

    // Spatial Audio
    let row3 = gtk4::ListBoxRow::builder().build();
    let row3_box = Box::builder().orientation(Orientation::Horizontal).spacing(16).margin_top(12).margin_bottom(12).build();
    let icon3 = gtk4::Image::builder().icon_name("audio-headphones-symbolic").pixel_size(32).build();
    let text_box3 = Box::builder().orientation(Orientation::Vertical).hexpand(true).build();
    text_box3.append(&Label::builder().label("Audio Spaziale Dinamico").halign(Align::Start).css_classes(["heading"]).build());
    text_box3.append(&Label::builder().label("Sintesi binaurale: i suoni di notifica provengono fisicamente dalla posizione della finestra nello spazio visivo.").halign(Align::Start).css_classes(["subtitle"]).wrap(true).build());
    let switch3 = Switch::builder().valign(Align::Center).build();
    row3_box.append(&icon3);
    row3_box.append(&text_box3);
    row3_box.append(&switch3);
    row3.set_child(Some(&row3_box));
    list_box.append(&row3);

    container.append(&list_box);

    container
}
