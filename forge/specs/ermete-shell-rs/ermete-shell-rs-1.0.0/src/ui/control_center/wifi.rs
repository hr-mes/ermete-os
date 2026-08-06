use crate::ui::popup_manager::setup_popup_autoclose;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label,
    Orientation, PasswordEntry, Switch,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_wifi_password_modal(app: &Application, ssid: &str) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Autenticazione Wi-Fi")
        .css_classes(["popup-window"])
        .default_width(380)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi-password");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 60);
    pop.set_margin(Edge::Right, 80);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    // Header
    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let texts_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).hexpand(true).build();
    let title_lbl = Label::builder().label("Accedi alla rete Wi-Fi").css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder().label(format!("Rete: {}", ssid)).css_classes(["cc-label-sub"]).halign(Align::Start).build();
    texts_box.append(&title_lbl);
    texts_box.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&texts_box);

    // Password field
    let pwd_entry = PasswordEntry::builder()
        .placeholder_text("Inserisci la password Wi-Fi...")
        .show_peek_icon(true)
        .css_classes(["wifi-pwd-entry"])
        .hexpand(true)
        .build();

    // Security note
    let sec_note = Label::builder()
        .label("🔒  NetworkManager memorizzerà questa password per la riconnessione automatica.")
        .css_classes(["cc-label-sub"])
        .wrap(true)
        .halign(Align::Start)
        .build();

    // Status label
    let status_lbl = Label::builder()
        .label("")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    // Action buttons
    let btn_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(Align::End)
        .build();

    let cancel_btn = Button::builder()
        .label("Annulla")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_cancel = pop.clone();
    cancel_btn.connect_clicked(move |_| {
        pop_cancel.close();
    });

    let connect_btn = Button::builder()
        .label("Connetti")
        .css_classes(["cc-quick-btn"])
        .build();

    let ssid_str = ssid.to_string();
    let pwd_clone = pwd_entry.clone();
    let pop_conn = pop.clone();
    let status_clone = status_lbl.clone();
    let do_connect = move || {
        let pwd = pwd_clone.text().to_string();
        if pwd.is_empty() {
            status_clone.set_label("⚠️ Inserisci prima la password.");
            return;
        }
        status_clone.set_label("⏳ Connessione in corso...");
        let ssid_c = ssid_str.clone();
        let pwd_c = pwd.clone();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            let _ = ctrl.connect_wifi(&ssid_c, &pwd_c).await;
        });
        pop_conn.close();
    };

    let do_conn_1 = do_connect.clone();
    connect_btn.connect_clicked(move |_| {
        do_conn_1();
    });

    let do_conn_2 = do_connect.clone();
    pwd_entry.connect_activate(move |_| {
        do_conn_2();
    });

    btn_box.append(&cancel_btn);
    btn_box.append(&connect_btn);

    card.append(&header_card);
    card.append(&pwd_entry);
    card.append(&sec_note);
    card.append(&status_lbl);
    card.append(&btn_box);

    pop.set_child(Some(&card));
    pop.present();
    pwd_entry.grab_focus();
}

