use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Scale};

#[derive(Debug, Clone)]
pub struct AudioModuleData {
    pub volume: f64,
    pub muted: bool,
    pub sink_name: String,
    pub source_name: String,
    pub source_volume: f64,
    pub source_muted: bool,
}

impl Default for AudioModuleData {
    fn default() -> Self {
        let init_vol = crate::core::get_audio_controller().get_cached_volume() * 100.0;
        let volume = if init_vol > 0.0 { init_vol } else { 80.0 };
        Self {
            volume,
            muted: false,
            sink_name: "PipeWire: ALSA Sink [Default Output]".to_string(),
            source_name: "PipeWire: Internal Microphone".to_string(),
            source_volume: 70.0,
            source_muted: false,
        }
    }
}

pub fn build_audio_widget(data: &AudioModuleData) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // PipeWire Sink Proxy Indicator
    let sink_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
        .build();

    let sink_lbl = Label::builder()
        .label(&format!("󰓃 PipeWire Proxy: {}", data.sink_name))
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();

    sink_box.append(&sink_lbl);

    // Master Volume Slider Row
    let vol_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();

    let vol_icon = Label::builder()
        .label(if data.muted || data.volume == 0.0 { "🔇" } else { "🔊" })
        .css_classes(["cc-slider-icon"])
        .build();

    let vol_slider = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(data.volume, 0.0, 100.0, 1.0, 10.0, 0.0))
        .css_classes(["cc-scale"])
        .hexpand(true)
        .valign(Align::Center)
        .build();

    vol_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetVolume(val));
    });

    let vol_pct = Label::builder()
        .label(&format!("{}%", data.volume.round() as i32))
        .css_classes(["cc-label-main"])
        .build();

    let vol_pct_clone = vol_pct.clone();
    vol_slider.connect_value_changed(move |s| {
        vol_pct_clone.set_label(&format!("{}%", s.value().round() as i32));
    });

    vol_row.append(&vol_icon);
    vol_row.append(&vol_slider);
    vol_row.append(&vol_pct);

    // Mute & Mixer Control Buttons
    let action_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .homogeneous(true)
        .build();

    let mute_btn = Button::builder()
        .label(if data.muted { "󰝟 Unmute Speaker" } else { "󰝞 Mute Speaker" })
        .css_classes(["cc-quick-btn"])
        .build();

    let mixer_btn = Button::builder()
        .label("🎚 PipeWire Mixer")
        .css_classes(["cc-quick-btn"])
        .build();
    mixer_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchSettings("sound".to_string()));
    });

    action_box.append(&mute_btn);
    action_box.append(&mixer_btn);

    container.append(&sink_box);
    container.append(&vol_row);
    container.append(&action_box);
    container
}
