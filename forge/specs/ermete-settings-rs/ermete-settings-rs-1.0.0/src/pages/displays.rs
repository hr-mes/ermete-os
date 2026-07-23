use gtk4::prelude::*;
use gtk4::{Align, Box, ComboBoxText, DrawingArea, Label, Orientation, Scale, Switch};

fn get_niri_outputs() -> Vec<String> {
    crate::niri_client::get_outputs()
}

pub fn build_page() -> Box {
    let container = Box::new(Orientation::Vertical, 20);
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(24);
    container.set_margin_end(24);

    let title = Label::builder()
        .label("<span size='large' weight='bold'>Schermi e Display (Niri Wayland)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Multi-monitor Spatial Layout Preview (Drag-and-Drop / Arrangement visualizer)
    let layout_frame = Box::new(Orientation::Vertical, 8);
    let layout_title = Label::builder()
        .label("<b>Disposizione Schermi e Posizionamento Spaziale:</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    
    let drawing_area = DrawingArea::builder()
        .content_width(400)
        .content_height(140)
        .css_classes(["card", "display-layout-preview"])
        .build();

    let outputs_clone = get_niri_outputs();
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        cr.set_source_rgb(0.12, 0.14, 0.18);
        let _ = cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();

        let count = outputs_clone.len().max(1);
        let box_w = ((width as f64 - 40.0) / count as f64).min(180.0);
        let box_h = 90.0;
        let start_x = (width as f64 - (box_w * count as f64 + (count as f64 - 1.0) * 16.0)) / 2.0;

        for (i, name) in outputs_clone.iter().enumerate() {
            let x = start_x + i as f64 * (box_w + 16.0);
            let y = (height as f64 - box_h) / 2.0;

            // Monitor border/background
            cr.set_source_rgb(0.22, 0.55, 0.88);
            let _ = cr.rectangle(x, y, box_w, box_h);
            let _ = cr.fill();

            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_font_size(13.0);
            let _ = cr.move_to(x + 12.0, y + box_h / 2.0);
            let _ = cr.show_text(name);
        }
    });

    layout_frame.append(&layout_title);
    layout_frame.append(&drawing_area);
    container.append(&layout_frame);

    // Output selector
    let monitor_box = Box::new(Orientation::Horizontal, 12);
    let monitor_label = Label::builder()
        .label("Schermo Selezionato:")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    let monitor_combo = ComboBoxText::new();

    let outputs = get_niri_outputs();
    for output in &outputs {
        monitor_combo.append_text(output);
    }
    monitor_combo.set_active(Some(0));
    monitor_box.append(&monitor_label);
    monitor_box.append(&monitor_combo);
    container.append(&monitor_box);

    // Fractional Scaling
    let scale_box = Box::new(Orientation::Vertical, 8);
    let scale_label = Label::builder()
        .label("Scala di Visualizzazione (Fractional Scaling):")
        .halign(Align::Start)
        .build();

    let scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.0, 0.1);
    scale.set_value(1.0);
    scale.set_draw_value(true);

    let combo_clone = monitor_combo.clone();
    scale.connect_value_changed(move |s| {
        let selected_output = combo_clone
            .active_text()
            .map(|t| t.to_string())
            .unwrap_or_else(|| "eDP-1".to_string());
        let val = s.value();
        crate::niri_client::set_output_scale(&selected_output, val);
    });

    scale_box.append(&scale_label);
    scale_box.append(&scale);
    container.append(&scale_box);

    // Resolution & Refresh Rate row
    let res_hz_box = Box::new(Orientation::Horizontal, 24);
    
    let res_box = Box::new(Orientation::Horizontal, 12);
    let res_lbl = Label::builder().label("Risoluzione:").build();
    let res_combo = ComboBoxText::new();
    res_combo.append_text("3840x2160 (4K UHD)");
    res_combo.append_text("2560x1440 (QHD)");
    res_combo.append_text("1920x1080 (FHD)");
    res_combo.append_text("1280x800");
    res_combo.set_active(Some(2));
    let combo_res_out = monitor_combo.clone();
    res_combo.connect_changed(move |c| {
        if let Some(mode) = c.active_text() {
            let out = combo_res_out.active_text().map(|t| t.to_string()).unwrap_or_else(|| "eDP-1".to_string());
            let clean_mode = mode.split_whitespace().next().unwrap_or("1920x1080");
            crate::niri_client::set_output_mode(&out, clean_mode);
        }
    });
    res_box.append(&res_lbl);
    res_box.append(&res_combo);

    let hz_box = Box::new(Orientation::Horizontal, 12);
    let hz_lbl = Label::builder().label("Frequenza (Hz):").build();
    let hz_combo = ComboBoxText::new();
    hz_combo.append_text("60 Hz");
    hz_combo.append_text("120 Hz");
    hz_combo.append_text("144 Hz");
    hz_combo.append_text("165 Hz");
    hz_combo.append_text("240 Hz");
    hz_combo.set_active(Some(1));
    hz_box.append(&hz_lbl);
    hz_box.append(&hz_combo);

    res_hz_box.append(&res_box);
    res_hz_box.append(&hz_box);
    container.append(&res_hz_box);

    // VRR (Variable Refresh Rate / Adaptive Sync)
    let vrr_box = Box::new(Orientation::Horizontal, 12);
    let vrr_lbl = Label::builder()
        .label("Frequenza di Aggiornamento Variabile (VRR / Adaptive Sync - FreeSync/G-Sync)")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    let vrr_switch = Switch::builder().valign(Align::Center).build();
    let combo_vrr_out = monitor_combo.clone();
    vrr_switch.connect_state_set(move |_, state| {
        let out = combo_vrr_out.active_text().map(|t| t.to_string()).unwrap_or_else(|| "eDP-1".to_string());
        crate::niri_client::set_output_vrr(&out, state);
        glib::Propagation::Proceed
    });
    vrr_box.append(&vrr_lbl);
    vrr_box.append(&vrr_switch);
    container.append(&vrr_box);

    // HDR (High Dynamic Range) & 10-Bit Color Depth
    let hdr_box = Box::new(Orientation::Horizontal, 12);
    let hdr_lbl = Label::builder()
        .label("High Dynamic Range (HDR / Colore a 10-bit per canale)")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    let hdr_switch = Switch::builder().valign(Align::Center).build();
    let combo_hdr_out = monitor_combo.clone();
    hdr_switch.connect_state_set(move |_, state| {
        let out = combo_hdr_out.active_text().map(|t| t.to_string()).unwrap_or_else(|| "eDP-1".to_string());
        crate::niri_client::set_output_hdr(&out, state);
        glib::Propagation::Proceed
    });
    hdr_box.append(&hdr_lbl);
    hdr_box.append(&hdr_switch);
    container.append(&hdr_box);

    // True Tone & ColorSync
    let tt_title = Label::builder()
        .label("<b>ColorSync & True Tone</b>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(24)
        .build();
    container.append(&tt_title);

    let tt_box = Box::new(Orientation::Horizontal, 12);
    let tt_lbl = Label::builder()
        .label("True Tone (Adatta automaticamente i colori per non affaticare la vista)")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    let tt_switch = Switch::builder().valign(Align::Center).build();
    let tt_sw_clone = tt_switch.clone();

    let temp_box = Box::new(Orientation::Vertical, 8);
    temp_box.set_margin_start(16);
    let temp_lbl = Label::builder()
        .label("Temperatura Colore (Kelvin):")
        .halign(Align::Start)
        .build();
    let temp_scale = Scale::with_range(Orientation::Horizontal, 3000.0, 6500.0, 100.0);
    temp_scale.set_value(4500.0);
    temp_scale.set_draw_value(true);
    let temp_scale_clone = temp_scale.clone();

    tt_switch.connect_state_set(move |_, state| {
        glib::MainContext::default().spawn_local(async move {
            if let Ok(connection) = zbus::Connection::session().await {
                let _ = connection.call_method(
                    Some("org.ermete.Settings"),
                    "/org/ermete/Settings",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.ermete.Settings", "TrueToneEnabled", zbus::zvariant::Value::from(state))
                ).await;
            }
        });
        glib::Propagation::Proceed
    });

    temp_scale.connect_value_changed(move |s| {
        let val = s.value() as u32;
        glib::MainContext::default().spawn_local(async move {
            if let Ok(connection) = zbus::Connection::session().await {
                let _ = connection.call_method(
                    Some("org.ermete.Settings"),
                    "/org/ermete/Settings",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.ermete.Settings", "TrueToneTemperature", zbus::zvariant::Value::from(val))
                ).await;
            }
        });
    });

    glib::MainContext::default().spawn_local(async move {
        if let Ok(connection) = zbus::Connection::session().await {
            if let Ok(msg) = connection.call_method(
                Some("org.ermete.Settings"),
                "/org/ermete/Settings",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.ermete.Settings", "TrueToneEnabled")
            ).await {
                if let Ok(val) = msg.body::<zbus::zvariant::OwnedValue>() {
                    if let Ok(enabled) = bool::try_from(val) {
                        tt_sw_clone.set_active(enabled);
                    }
                }
            }

            if let Ok(msg) = connection.call_method(
                Some("org.ermete.Settings"),
                "/org/ermete/Settings",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.ermete.Settings", "TrueToneTemperature")
            ).await {
                if let Ok(val) = msg.body::<zbus::zvariant::OwnedValue>() {
                    if let Ok(temp) = u32::try_from(val) {
                        temp_scale_clone.set_value(temp as f64);
                    }
                }
            }
        }
    });

    tt_box.append(&tt_lbl);
    tt_box.append(&tt_switch);
    temp_box.append(&temp_lbl);
    temp_box.append(&temp_scale);

    container.append(&tt_box);
    container.append(&temp_box);

    container
}

