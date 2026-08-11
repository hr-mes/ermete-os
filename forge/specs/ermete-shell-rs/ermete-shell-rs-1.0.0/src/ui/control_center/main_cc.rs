use crate::ui::popup_manager::setup_popup_autoclose;
use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent, NavigationViewModel, UiPopoverTarget};
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Scale};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn build_quick_toggle_pill(badge_class: &str, icon_glyph: &str, title: &str, sub: &str) -> (Button, Label, Label, Label) {
    let btn = Button::builder().css_classes(["cc-toggle-pill"]).hexpand(true).build();
    let box_ = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .valign(Align::Center)
        .build();

    let badge = Label::builder()
        .label(icon_glyph)
        .css_classes([badge_class])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let text_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(1)
        .valign(Align::Center)
        .build();

    let lbl_title = Label::builder()
        .label(title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();

    let lbl_sub = Label::builder()
        .label(sub)
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    text_box.append(&lbl_title);
    text_box.append(&lbl_sub);

    box_.append(&badge);
    box_.append(&text_box);

    btn.set_child(Some(&box_));
    (btn, badge, lbl_title, lbl_sub)
}

fn build_quick_action_btn(icon: &str, text: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-quick-btn"]).build();
    let box_ = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    let icon_lbl = Label::builder().label(icon).css_classes(["cc-slider-icon"]).build();
    let text_lbl = Label::builder().label(text).css_classes(["cc-label-main"]).build();
    box_.append(&icon_lbl);
    box_.append(&text_lbl);
    btn.set_child(Some(&box_));
    btn
}

pub fn show_control_center_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Control Center")
        .css_classes(["popup-window"])
        .default_width(380)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "control-center");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let initial = ControlCenterViewModel::get_initial_state();

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .css_classes(["liquid-surface"])
        .build();

    // 0. HEADER BAR (Title + System Settings button)
    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .valign(Align::Center)
        .build();
    let cc_title_lbl = Label::builder()
        .label("Control Center")
        .css_classes(["cc-label-title"])
        .hexpand(true)
        .halign(Align::Start)
        .build();
    let settings_btn = Button::builder()
        .label("⚙ Impostazioni")
        .css_classes(["cc-quick-btn"])
        .tooltip_text("Apri Impostazioni di Sistema")
        .build();
    settings_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchSettings(String::new()));
    }));
    header_box.append(&cc_title_lbl);
    header_box.append(&settings_btn);

    // 1. QUICK TOGGLES GRID (iOS/AGS Inspired 2x2 Grid)
    let quick_toggles_grid = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let row1 = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).homogeneous(true).build();
    let row2 = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).homogeneous(true).build();

    // Wi-Fi Quick Toggle
    let (wifi_toggle_btn, wifi_badge, _wifi_title_lbl, wifi_sub_lbl) =
        build_quick_toggle_pill("cc-circle-blue", &initial.network_icon, "Wi-Fi", &initial.network_sub);
    if initial.is_network_connected {
        wifi_toggle_btn.add_css_class("cc-btn-active");
    }
    wifi_toggle_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        NavigationViewModel::navigate_to(&app, UiPopoverTarget::Wifi);
    }));
    crate::core::attach_voiceover_hover(&wifi_toggle_btn, "Apri impostazioni o stato Wi-Fi");

    // Bluetooth Quick Toggle
    let (bt_toggle_btn, _bt_badge, _bt_title_lbl, bt_sub_lbl) =
        build_quick_toggle_pill("cc-circle-blue", "", "Bluetooth", &initial.bluetooth_sub);
    if initial.is_bluetooth_active {
        bt_toggle_btn.add_css_class("cc-btn-active");
    }
    bt_toggle_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        NavigationViewModel::navigate_to(&app, UiPopoverTarget::Bluetooth);
    }));
    crate::core::attach_voiceover_hover(&bt_toggle_btn, "Apri impostazioni o stato Bluetooth");

    row1.append(&wifi_toggle_btn);
    row1.append(&bt_toggle_btn);

    // True Tone Quick Toggle
    let tt_sub_str = if initial.true_tone { "Attivo" } else { "Disattivato" };
    let (tt_toggle_btn, _tt_badge, _tt_title_lbl, tt_sub_lbl) =
        build_quick_toggle_pill("cc-circle-amber", "󰛨", "True Tone", tt_sub_str);
    if initial.true_tone {
        tt_toggle_btn.add_css_class("cc-truetone-active");
    }
    let tt_btn_click = tt_toggle_btn.clone();
    let tt_sub_lbl_clone = tt_sub_lbl.clone();
    tt_toggle_btn.connect_clicked(move |_| {
        let is_active = tt_btn_click.has_css_class("cc-truetone-active");
        let new_state = !is_active;
        if new_state {
            tt_btn_click.add_css_class("cc-truetone-active");
            tt_sub_lbl_clone.set_label("Attivo");
        } else {
            tt_btn_click.remove_css_class("cc-truetone-active");
            tt_sub_lbl_clone.set_label("Disattivato");
        }
        ControlCenterViewModel::execute_intent(ControlCenterIntent::ToggleTrueTone(new_state));
    });
    crate::core::attach_voiceover_hover(&tt_toggle_btn, "Attiva o disattiva il filtro True Tone per affaticamento visivo");

    // Focus Mode (macOS/iOS Style) Quick Toggle
    let ft_sub_str = initial.focus_mode.name();
    let ft_icon = initial.focus_mode.icon();
    let (ft_toggle_btn, ft_badge, _ft_title_lbl, ft_sub_lbl) =
        build_quick_toggle_pill("cc-circle-indigo", ft_icon, "Focus Mode", ft_sub_str);
    if initial.focus_mode.is_active() {
        ft_toggle_btn.add_css_class("cc-focus-active");
    }
    let ft_btn_click = ft_toggle_btn.clone();
    let ft_sub_lbl_clone = ft_sub_lbl.clone();
    let ft_badge_clone = ft_badge.clone();
    ft_toggle_btn.connect_clicked(move |_| {
        let current = crate::ipc::notifications::get_focus_mode();
        let next = match current {
            crate::ipc::notifications::FocusMode::Off => crate::ipc::notifications::FocusMode::Personal,
            crate::ipc::notifications::FocusMode::Personal => crate::ipc::notifications::FocusMode::Work,
            crate::ipc::notifications::FocusMode::Work => crate::ipc::notifications::FocusMode::Sleep,
            crate::ipc::notifications::FocusMode::Sleep => crate::ipc::notifications::FocusMode::Off,
        };
        if next.is_active() {
            ft_btn_click.add_css_class("cc-focus-active");
        } else {
            ft_btn_click.remove_css_class("cc-focus-active");
        }
        ft_badge_clone.set_label(next.icon());
        ft_sub_lbl_clone.set_label(next.name());
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetFocusMode(next));
    });
    crate::core::attach_voiceover_hover(&ft_toggle_btn, "Cambia modalità Focus (Personale, Lavoro, Sonno, Disattivato)");

    row2.append(&tt_toggle_btn);
    row2.append(&ft_toggle_btn);

    quick_toggles_grid.append(&row1);
    quick_toggles_grid.append(&row2);

    // 1.5. FOCUS MODE INTERACTIVE PILL WIDGET (macOS / iOS Style Focus Filters)
    let focus_card = build_focus_mode_widget(
        initial.focus_mode,
        Some(ft_badge.clone()),
        Some(ft_sub_lbl.clone()),
        Some(ft_toggle_btn.clone()),
    );

    // 2. SLIDERS SECTION (Luminosità & Master Volume)
    // Slider Luminosità
    let bright_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let bright_icon = Label::builder().label("☀").css_classes(["cc-slider-icon"]).build();
    let bright_slider = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(initial.brightness, 0.0, 100.0, 1.0, 10.0, 0.0))
        .css_classes(["cc-scale"])
        .hexpand(true)
        .valign(Align::Center)
        .build();
    bright_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetBrightness(val));
    });
    bright_card.append(&bright_icon);
    bright_card.append(&bright_slider);

    let disp_settings_btn = Button::builder()
        .label("⚙")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Impostazioni Schermi")
        .build();
    disp_settings_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchSettings("displays".to_string()));
    }));
    bright_card.append(&disp_settings_btn);

    // Slider Volume Master Audio
    let audio_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let audio_icon = Label::builder().label("🔊").css_classes(["cc-slider-icon"]).build();
    let audio_slider = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&gtk4::Adjustment::new(initial.volume, 0.0, 100.0, 1.0, 10.0, 0.0))
        .css_classes(["cc-scale"])
        .hexpand(true)
        .valign(Align::Center)
        .build();
    audio_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetVolume(val));
    });
    audio_card.append(&audio_icon);
    audio_card.append(&audio_slider);

    let audio_mixer_btn = Button::builder()
        .label("🎚️")
        .css_classes(["cc-quick-btn"])
        .valign(Align::Center)
        .tooltip_text("Mixer Audio Avanzato")
        .build();
    audio_mixer_btn.connect_clicked(glib::clone!(@weak pop, @weak app => move |_| {
        pop.close();
        NavigationViewModel::navigate_to(&app, UiPopoverTarget::AudioMixer);
    }));
    audio_card.append(&audio_mixer_btn);

    // 3. ADVANCED MPRIS MEDIA PLAYER CARD
    let mpris_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-mpris-card"])
        .valign(Align::Center)
        .build();

    let mpris_art_badge = Label::builder()
        .label("🎵")
        .css_classes(["mpris-art-badge"])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let mpris_info_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .valign(Align::Center)
        .hexpand(true)
        .build();

    let mpris_title = Label::builder()
        .label(&initial.mpris_title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    let mpris_artist = Label::builder()
        .label(&initial.mpris_artist)
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();

    mpris_info_box.append(&mpris_title);
    mpris_info_box.append(&mpris_artist);

    let mpris_ctrl_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .valign(Align::Center)
        .build();

    let prev_btn = Button::builder().label("⏮").css_classes(["mpris-ctrl-btn"]).build();
    let play_btn_label = if initial.is_playing { "⏸" } else { "▶" };
    let play_btn = Button::builder().label(play_btn_label).css_classes(["mpris-ctrl-btn", "mpris-play-btn"]).build();
    let next_btn = Button::builder().label("⏭").css_classes(["mpris-ctrl-btn"]).build();

    prev_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::MediaPrevious);
    });
    play_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::MediaPlayPause);
    });
    next_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::MediaNext);
    });

    crate::core::attach_voiceover_hover(&prev_btn, "Brano precedente");
    crate::core::attach_voiceover_hover(&play_btn, "Riproduci o metti in pausa");
    crate::core::attach_voiceover_hover(&next_btn, "Brano successivo");

    mpris_ctrl_box.append(&prev_btn);
    mpris_ctrl_box.append(&play_btn);
    mpris_ctrl_box.append(&next_btn);

    mpris_card.append(&mpris_art_badge);
    mpris_card.append(&mpris_info_box);
    mpris_card.append(&mpris_ctrl_box);

    // 4. BOTTOM QUICK ACTIONS GRID
    let bottom_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let dark_btn = build_quick_action_btn("☾", "Scuro");
    crate::core::attach_voiceover_hover(&dark_btn, "Modalità scura");
    dark_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::SetDarkMode);
    });

    let standby_btn = build_quick_action_btn("🖥", "Standby");
    crate::core::attach_voiceover_hover(&standby_btn, "Sospendi il sistema");
    standby_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::TriggerStandby);
    }));

    let term_btn = build_quick_action_btn("", "Shell");
    crate::core::attach_voiceover_hover(&term_btn, "Apri un terminale shell");
    term_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchTerminal);
    }));

    let screenshot_btn = build_quick_action_btn("📷", "Cattura");
    crate::core::attach_voiceover_hover(&screenshot_btn, "Scatta uno screenshot");
    screenshot_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::TriggerScreenshot);
    }));

    let lock_btn = build_quick_action_btn("🔒", "Blocca");
    crate::core::attach_voiceover_hover(&lock_btn, "Blocca lo schermo");
    lock_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::TriggerLock);
    }));

    bottom_grid.append(&dark_btn);
    bottom_grid.append(&standby_btn);
    bottom_grid.append(&term_btn);
    bottom_grid.append(&screenshot_btn);
    bottom_grid.append(&lock_btn);

    // 5. HARDWARE-TAILORED KERNEL FORGE ACTION BUTTON
    let kernel_forge_btn = Button::builder()
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let kf_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    let kf_icon = Label::builder().label("🧬").css_classes(["cc-slider-icon"]).build();
    let kf_label = Label::builder().label("Optimize Kernel for this Hardware").css_classes(["cc-label-main"]).build();
    kf_box.append(&kf_icon);
    kf_box.append(&kf_label);
    kernel_forge_btn.set_child(Some(&kf_box));
    kernel_forge_btn.set_tooltip_text(Some("Gentoo-style hardware tailored kernel compilation with LTO & AutoFDO"));
    kernel_forge_btn.connect_clicked(glib::clone!(@weak pop => move |_| {
        pop.close();
        ControlCenterViewModel::execute_intent(ControlCenterIntent::OptimizeKernel);
    }));
    crate::core::attach_voiceover_hover(&kernel_forge_btn, "Optimize Kernel for this Hardware");

    // ASSEMBLY INTO CARD CONTAINER
    card.append(&header_box);
    card.append(&quick_toggles_grid);
    card.append(&focus_card);
    card.append(&bright_card);
    card.append(&audio_card);
    card.append(&mpris_card);
    card.append(&bottom_grid);
    card.append(&kernel_forge_btn);

    // REACTIVE BINDINGS TO VIEWMODEL STATE
    let bright_slider_clone = bright_slider.clone();
    let audio_slider_clone = audio_slider.clone();
    let mpris_t = mpris_title.clone();
    let mpris_a = mpris_artist.clone();
    let mpris_p = play_btn.clone();
    let wifi_btn_clone = wifi_toggle_btn.clone();
    let wifi_badge_clone = wifi_badge.clone();
    let wifi_sub_clone = wifi_sub_lbl.clone();
    let bt_btn_clone = bt_toggle_btn.clone();
    let bt_sub_clone = bt_sub_lbl.clone();

    ControlCenterViewModel::subscribe_network(move |icon, _title, sub, connected, bt_enabled| {
        if connected {
            wifi_btn_clone.add_css_class("cc-btn-active");
        } else {
            wifi_btn_clone.remove_css_class("cc-btn-active");
        }
        wifi_badge_clone.set_label(&icon);
        wifi_sub_clone.set_label(&sub);

        if let Some(enabled) = bt_enabled {
            if enabled {
                bt_btn_clone.add_css_class("cc-btn-active");
                bt_sub_clone.set_label("Attivo");
            } else {
                bt_btn_clone.remove_css_class("cc-btn-active");
                bt_sub_clone.set_label("Disattivato");
            }
        }
    });

    ControlCenterViewModel::subscribe_hardware(move |val| {
        if (bright_slider_clone.value() - val * 100.0).abs() > 1.5 {
            bright_slider_clone.set_value(val * 100.0);
        }
    });

    ControlCenterViewModel::subscribe_audio(move |val| {
        if (audio_slider_clone.value() - val * 100.0).abs() > 1.5 {
            audio_slider_clone.set_value(val * 100.0);
        }
    });

    ControlCenterViewModel::subscribe_mpris(move |title, artist, is_playing| {
        mpris_t.set_label(&title);
        mpris_a.set_label(&artist);
        mpris_p.set_label(if is_playing { "⏸" } else { "▶" });
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

fn build_focus_mode_widget(
    initial_mode: crate::ipc::notifications::FocusMode,
    quick_badge: Option<Label>,
    quick_sub: Option<Label>,
    quick_btn: Option<Button>,
) -> GtkBox {
    use crate::ipc::notifications::FocusMode;

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["liquid-surface"])
        .build();

    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
        .build();

    let icon_lbl = Label::builder()
        .label(initial_mode.icon())
        .css_classes(["cc-circle-indigo"])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let text_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(1)
        .hexpand(true)
        .build();

    let title_lbl = Label::builder()
        .label("Filtri Focus")
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();

    let sub_lbl = Label::builder()
        .label(initial_mode.description())
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    text_box.append(&title_lbl);
    text_box.append(&sub_lbl);
    header_box.append(&icon_lbl);
    header_box.append(&text_box);

    card.append(&header_box);

    // Interactive Pill Buttons (macOS / iOS style segmented pill bar)
    let pills_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let modes = [
        FocusMode::Off,
        FocusMode::Personal,
        FocusMode::Work,
        FocusMode::Sleep,
    ];

    let mut pill_btns: Vec<(FocusMode, Button)> = Vec::new();

    for mode in modes {
        let label_text = format!("{} {}", mode.icon(), mode.name());
        let btn = Button::builder()
            .label(&label_text)
            .css_classes(["cc-btn", "cc-quick-btn"])
            .build();

        if mode == initial_mode {
            btn.add_css_class("cc-btn-active");
        }

        pill_btns.push((mode, btn.clone()));
        pills_box.append(&btn);
    }

    let pill_btns_rc = std::rc::Rc::new(pill_btns);
    for (mode, btn) in pill_btns_rc.iter() {
        let mode_val = *mode;
        let icon_lbl_clone = icon_lbl.clone();
        let sub_lbl_clone = sub_lbl.clone();
        let btns_clone = pill_btns_rc.clone();
        let q_badge = quick_badge.clone();
        let q_sub = quick_sub.clone();
        let q_btn = quick_btn.clone();

        btn.connect_clicked(move |_| {
            for (m, b) in btns_clone.iter() {
                if *m == mode_val {
                    b.add_css_class("cc-btn-active");
                } else {
                    b.remove_css_class("cc-btn-active");
                }
            }
            icon_lbl_clone.set_label(mode_val.icon());
            sub_lbl_clone.set_label(mode_val.description());

            if let Some(ref b) = q_badge {
                b.set_label(mode_val.icon());
            }
            if let Some(ref s) = q_sub {
                s.set_label(mode_val.name());
            }
            if let Some(ref btn_q) = q_btn {
                if mode_val.is_active() {
                    btn_q.add_css_class("cc-focus-active");
                } else {
                    btn_q.remove_css_class("cc-focus-active");
                }
            }

            ControlCenterViewModel::execute_intent(ControlCenterIntent::SetFocusMode(mode_val));
        });
        crate::core::attach_voiceover_hover(btn, &format!("Seleziona filtro Focus {}", mode.name()));
    }

    card.append(&pills_box);
    card
}

