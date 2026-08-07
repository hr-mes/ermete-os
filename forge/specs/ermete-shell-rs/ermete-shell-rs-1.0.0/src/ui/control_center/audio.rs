use crate::ui::popup_manager::setup_popup_autoclose;
use crate::ui::viewmodel::{AudioViewModel, AudioIntent};
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Scale};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_audio_mixer_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Mixer Audio")
        .css_classes(["popup-window"])
        .default_width(380)
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
        .spacing(14)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .css_classes(["cc-card"])
        .build();

    // 0. HEADER
    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("🎚️").css_classes(["cc-slider-icon"]).build();
    let header_texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
    let title_lbl = Label::builder().label("MIXER AUDIO ERMETE OS").css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder().label("PipeWire / WirePlumber Control").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    header_texts.append(&title_lbl);
    header_texts.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&header_texts);

    // 1. SEZIONE USCITA AUDIO MASTER
    let out_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["pro-applet-card"])
        .build();
    let out_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let out_lbl = Label::builder().label("🔊  Uscita Audio Master").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_out_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    let mute_out_btn_clone = mute_out_btn.clone();
    mute_out_btn.connect_clicked(move |_| {
        let is_active = mute_out_btn_clone.has_css_class("cc-btn-active");
        if is_active {
            mute_out_btn_clone.remove_css_class("cc-btn-active");
        } else {
            mute_out_btn_clone.add_css_class("cc-btn-active");
        }
        AudioViewModel::execute_intent(AudioIntent::ToggleOutputMute);
    });
    out_header.append(&out_lbl);
    out_header.append(&mute_out_btn);

    let cached_vol = crate::core::get_audio_controller().get_cached_volume() * 100.0;
    let out_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    out_slider.set_value(if cached_vol > 0.0 { cached_vol } else { 80.0 });
    out_slider.set_hexpand(true);
    out_slider.set_valign(Align::Center);
    out_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        AudioViewModel::execute_intent(AudioIntent::SetOutputVolume(val));
    });
    out_card.append(&out_header);
    out_card.append(&out_slider);

    // 2. SEZIONE INGRESSO MICROFONO
    let in_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["pro-applet-card"])
        .build();
    let in_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let in_lbl = Label::builder().label("🎙  Ingresso Microfono").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_in_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    let mute_in_btn_clone = mute_in_btn.clone();
    mute_in_btn.connect_clicked(move |_| {
        let is_active = mute_in_btn_clone.has_css_class("cc-btn-active");
        if is_active {
            mute_in_btn_clone.remove_css_class("cc-btn-active");
        } else {
            mute_in_btn_clone.add_css_class("cc-btn-active");
        }
        AudioViewModel::execute_intent(AudioIntent::ToggleInputMute);
    });
    in_header.append(&in_lbl);
    in_header.append(&mute_in_btn);

    let in_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    in_slider.set_value(75.0);
    in_slider.set_hexpand(true);
    in_slider.set_valign(Align::Center);
    in_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        AudioViewModel::execute_intent(AudioIntent::SetInputVolume(val));
    });
    in_card.append(&in_header);
    in_card.append(&in_slider);

    // 3. SEZIONE MIXER APPLICAZIONI (Per-App Audio Streams)
    let apps_section_lbl = Label::builder()
        .label("🎛  Mixer Applicazioni Attive")
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .margin_top(4)
        .build();

    let apps_container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let apps_container_clone = apps_container.clone();
    AudioViewModel::fetch_app_streams(move |streams| {
        while let Some(child) = apps_container_clone.first_child() {
            apps_container_clone.remove(&child);
        }

        for stream in streams {
            let stream_card = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(6)
                .css_classes(["pro-applet-card"])
                .build();

            let row_header = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .valign(Align::Center)
                .build();

            let icon_lbl = Label::builder().label(&stream.icon).css_classes(["cc-slider-icon"]).build();
            let name_lbl = Label::builder()
                .label(&stream.name)
                .css_classes(["cc-label-main"])
                .hexpand(true)
                .halign(Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();

            let app_mute_btn = Button::builder()
                .label(if stream.muted { "Muto 🔇" } else { "Attivo 🔊" })
                .css_classes(["cc-quick-btn"])
                .build();

            if stream.muted {
                app_mute_btn.add_css_class("cc-btn-active");
            }

            let stream_id = stream.id.clone();
            let app_mute_btn_clone = app_mute_btn.clone();
            app_mute_btn.connect_clicked(move |_| {
                let is_active = app_mute_btn_clone.has_css_class("cc-btn-active");
                if is_active {
                    app_mute_btn_clone.remove_css_class("cc-btn-active");
                    app_mute_btn_clone.set_label("Attivo 🔊");
                } else {
                    app_mute_btn_clone.add_css_class("cc-btn-active");
                    app_mute_btn_clone.set_label("Muto 🔇");
                }
                AudioViewModel::execute_intent(AudioIntent::ToggleAppMute { id: stream_id.clone() });
            });

            row_header.append(&icon_lbl);
            row_header.append(&name_lbl);
            row_header.append(&app_mute_btn);

            let app_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
            app_slider.set_value(stream.volume * 100.0);
            app_slider.set_hexpand(true);
            app_slider.set_valign(Align::Center);

            let stream_id_vol = stream.id.clone();
            app_slider.connect_value_changed(move |s| {
                let val = s.value() / 100.0;
                AudioViewModel::execute_intent(AudioIntent::SetAppVolume {
                    id: stream_id_vol.clone(),
                    volume: val,
                });
            });

            stream_card.append(&row_header);
            stream_card.append(&app_slider);

            apps_container_clone.append(&stream_card);
        }
    });

    // 4. FOOTER & SETTINGS
    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(4)
        .build();

    let settings_audio_btn = Button::builder()
        .label("⚙ Impostazioni Audio")
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let pop_audio_s = pop.clone();
    settings_audio_btn.connect_clicked(move |_| {
        pop_audio_s.close();
        AudioViewModel::execute_intent(AudioIntent::LaunchAudioSettings);
    });

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    footer_box.append(&settings_audio_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&out_card);
    card.append(&in_card);
    card.append(&apps_section_lbl);
    card.append(&apps_container);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}

