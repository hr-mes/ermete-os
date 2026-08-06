use crate::core::system_proxies::{subscribe_system_events, SystemEvent};
use crate::ui::control_center::audio::show_audio_mixer_popover;
use crate::ui::control_center::bluetooth::show_bluetooth_popover;
use crate::ui::control_center::sysmon::show_system_monitor_modal;
use crate::ui::control_center::widgets::{build_cc_compact_tile, build_cc_row, build_cc_row_content};
use crate::ui::control_center::wifi::show_wifi_popover;
use crate::ui::popup_manager::setup_popup_autoclose;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Scale};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::process::Command;

fn build_quick_toggle_content(icon: &str, text: &str) -> GtkBox {
    let box_ = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).halign(Align::Center).build();
    box_.append(&Label::builder().label(icon).build());
    box_.append(&Label::builder().label(text).build());
    box_
}

pub fn show_control_center_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Control Center")
        .css_classes(["popup-window"])
        .default_width(350)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "control-center");
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

    // 0. HEADER BAR (Control Center title + System Settings button)
    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .valign(Align::Center)
        .build();
    let cc_title_lbl = Label::builder()
        .label("Control Center")
        .css_classes(["cc-label-main"])
        .hexpand(true)
        .halign(Align::Start)
        .build();
    let settings_btn = Button::builder()
        .label("⚙ Impostazioni")
        .css_classes(["cc-quick-btn"])
        .tooltip_text("Impostazioni di Sistema")
        .build();
    settings_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs");
    }));
    header_box.append(&cc_title_lbl);
    header_box.append(&settings_btn);

    // 1. TOP SECTION (Grid a 2 Colonne)
    let top_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    // Colonna Sinistra (Connettività) - Initialized passively from cached state
    let conn_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["cc-tile"])
        .hexpand(true)
        .build();

    let (net_icon, net_title, net_sub) = crate::core::get_network_controller().get_cached_network_status();
    let wifi_btn = build_cc_row("cc-circle-blue", &net_icon, &net_title, &net_sub);
    wifi_btn.set_hexpand(true);
    let net_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
    if net_connected {
        wifi_btn.add_css_class("cc-btn-active");
    }

    wifi_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        show_wifi_popover(&app);
    }));
    let wifi_row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    let wifi_settings_btn = Button::builder()
        .label("⚙")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Impostazioni Wi-Fi")
        .build();
    wifi_settings_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "wifi"));
    }));
    wifi_row_box.append(&wifi_btn);
    wifi_row_box.append(&wifi_settings_btn);

    let bt_btn = build_cc_row("cc-circle-blue", "", "Bluetooth", "Dispositivi");
    bt_btn.set_hexpand(true);
    bt_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        show_bluetooth_popover(&app);
    }));
    let bt_row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    let bt_settings_btn = Button::builder()
        .label("⚙")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Impostazioni Bluetooth")
        .build();
    bt_settings_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "bluetooth"));
    }));
    bt_row_box.append(&bt_btn);
    bt_row_box.append(&bt_settings_btn);

    let sys_btn = build_cc_row("cc-circle-blue", "⚙", "Risorse", "Monitor Live");
    sys_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        show_system_monitor_modal(&app);
    }));

    conn_box.append(&wifi_row_box);
    conn_box.append(&bt_row_box);
    conn_box.append(&sys_btn);

    // Colonna Destra (2 Card verticali)
    let right_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .homogeneous(true)
        .hexpand(true)
        .build();

    let screenshot_tile = build_cc_compact_tile("cc-circle-indigo", "📷", "Screenshot");
    screenshot_tile.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        glib::MainContext::default().spawn_local(async move {
            ermete_niri_ipc::async_client::screenshot().await;
        });

    }));

    let lock_tile = build_cc_compact_tile("cc-circle-blue", "🔒", "Blocca");
    lock_tile.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_power_controller();
            let _ = ctrl.lock_screen().await;
        });
    }));

    right_col.append(&screenshot_tile);
    right_col.append(&lock_tile);

    top_grid.append(&conn_box);
    top_grid.append(&right_col);

    // 2. MIDDLE SECTION (Slider Apple-Style)
    // Slider Luminosità - Passive initialization from cached state
    let init_brightness = crate::core::live_state::get_live_state().brightness;
    let bright_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let bright_icon = Label::builder().label("☀").css_classes(["cc-slider-icon"]).build();
    let bright_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    bright_slider.set_value(if init_brightness > 0.0 { init_brightness } else { 75.0 });
    bright_slider.set_hexpand(true);
    bright_slider.set_valign(Align::Center);
    bright_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_display_controller();
            let _ = ctrl.set_brightness(val).await;
        });
    });
    bright_card.append(&bright_icon);
    bright_card.append(&bright_slider);
    let disp_settings_btn = Button::builder()
        .label("⚙")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Impostazioni Schermi")
        .build();
    disp_settings_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "displays"));
    }));

    let tt_btn = Button::builder()
        .label("☾")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("True Tone")
        .build();
    let tt_btn_clone_click = tt_btn.clone();
    tt_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        let is_active = tt_btn_clone_click.has_css_class("cc-btn-active");
        let new_state = !is_active;
        if new_state {
            tt_btn_clone_click.add_css_class("cc-btn-active");
        } else {
            tt_btn_clone_click.remove_css_class("cc-btn-active");
        }
        glib::MainContext::default().spawn_local(async move {
            if let Ok(connection) = zbus::Connection::session().await {
                let _ = connection.call_method(
                    Some("org.ermete.Settings"),
                    "/org/ermete/Settings",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.ermete.Settings", "TrueToneEnabled", zbus::zvariant::Value::from(new_state))
                ).await;
            }
        });
    }));

    bright_card.append(&tt_btn);
    bright_card.append(&disp_settings_btn);

    // Slider Volume Audio - Passive initialization from cached state
    let init_volume = crate::core::get_audio_controller().get_cached_volume() * 100.0;
    let audio_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let audio_icon = Label::builder().label("🔊").css_classes(["cc-slider-icon"]).build();
    let audio_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    audio_slider.set_value(if init_volume > 0.0 { init_volume } else { 80.0 });
    audio_slider.set_hexpand(true);
    audio_slider.set_valign(Align::Center);
    audio_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_audio_controller();
            let _ = ctrl.set_volume(val).await;
        });
    });
    audio_card.append(&audio_icon);
    audio_card.append(&audio_slider);

    let audio_settings_btn = Button::builder()
        .label("⚙")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Impostazioni Audio")
        .build();
    audio_settings_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "audio"));
    }));
    audio_card.append(&audio_settings_btn);

    // 3. MEDIA CONTROL (MPRIS) - Passive initialization
    let mpris_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(["cc-tile"])
        .build();
    let initial_mpris = crate::core::get_mpris_controller().get_cached_mpris_state();
    let (init_mpris_title, init_mpris_artist, init_mpris_btn) = match &initial_mpris {
        Some(m) => (m.title.clone(), m.artist.clone(), if m.status.contains("Playing") { "⏸" } else { "▶" }),
        None => ("Nessun media in riproduzione".to_string(), "-".to_string(), "▶"),
    };
    let mpris_title = Label::builder().label(&init_mpris_title).css_classes(["cc-label-main"]).halign(Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();
    let mpris_artist = Label::builder().label(&init_mpris_artist).css_classes(["cc-label-sub"]).halign(Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();
    let mpris_ctrl_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(12).halign(Align::Center).build();
    let prev_btn = Button::builder().label("⏮").css_classes(["cc-quick-btn"]).build();
    let play_btn = Button::builder().label(init_mpris_btn).css_classes(["cc-quick-btn"]).build();
    let next_btn = Button::builder().label("⏭").css_classes(["cc-quick-btn"]).build();
    
    prev_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_mpris_controller();
            let _ = ctrl.player_command("previous").await;
        });
    }));
    play_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_mpris_controller();
            let _ = ctrl.player_command("play-pause").await;
        });
    }));
    next_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_mpris_controller();
            let _ = ctrl.player_command("next").await;
        });
    }));

    mpris_ctrl_box.append(&prev_btn);
    mpris_ctrl_box.append(&play_btn);
    mpris_ctrl_box.append(&next_btn);
    
    mpris_card.append(&mpris_title);
    mpris_card.append(&mpris_artist);
    mpris_card.append(&mpris_ctrl_box);

    // 4. BOTTOM SECTION (4 Quick Toggles Grid)
    let bottom_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .homogeneous(true)
        .build();

    let dark_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    dark_btn.set_child(Some(&build_quick_toggle_content("☾", "Scuro")));
    crate::core::attach_voiceover_hover(&dark_btn, "Attiva o disattiva la modalità scura");

    dark_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        let settings = gtk4::gio::Settings::new("org.gnome.desktop.interface");
        let _ = settings.set_string("color-scheme", "prefer-dark");
    }));

    let standby_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    standby_btn.set_child(Some(&build_quick_toggle_content("🖥", "Standby")));
    crate::core::attach_voiceover_hover(&standby_btn, "Sospendi il computer");

    standby_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        glib::MainContext::default().spawn_local(async move {
            ermete_niri_ipc::async_client::power_off_monitors().await;
            let ctrl = crate::core::get_power_controller();
            let _ = ctrl.suspend().await;
        });
    }));


    let mixer_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    mixer_btn.set_child(Some(&build_quick_toggle_content("🎚️", "Mixer")));
    crate::core::attach_voiceover_hover(&mixer_btn, "Apri il mixer audio avanzato");

    mixer_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        show_audio_mixer_popover(&app);
    }));

    let term_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    term_btn.set_child(Some(&build_quick_toggle_content("", "Shell")));
    crate::core::attach_voiceover_hover(&term_btn, "Apri un terminale");

    term_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        let _ = Command::new("foot").spawn();
    }));

    bottom_grid.append(&dark_btn);
    bottom_grid.append(&standby_btn);
    bottom_grid.append(&mixer_btn);
    bottom_grid.append(&term_btn);

    card.append(&header_box);
    card.append(&top_grid);
    card.append(&bright_card);
    card.append(&audio_card);
    card.append(&mpris_card);
    card.append(&bottom_grid);

    // 100% PASSIVE REACTIVE SUBSCRIPTION (Zero active polling, Zero blocking calls)
    let bright_slider_clone = bright_slider.clone();
    let audio_slider_clone = audio_slider.clone();
    let mpris_t = mpris_title.clone();
    let mpris_a = mpris_artist.clone();
    let mpris_p = play_btn.clone();
    let wifi_btn_clone = wifi_btn.clone();
    let bt_btn_clone = bt_btn.clone();

    let mut rx = subscribe_system_events();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                SystemEvent::NetworkUpdated(_) | SystemEvent::WifiToggled(_) => {
                    let (net_icon, net_title, net_sub) = crate::core::get_network_controller().get_cached_network_status();
                    let net_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
                    if net_connected {
                        wifi_btn_clone.add_css_class("cc-btn-active");
                    } else {
                        wifi_btn_clone.remove_css_class("cc-btn-active");
                    }
                    wifi_btn_clone.set_child(Some(&build_cc_row_content("cc-circle-blue", &net_icon, &net_title, &net_sub)));
                }
                SystemEvent::BluetoothToggled(enabled) => {
                    if enabled {
                        bt_btn_clone.add_css_class("cc-btn-active");
                    } else {
                        bt_btn_clone.remove_css_class("cc-btn-active");
                    }
                }
                SystemEvent::BrightnessChanged(val) => {
                    if (bright_slider_clone.value() - val * 100.0).abs() > 1.5 {
                        bright_slider_clone.set_value(val * 100.0);
                    }
                }
                SystemEvent::VolumeChanged(val) => {
                    if (audio_slider_clone.value() - val * 100.0).abs() > 1.5 {
                        audio_slider_clone.set_value(val * 100.0);
                    }
                }
                SystemEvent::MprisUpdated(mpris_opt) => {
                    if let Some(mpris) = mpris_opt {
                        mpris_t.set_label(&mpris.title);
                        mpris_a.set_label(&mpris.artist);
                        if mpris.status.contains("Playing") {
                            mpris_p.set_label("⏸");
                        } else {
                            mpris_p.set_label("▶");
                        }
                    } else {
                        mpris_t.set_label("Nessun media in riproduzione");
                        mpris_a.set_label("-");
                        mpris_p.set_label("▶");
                    }
                }
                _ => {}
            }
        }
    });

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);


    pop.set_child(Some(&card));
    pop.present();
}
