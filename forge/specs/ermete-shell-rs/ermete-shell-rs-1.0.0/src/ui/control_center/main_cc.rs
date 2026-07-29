use crate::core::*;
use crate::ui::control_center::audio::show_audio_mixer_popover;
use crate::ui::control_center::bluetooth::show_bluetooth_popover;
use crate::ui::control_center::sysmon::show_system_monitor_modal;
use crate::ui::control_center::widgets::{build_cc_compact_tile, build_cc_row};
use crate::ui::control_center::wifi::show_wifi_popover;
use crate::ui::topbar::setup_popup_autoclose;
use glib::clone;
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
    let pop_settings = pop.clone();
    settings_btn.connect_clicked(move |_| {
        pop_settings.close();
        let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs");
    });
    header_box.append(&cc_title_lbl);
    header_box.append(&settings_btn);

    // 1. TOP SECTION (Grid a 2 Colonne)
    let top_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    // Colonna Sinistra (Connettività)
    let conn_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["cc-tile"])
        .hexpand(true)
        .build();

    let (net_icon, net_title, net_sub) = get_network_status();
    let wifi_btn = build_cc_row("cc-circle-blue", &net_icon, &net_title, &net_sub);
    wifi_btn.set_hexpand(true);
    let net_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
    if net_connected {
        wifi_btn.add_css_class("cc-btn-active");
    }
    let app_wifi = app.clone();
    let pop_wifi = pop.clone();
    wifi_btn.connect_clicked(move |_| {
        pop_wifi.close();
        show_wifi_popover(&app_wifi);
    });
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
    let pop_wifi_s = pop.clone();
    wifi_settings_btn.connect_clicked(move |_| {
        pop_wifi_s.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "wifi"));
    });
    wifi_row_box.append(&wifi_btn);
    wifi_row_box.append(&wifi_settings_btn);

    let bt_btn = build_cc_row("cc-circle-blue", "", "Bluetooth", "Dispositivi");
    bt_btn.set_hexpand(true);
    let bt_btn_clone_init = bt_btn.clone();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::system_proxies::get_global_controller();
        if let Ok(enabled) = ctrl.is_bluetooth_enabled().await {
            if enabled {
                bt_btn_clone_init.add_css_class("cc-btn-active");
            }
        }
    });
    let app_bt = app.clone();
    let pop_bt = pop.clone();
    bt_btn.connect_clicked(move |_| {
        pop_bt.close();
        show_bluetooth_popover(&app_bt);
    });
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
    let pop_bt_s = pop.clone();
    bt_settings_btn.connect_clicked(move |_| {
        pop_bt_s.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "bluetooth"));
    });
    bt_row_box.append(&bt_btn);
    bt_row_box.append(&bt_settings_btn);

    let sys_btn = build_cc_row("cc-circle-blue", "⚙", "Risorse", "Monitor Live");
    let app_sys = app.clone();
    let pop_sys = pop.clone();
    sys_btn.connect_clicked(move |_| {
        pop_sys.close();
        show_system_monitor_modal(&app_sys);
    });

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
    let pop_shot = pop.clone();
    screenshot_tile.connect_clicked(move |_| {
        pop_shot.close();
        crate::core::niri_client::screenshot();
    });

    let lock_tile = build_cc_compact_tile("cc-circle-blue", "🔒", "Blocca");
    let pop_lock = pop.clone();
    lock_tile.connect_clicked(move |_| {
        pop_lock.close();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.lock_screen().await;
        });
    });

    right_col.append(&screenshot_tile);
    right_col.append(&lock_tile);

    top_grid.append(&conn_box);
    top_grid.append(&right_col);

    // 2. MIDDLE SECTION (Slider Apple-Style)
    // Slider Luminosità
    let bright_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let bright_icon = Label::builder().label("☀").css_classes(["cc-slider-icon"]).build();
    let bright_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    bright_slider.set_value(75.0);
    bright_slider.set_hexpand(true);
    bright_slider.set_valign(Align::Center);
    bright_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
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
    let pop_disp_s = pop.clone();
    disp_settings_btn.connect_clicked(move |_| {
        pop_disp_s.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "displays"));
    });

    let tt_btn = Button::builder()
        .label("☾")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("True Tone")
        .build();
    let tt_btn_clone_click = tt_btn.clone();
    tt_btn.connect_clicked(move |_| {
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
    });
    let tt_btn_clone_init = tt_btn.clone();
    glib::MainContext::default().spawn_local(async move {
        if let Ok(connection) = zbus::Connection::session().await {
            if let Ok(msg) = connection.call_method(
                Some("org.ermete.Settings"),
                "/org/ermete/Settings",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.ermete.Settings", "TrueToneEnabled")
            ).await {
                if let Ok(val) = msg.body().deserialize::<zbus::zvariant::OwnedValue>() {
                    if let Ok(enabled) = bool::try_from(val) {
                        if enabled {
                            tt_btn_clone_init.add_css_class("cc-btn-active");
                        }
                    }
                }
            }
        }
    });

    bright_card.append(&tt_btn);
    bright_card.append(&disp_settings_btn);

    // Slider Volume Audio
    let audio_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let audio_icon = Label::builder().label("🔊").css_classes(["cc-slider-icon"]).build();
    let audio_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    audio_slider.set_value(80.0);
    audio_slider.set_hexpand(true);
    audio_slider.set_valign(Align::Center);
    audio_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.set_volume(val).await;
        });
    });
    audio_card.append(&audio_icon);
    audio_card.append(&audio_slider);

    let bright_slider_clone_init = bright_slider.clone();
    let audio_slider_clone_init = audio_slider.clone();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::system_proxies::get_global_controller();
        if let Ok(b) = ctrl.get_brightness().await {
            bright_slider_clone_init.set_value(b * 100.0);
        }
        if let Ok(v) = ctrl.get_volume().await {
            audio_slider_clone_init.set_value(v * 100.0);
        }
    });
    let audio_settings_btn = Button::builder()
        .label("⚙")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Impostazioni Audio")
        .build();
    let pop_audio_s = pop.clone();
    audio_settings_btn.connect_clicked(move |_| {
        pop_audio_s.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "audio"));
    });
    audio_card.append(&audio_settings_btn);

    // 3. MEDIA CONTROL (MPRIS)
    let mpris_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(["cc-tile"])
        .build();
    let mpris_title = Label::builder().label("Nessun media in riproduzione").css_classes(["cc-label-main"]).halign(Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();
    let mpris_artist = Label::builder().label("-").css_classes(["cc-label-sub"]).halign(Align::Start).hexpand(true).ellipsize(gtk4::pango::EllipsizeMode::End).build();
    let mpris_ctrl_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(12).halign(Align::Center).build();
    let prev_btn = Button::builder().label("⏮").css_classes(["cc-quick-btn"]).build();
    let play_btn = Button::builder().label("▶").css_classes(["cc-quick-btn"]).build();
    let next_btn = Button::builder().label("⏭").css_classes(["cc-quick-btn"]).build();
    
    prev_btn.connect_clicked(|_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.player_command("previous").await;
        });
    });
    play_btn.connect_clicked(|_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.player_command("play-pause").await;
        });
    });
    next_btn.connect_clicked(|_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.player_command("next").await;
        });
    });

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

    dark_btn.connect_clicked(move |_| {
        let settings = gtk4::gio::Settings::new("org.gnome.desktop.interface");
        let _ = settings.set_string("color-scheme", "prefer-dark");
    });

    let standby_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    standby_btn.set_child(Some(&build_quick_toggle_content("🖥", "Standby")));
    crate::core::attach_voiceover_hover(&standby_btn, "Sospendi il computer");

    let pop_std = pop.clone();
    standby_btn.connect_clicked(move |_| {
        pop_std.close();
        crate::core::niri_client::power_off_monitors();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.suspend().await;
        });
    });

    let mixer_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    mixer_btn.set_child(Some(&build_quick_toggle_content("🎚️", "Mixer")));
    crate::core::attach_voiceover_hover(&mixer_btn, "Apri il mixer audio avanzato");

    let app_mixer = app.clone();
    let pop_mixer = pop.clone();
    mixer_btn.connect_clicked(move |_| {
        pop_mixer.close();
        show_audio_mixer_popover(&app_mixer);
    });

    let term_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .build();
    term_btn.set_child(Some(&build_quick_toggle_content("", "Shell")));
    crate::core::attach_voiceover_hover(&term_btn, "Apri un terminale");

    let pop_term = pop.clone();
    term_btn.connect_clicked(move |_| {
        pop_term.close();
        let _ = Command::new("foot").spawn();
    });

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

    // LIVE STATE POLLING
    let bright_slider_clone = bright_slider.clone();
    let mpris_t = mpris_title.clone();
    let mpris_a = mpris_artist.clone();
    let mpris_p = play_btn.clone();
    let wifi_btn_clone = wifi_btn.clone();
    let bt_btn_clone = bt_btn.clone();
    

    glib::timeout_add_local(std::time::Duration::from_millis(1000), clone!(@weak pop => @default-return glib::ControlFlow::Break, move || {
        let live = crate::core::live_state::get_live_state();
        
        // Update sliders only if the difference is > 1.5 (to avoid fighting user input)
        if (bright_slider_clone.value() - live.brightness).abs() > 1.5 {
            bright_slider_clone.set_value(live.brightness);
        }

        if let Some(mpris) = crate::core::mpris::get_mpris_state() {
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

        let (_, _, net_sub) = get_network_status();
        let net_connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
        if net_connected {
            wifi_btn_clone.add_css_class("cc-btn-active");
        } else {
            wifi_btn_clone.remove_css_class("cc-btn-active");
        }

        let bt_btn_clone_timer = bt_btn_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            if let Ok(enabled) = ctrl.is_bluetooth_enabled().await {
                if enabled {
                    bt_btn_clone_timer.add_css_class("cc-btn-active");
                } else {
                    bt_btn_clone_timer.remove_css_class("cc-btn-active");
                }
            }
        });

        glib::ControlFlow::Continue
    }));

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