pub fn show_wifi_details_modal(app: &Application, ssid: &str, active: bool) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title(format!("Configurazione Rete: {}", ssid))
        .css_classes(["popup-window"])
        .default_width(420)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi-details");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 50);
    pop.set_margin(Edge::Right, 60);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let texts_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).hexpand(true).build();
    let title_lbl = Label::builder().label(ssid).css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder()
        .label(if active { "Connesso — Rete Salvata" } else { "Profilo Memorizzato" })
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();
    texts_box.append(&title_lbl);
    texts_box.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&texts_box);

    let cur_method = "auto".to_string();
    let cur_ip = "".to_string();
    let cur_gw = "".to_string();
    let cur_dns = "".to_string();
    let cur_auto = true;

    let ip_section = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let ip_header = Label::builder().label("CONFIGURAZIONE IP (IPv4)").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    let dhcp_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(10).build();
    let dhcp_lbl = Label::builder().label("IP Automatico (DHCP)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let dhcp_sw = Switch::builder().active(cur_method == "auto").valign(Align::Center).build();
    dhcp_row.append(&dhcp_lbl);
    dhcp_row.append(&dhcp_sw);

    let ip_entry = Entry::builder()
        .placeholder_text("Indirizzo IP/Subnet (es. 192.168.1.50/24)")
        .text(&cur_ip)
        .sensitive(cur_method != "auto")
        .build();
    let gw_entry = Entry::builder()
        .placeholder_text("Gateway Router (es. 192.168.1.1)")
        .text(&cur_gw)
        .sensitive(cur_method != "auto")
        .build();

    let ip_e_clone = ip_entry.clone();
    let gw_e_clone = gw_entry.clone();
    dhcp_sw.connect_state_set(move |_, is_dhcp| {
        ip_e_clone.set_sensitive(!is_dhcp);
        gw_e_clone.set_sensitive(!is_dhcp);
        glib::Propagation::Proceed
    });

    ip_section.append(&ip_header);
    ip_section.append(&dhcp_row);
    ip_section.append(&ip_entry);
    ip_section.append(&gw_entry);

    let dns_section = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let dns_header = Label::builder().label("SERVER DNS").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    let dns_entry = Entry::builder()
        .placeholder_text("DNS Personalizzati (es. 1.1.1.1, 8.8.8.8)")
        .text(&cur_dns)
        .build();
    dns_section.append(&dns_header);
    dns_section.append(&dns_entry);

    let auto_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(10).build();
    let auto_lbl = Label::builder().label("Riconnetti automaticamente").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let auto_sw = Switch::builder().active(cur_auto).valign(Align::Center).build();
    auto_row.append(&auto_lbl);
    auto_row.append(&auto_sw);

    let ip_e_clone2 = ip_entry.clone();
    let gw_e_clone2 = gw_entry.clone();
    let dns_e_clone2 = dns_entry.clone();
    let dhcp_sw_clone2 = dhcp_sw.clone();
    let auto_sw_clone2 = auto_sw.clone();
    let ssid_clone = ssid.to_string();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::get_network_controller();
        if let Ok((method, ip, gw, dns, auto)) = ctrl.get_wifi_details(&ssid_clone).await {
            dhcp_sw_clone2.set_active(method == "auto");
            ip_e_clone2.set_text(&ip);
            gw_e_clone2.set_text(&gw);
            dns_e_clone2.set_text(&dns);
            auto_sw_clone2.set_active(auto);
        }
    });

    let btn_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();

    let forget_btn = Button::builder().label("Dimentica").css_classes(["cc-quick-btn"]).build();
    let ssid_f = ssid.to_string();
    let pop_f = pop.clone();
    forget_btn.connect_clicked(move |_| {
        let ssid_f = ssid_f.clone();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            let _ = ctrl.delete_wifi(&ssid_f).await;
        });
        pop_f.close();
    });

    let disc_btn = Button::builder().label("Disconnetti").css_classes(["cc-quick-btn"]).build();
    let ssid_d = ssid.to_string();
    let pop_d = pop.clone();
    disc_btn.connect_clicked(move |_| {
        let ssid_d = ssid_d.clone();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            let _ = ctrl.disconnect_wifi(&ssid_d).await;
        });
        pop_d.close();
    });

    let save_btn = Button::builder().label("Salva e Applica").css_classes(["cc-quick-btn"]).hexpand(true).build();
    let ssid_s = ssid.to_string();
    let dhcp_sw_clone = dhcp_sw.clone();
    let ip_e_s = ip_entry.clone();
    let gw_e_s = gw_entry.clone();
    let dns_e_s = dns_entry.clone();
    let auto_sw_s = auto_sw.clone();
    let pop_s = pop.clone();
    save_btn.connect_clicked(move |_| {
        let ssid_s = ssid_s.clone();
        let dhcp_val = dhcp_sw_clone.is_active();
        let ip_val = ip_e_s.text().to_string();
        let gw_val = gw_e_s.text().to_string();
        let dns_val = dns_e_s.text().to_string();
        let auto_val = auto_sw_s.is_active();
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            let _ = ctrl.modify_wifi(&ssid_s, dhcp_val, &ip_val, &gw_val, &dns_val, auto_val).await;
        });
        pop_s.close();
    });

    btn_box.append(&forget_btn);
    if active {
        btn_box.append(&disc_btn);
    }
    btn_box.append(&save_btn);

    card.append(&header_card);
    card.append(&ip_section);
    card.append(&dns_section);
    card.append(&auto_row);
    card.append(&btn_box);

    pop.set_child(Some(&card));
    pop.present();
}

