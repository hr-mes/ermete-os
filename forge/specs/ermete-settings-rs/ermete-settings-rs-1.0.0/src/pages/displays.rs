use gtk4::prelude::*;
use gtk4::{Align, Box, ComboBoxText, DrawingArea, Label, Orientation, Scale, Switch};
use crate::components::action_row::ActionRow;
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("<span size='large' weight='bold'>Schermi e Display (Niri Wayland)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Multi-monitor Spatial Layout Preview
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

    let outputs_store = Rc::new(RefCell::new(Vec::<String>::new()));
    let outputs_store_draw = outputs_store.clone();

    drawing_area.set_draw_func(move |_area, cr, width, height| {
        cr.set_source_rgb(0.12, 0.14, 0.18);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();

        let store = outputs_store_draw.borrow();
        let count = store.len().max(1);
        let box_w = ((width as f64 - 40.0) / count as f64).min(180.0);
        let box_h = 90.0;
        let start_x = (width as f64 - (box_w * count as f64 + (count as f64 - 1.0) * 16.0)) / 2.0;

        if store.is_empty() {
            let x = start_x;
            let y = (height as f64 - box_h) / 2.0;
            cr.set_source_rgb(0.22, 0.55, 0.88);
            cr.rectangle(x, y, box_w, box_h);
            let _ = cr.fill();

            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_font_size(13.0);
            cr.move_to(x + 12.0, y + box_h / 2.0);
            let _ = cr.show_text("eDP-1");
        } else {
            for (i, name) in store.iter().enumerate() {
                let x = start_x + i as f64 * (box_w + 16.0);
                let y = (height as f64 - box_h) / 2.0;

                cr.set_source_rgb(0.22, 0.55, 0.88);
                cr.rectangle(x, y, box_w, box_h);
                let _ = cr.fill();

                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_font_size(13.0);
                cr.move_to(x + 12.0, y + box_h / 2.0);
                let _ = cr.show_text(name);
            }
        }
    });

    layout_frame.append(&layout_title);
    layout_frame.append(&drawing_area);
    container.append(&layout_frame);

    // Output selector
    let monitor_combo = ComboBoxText::new();

    let combo_ref = monitor_combo.clone();
    let da_ref = drawing_area.clone();
    let outputs_store_async = outputs_store.clone();

    relm4::spawn_local(async move {
        let outputs = ermete_niri_ipc::async_client::get_outputs().await;
        {
            let mut store = outputs_store_async.borrow_mut();
            *store = outputs.clone();
        }
        combo_ref.remove_all();
        for output in &outputs {
            combo_ref.append_text(output);
        }
        if !outputs.is_empty() {
            combo_ref.set_active(Some(0));
        }
        da_ref.queue_draw();
    });

    let monitor_row = ActionRow::builder("Schermo Selezionato")
        .subtitle("Seleziona il display Wayland da configurare")
        .suffix(&monitor_combo)
        .build();
    container.append(&monitor_row);

    // Fractional Scaling
    let scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.0, 0.1);
    scale.set_value(1.0);
    scale.set_draw_value(true);
    scale.set_hexpand(true);

    let combo_scale_out = monitor_combo.clone();
    scale.connect_value_changed(move |s| {
        let selected_output = match combo_scale_out.active_text() {
            Some(t) => t.to_string(),
            None => "eDP-1".to_string(),
        };
        let val = s.value();
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::set_output_scale(&selected_output, val).await;
        });
    });

    let scale_row = ActionRow::builder("Scala di Visualizzazione")
        .subtitle("Fractional Scaling (1.0x - 2.0x)")
        .suffix(&scale)
        .build();
    container.append(&scale_row);

    // Resolution
    let res_combo = ComboBoxText::new();
    res_combo.append_text("3840x2160 (4K UHD)");
    res_combo.append_text("2560x1440 (QHD)");
    res_combo.append_text("1920x1080 (FHD)");
    res_combo.append_text("1280x800");
    res_combo.set_active(Some(2));

    let combo_res_out = monitor_combo.clone();
    res_combo.connect_changed(move |c| {
        if let Some(mode) = c.active_text() {
            let out = match combo_res_out.active_text() {
                Some(t) => t.to_string(),
                None => "eDP-1".to_string(),
            };
            let clean_mode = match mode.split_whitespace().next() {
                Some(m) => m.to_string(),
                None => "1920x1080".to_string(),
            };
            relm4::spawn_local(async move {
                ermete_niri_ipc::async_client::set_output_mode(&out, &clean_mode).await;
            });
        }
    });

    let res_row = ActionRow::builder("Risoluzione")
        .subtitle("Risoluzione nativa dello schermo")
        .suffix(&res_combo)
        .build();
    container.append(&res_row);

    // Refresh Rate
    let hz_combo = ComboBoxText::new();
    hz_combo.append_text("60 Hz");
    hz_combo.append_text("120 Hz");
    hz_combo.append_text("144 Hz");
    hz_combo.append_text("165 Hz");
    hz_combo.append_text("240 Hz");
    hz_combo.set_active(Some(1));

    let hz_row = ActionRow::builder("Frequenza di Aggiornamento")
        .subtitle("Frequenza del display in Hz")
        .suffix(&hz_combo)
        .build();
    container.append(&hz_row);

    // VRR (Variable Refresh Rate)
    let vrr_switch = Switch::builder().valign(Align::Center).build();
    let combo_vrr_out = monitor_combo.clone();

    vrr_switch.connect_state_set(move |_, state| {
        let out = match combo_vrr_out.active_text() {
            Some(t) => t.to_string(),
            None => "eDP-1".to_string(),
        };
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::set_output_vrr(&out, state).await;
        });
        glib::Propagation::Proceed
    });

    let vrr_row = ActionRow::builder("Frequenza Variabile (VRR)")
        .subtitle("Adaptive Sync / FreeSync / G-Sync")
        .suffix(&vrr_switch)
        .build();
    container.append(&vrr_row);

    // HDR
    let hdr_switch = Switch::builder().valign(Align::Center).build();
    let combo_hdr_out = monitor_combo.clone();

    hdr_switch.connect_state_set(move |_, state| {
        let out = match combo_hdr_out.active_text() {
            Some(t) => t.to_string(),
            None => "eDP-1".to_string(),
        };
        relm4::spawn_local(async move {
            ermete_niri_ipc::async_client::set_output_hdr(&out, state).await;
        });
        glib::Propagation::Proceed
    });

    let hdr_row = ActionRow::builder("High Dynamic Range (HDR)")
        .subtitle("Profondità colore a 10-bit per canale")
        .suffix(&hdr_switch)
        .build();
    container.append(&hdr_row);

    // ColorSync & True Tone
    let tt_title = Label::builder()
        .label("<b>ColorSync &amp; True Tone</b>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(16)
        .build();
    container.append(&tt_title);

    let tt_switch = Switch::builder().valign(Align::Center).build();
    let tt_sw_clone = tt_switch.clone();

    let temp_scale = Scale::with_range(Orientation::Horizontal, 3000.0, 6500.0, 100.0);
    temp_scale.set_value(4500.0);
    temp_scale.set_draw_value(true);
    temp_scale.set_hexpand(true);
    let temp_scale_clone = temp_scale.clone();

    tt_switch.connect_state_set(move |_, state| {
        relm4::spawn_local(async move {
            if let Ok(connection) = crate::get_connection().await {
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
        relm4::spawn_local(async move {
            if let Ok(connection) = crate::get_connection().await {
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

    relm4::spawn_local(async move {
        if let Ok(connection) = crate::get_connection().await {
            if let Ok(msg) = connection.call_method(
                Some("org.ermete.Settings"),
                "/org/ermete/Settings",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.ermete.Settings", "TrueToneEnabled")
            ).await {
                if let Ok(val) = msg.body().deserialize::<zbus::zvariant::OwnedValue>() {
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
                if let Ok(val) = msg.body().deserialize::<zbus::zvariant::OwnedValue>() {
                    if let Ok(temp) = u32::try_from(val) {
                        temp_scale_clone.set_value(temp as f64);
                    }
                }
            }
        }
    });

    let tt_row = ActionRow::builder("True Tone")
        .subtitle("Adatta automaticamente i colori per non affaticare la vista")
        .suffix(&tt_switch)
        .build();
    container.append(&tt_row);

    let temp_row = ActionRow::builder("Temperatura Colore (Kelvin)")
        .subtitle("Regola il calore dello schermo (3000K - 6500K)")
        .suffix(&temp_scale)
        .build();
    container.append(&temp_row);

    container
}
