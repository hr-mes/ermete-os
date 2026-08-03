use crate::ui::topbar::setup_popup_autoclose;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Scale};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_audio_mixer_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Mixer Audio")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "media-player");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .css_classes(["cc-card"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("🎚️").css_classes(["cc-slider-icon"]).build();
    let header_texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
    let title_lbl = Label::builder().label("MIXER AUDIO ERMETE OS").css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder().label("PipeWire / WirePlumber").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    header_texts.append(&title_lbl);
    header_texts.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&header_texts);

    // Sezione Uscita Audio
    let out_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["pro-applet-card"])
        .build();
    let out_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let out_lbl = Label::builder().label("🔊  Uscita Audio (Speaker/Cuffie)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_out_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    mute_out_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_audio_controller();
            let _ = ctrl.toggle_mute().await;
        });
    });
    out_header.append(&out_lbl);
    out_header.append(&mute_out_btn);

    let out_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    out_slider.set_value(80.0);
    out_slider.set_hexpand(true);
    out_slider.set_valign(Align::Center);
    out_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_audio_controller();
            let _ = ctrl.set_volume(val).await;
        });
    });
    out_card.append(&out_header);
    out_card.append(&out_slider);

    // Sezione Ingresso Microfono
    let in_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["pro-applet-card"])
        .build();
    let in_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let in_lbl = Label::builder().label("🎙  Ingresso Audio (Microfono)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_in_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    mute_in_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_audio_controller();
            let _ = ctrl.toggle_source_mute().await;
        });
    });
    in_header.append(&in_lbl);
    in_header.append(&mute_in_btn);

    let in_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    in_slider.set_value(75.0);
    in_slider.set_hexpand(true);
    in_slider.set_valign(Align::Center);
    in_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_audio_controller();
            let _ = ctrl.set_source_volume(val).await;
        });
    });
    in_card.append(&in_header);
    in_card.append(&in_slider);

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let settings_audio_btn = Button::builder()
        .label("⚙ Impostazioni Audio")
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let pop_audio_s = pop.clone();
    settings_audio_btn.connect_clicked(move |_| {
        pop_audio_s.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "audio"));
    });
    footer_box.append(&settings_audio_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&out_card);
    card.append(&in_card);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}
