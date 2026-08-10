use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Scale, Switch};

#[derive(Debug, Clone)]
pub struct DisplayModuleData {
    pub brightness: f64,
    pub mica_tint: f64,
    pub mica_preset: String,
    pub true_tone: bool,
    pub dark_mode: bool,
}

impl Default for DisplayModuleData {
    fn default() -> Self {
        let init_bright = crate::core::live_state::get_live_state().brightness;
        let brightness = if init_bright > 0.0 { init_bright } else { 75.0 };
        Self {
            brightness,
            mica_tint: 65.0,
            mica_preset: "Deep Mica".to_string(),
            true_tone: false,
            dark_mode: true,
        }
    }
}

pub fn build_display_widget(data: &DisplayModuleData) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // Brightness Slider Row
    let bright_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();

    let bright_icon = Label::builder()
        .label("☀")
        .css_classes(["cc-slider-icon"])
        .build();

    let bright_slider = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(data.brightness, 0.0, 100.0, 1.0, 10.0, 0.0))
        .css_classes(["cc-scale"])
        .hexpand(true)
        .valign(Align::Center)
        .build();

    let bright_pct = Label::builder()
        .label(&format!("{}%", data.brightness.round() as i32))
        .css_classes(["cc-label-main"])
        .build();

    let bright_pct_clone = bright_pct.clone();
    bright_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        bright_pct_clone.set_label(&format!("{}%", s.value().round() as i32));
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetBrightness(val));
    });

    bright_row.append(&bright_icon);
    bright_row.append(&bright_slider);
    bright_row.append(&bright_pct);

    // Mica Glass Tinting Intensity Row
    let mica_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();

    let mica_icon = Label::builder()
        .label("✨")
        .css_classes(["cc-slider-icon"])
        .build();

    let mica_slider = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(data.mica_tint, 0.0, 100.0, 1.0, 10.0, 0.0))
        .css_classes(["cc-scale"])
        .hexpand(true)
        .valign(Align::Center)
        .build();

    let mica_pct = Label::builder()
        .label(&format!("Mica: {}%", data.mica_tint.round() as i32))
        .css_classes(["cc-label-main"])
        .build();

    let mica_pct_clone = mica_pct.clone();
    mica_slider.connect_value_changed(move |s| {
        mica_pct_clone.set_label(&format!("Mica: {}%", s.value().round() as i32));
    });

    mica_row.append(&mica_icon);
    mica_row.append(&mica_slider);
    mica_row.append(&mica_pct);

    // Toggles Row: True Tone & Dark Mode
    let toggles_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .homogeneous(true)
        .build();

    // True Tone
    let tt_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .valign(Align::Center)
        .build();
    let tt_lbl = Label::builder()
        .label("󰛨 True Tone")
        .css_classes(["cc-label-main"])
        .hexpand(true)
        .halign(Align::Start)
        .build();
    let tt_sw = Switch::builder()
        .active(data.true_tone)
        .valign(Align::Center)
        .build();
    tt_sw.connect_state_set(move |_, state| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::ToggleTrueTone(state));
        glib::Propagation::Proceed
    });
    tt_box.append(&tt_lbl);
    tt_box.append(&tt_sw);

    // Dark Mode
    let dark_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .valign(Align::Center)
        .build();
    let dark_lbl = Label::builder()
        .label("☾ Dark Mode")
        .css_classes(["cc-label-main"])
        .hexpand(true)
        .halign(Align::Start)
        .build();
    let dark_sw = Switch::builder()
        .active(data.dark_mode)
        .valign(Align::Center)
        .build();
    dark_sw.connect_state_set(move |_, _| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetDarkMode);
        glib::Propagation::Proceed
    });
    dark_box.append(&dark_lbl);
    dark_box.append(&dark_sw);

    toggles_row.append(&tt_box);
    toggles_row.append(&dark_box);

    container.append(&bright_row);
    container.append(&mica_row);
    container.append(&toggles_row);
    container
}