pub(crate) fn populate_wifi_list(list_box: &GtkBox, app: &Application, pop: &ApplicationWindow, wifi_enabled: bool) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    if !wifi_enabled {
        let disabled_card = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .css_classes(["pro-applet-card"])
            .build();
        let lbl1 = Label::builder().label("󰖪  Rete Wi-Fi disattivata").css_classes(["cc-label-main"]).halign(Align::Start).build();
        let lbl2 = Label::builder().label("Attiva l'interruttore in alto per cercare e visualizzare le reti Wi-Fi vicine.").css_classes(["cc-label-sub"]).wrap(true).halign(Align::Start).build();
        disabled_card.append(&lbl1);
        disabled_card.append(&lbl2);
        list_box.append(&disabled_card);
        return;
    }

    let list_box_clone = list_box.clone();
    let app_clone = app.clone();
    let pop_clone = pop.clone();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::get_network_controller();
        if let Ok(networks) = ctrl.list_wifi_networks().await {
            while let Some(child) = list_box_clone.first_child() {
                list_box_clone.remove(&child);
            }
            let mut count = 0;
            for net in networks {
                if count >= 8 {
                    break;
                }
                let icon = if net.signal > 75 {
                    "󰤨"
                } else if net.signal > 40 {
                    "󰤥"
                } else {
                    "󰤢"
                };

                let item_row = Button::builder()
                    .css_classes(["pro-applet-card-btn"])
                    .build();

                let inner_box = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(10)
                    .build();

                let icon_lbl = Label::builder().label(icon).build();
                let texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
                let ssid_lbl = Label::builder()
                    .label(&net.ssid)
                    .css_classes(["cc-label-main"])
                    .halign(Align::Start)
                    .build();
                let status_text = if net.active {
                    "Connesso — Attiva"
                } else if net.saved {
                    "Salvato — Clicca per impostazioni"
                } else {
                    "Disponibile — Clicca per connetterti"
                };
                let status_lbl = Label::builder()
                    .label(status_text)
                    .css_classes(["cc-label-sub"])
                    .halign(Align::Start)
                    .build();
                texts.append(&ssid_lbl);
                texts.append(&status_lbl);

                inner_box.append(&icon_lbl);
                inner_box.append(&texts);

                if net.active {
                    let check_lbl = Label::builder().label("✓").css_classes(["cc-label-main"]).build();
                    inner_box.append(&check_lbl);
                }

                item_row.set_child(Some(&inner_box));

                let app_c = app_clone.clone();
                let pop_c = pop_clone.clone();
                let ssid_str = net.ssid.clone();
                let active_f = net.active;
                let saved_f = net.saved;
                item_row.connect_clicked(move |_| {
                    pop_c.close();
                    if active_f || saved_f {
                        show_wifi_details_modal(&app_c, &ssid_str, active_f);
                    } else {
                        show_wifi_password_modal(&app_c, &ssid_str);
                    }
                });

                list_box_clone.append(&item_row);
                count += 1;
            }
            if count == 0 {
                let no_wifi = Label::builder()
                    .label("Nessuna rete Wi-Fi rilevata")
                    .css_classes(["cc-label-sub"])
                    .build();
                list_box_clone.append(&no_wifi);
            }
        } else {
            let err_lbl = Label::builder()
                .label("Impossibile interrogare NetworkManager")
                .css_classes(["cc-label-sub"])
                .build();
            list_box_clone.append(&err_lbl);
        }
    });
}

pub fn show_wifi_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Reti Wi-Fi")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi");
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
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let header_lbl = Label::builder().label("Rete Wi-Fi").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let wifi_sw = Switch::builder().active(true).valign(Align::Center).build();
    let wifi_sw_clone = wifi_sw.clone();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::get_network_controller();
        if let Ok(enabled) = ctrl.is_wifi_enabled().await {
            wifi_sw_clone.set_active(enabled);
        }
    });
    header_card.append(&header_icon);
    header_card.append(&header_lbl);
    header_card.append(&wifi_sw);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    populate_wifi_list(&list_box, app, &pop, true);

    let list_clone = list_box.clone();
    let app_clone = app.clone();
    let pop_clone = pop.clone();
    wifi_sw.connect_state_set(move |_, state| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            let _ = ctrl.set_wifi_powered(state).await;
        });
        populate_wifi_list(&list_clone, &app_clone, &pop_clone, state);
        glib::Propagation::Proceed
    });

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone2 = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone2.close();
    });

    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let settings_wifi_btn = Button::builder()
        .label("⚙ Impostazioni Wi-Fi")
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let pop_wifi_s = pop.clone();
    settings_wifi_btn.connect_clicked(move |_| {
        pop_wifi_s.close();
        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", "wifi"));
    });
    footer_box.append(&settings_wifi_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&list_box);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}
