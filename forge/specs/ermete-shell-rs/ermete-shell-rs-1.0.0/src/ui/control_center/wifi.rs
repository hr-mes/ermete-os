use crate::ui::popup_manager::setup_popup_autoclose;
use crate::ui::viewmodel::{WifiViewModel, WifiIntent, NavigationViewModel, UiPopoverTarget};
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
        .css_classes(["liquid-surface"])
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
        WifiViewModel::execute_intent(WifiIntent::ConnectWifiWithPassword {
            ssid: ssid_str.clone(),
            password: pwd,
        });
        pop_conn.close();
    };

    let do_conn_1 = do_connect.clone();
    connect_btn.connect_clicked(move |_| {
        do_conn_1();
    });

    let do_conn_2 = do_connect;
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
}

pub fn show_wifi_details_modal(app: &Application, ssid: &str, active: bool) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Dettagli Rete Wi-Fi")
        .css_classes(["popup-window"])
        .default_width(380)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi-details");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 60);
    pop.set_margin(Edge::Right, 80);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["liquid-surface"])
        .build();

    // Header
    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("⚙").css_classes(["cc-circle-blue"]).build();
    let texts_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).hexpand(true).build();
    let title_lbl = Label::builder().label(ssid).css_classes(["cc-label-main"]).halign(Align::Start).build();
    let status_str = if active { "Connesso — Attiva" } else { "Salvata" };
    let sub_lbl = Label::builder().label(status_str).css_classes(["cc-label-sub"]).halign(Align::Start).build();
    texts_box.append(&title_lbl);
    texts_box.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&texts_box);

    let cur_ip = String::new();
    let cur_gw = String::new();
    let cur_dns = String::new();
    let cur_dhcp = true;
    let cur_auto = true;

    let ip_section = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let ip_header = Label::builder().label("CONFIGURAZIONE IP").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    
    let dhcp_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(10).build();
    let dhcp_lbl = Label::builder().label("DHCP Automatico").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let dhcp_sw = Switch::builder().active(cur_dhcp).valign(Align::Center).build();
    dhcp_row.append(&dhcp_lbl);
    dhcp_row.append(&dhcp_sw);

    let ip_entry = Entry::builder()
        .placeholder_text("Indirizzo IP (es. 192.168.1.50)")
        .text(&cur_ip)
        .sensitive(!cur_dhcp)
        .build();

    let gw_entry = Entry::builder()
        .placeholder_text("Gateway (es. 192.168.1.1)")
        .text(&cur_gw)
        .sensitive(!cur_dhcp)
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
    
    WifiViewModel::fetch_details(ssid, move |method, ip, gw, dns, auto| {
        dhcp_sw_clone2.set_active(method == "auto");
        ip_e_clone2.set_text(&ip);
        gw_e_clone2.set_text(&gw);
        dns_e_clone2.set_text(&dns);
        auto_sw_clone2.set_active(auto);
    });

    let btn_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();

    let forget_btn = Button::builder().label("Dimentica").css_classes(["cc-quick-btn"]).build();
    let ssid_f = ssid.to_string();
    let pop_f = pop.clone();
    forget_btn.connect_clicked(move |_| {
        WifiViewModel::execute_intent(WifiIntent::ForgetNetwork { ssid: ssid_f.clone() });
        pop_f.close();
    });

    let disc_btn = Button::builder().label("Disconnetti").css_classes(["cc-quick-btn"]).build();
    let ssid_d = ssid.to_string();
    let pop_d = pop.clone();
    disc_btn.connect_clicked(move |_| {
        WifiViewModel::execute_intent(WifiIntent::DisconnectNetwork { ssid: ssid_d.clone() });
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
        let dhcp_val = dhcp_sw_clone.is_active();
        let ip_val = ip_e_s.text().to_string();
        let gw_val = gw_e_s.text().to_string();
        let dns_val = dns_e_s.text().to_string();
        let auto_val = auto_sw_s.is_active();
        WifiViewModel::execute_intent(WifiIntent::ModifyWifi {
            ssid: ssid_s.clone(),
            dhcp: dhcp_val,
            ip: ip_val,
            gw: gw_val,
            dns: dns_val,
            auto: auto_val,
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

    WifiViewModel::fetch_networks(move |result| {
        while let Some(child) = list_box_clone.first_child() {
            list_box_clone.remove(&child);
        }
        match result {
            Ok(networks) => {
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
                            NavigationViewModel::navigate_to(&app_c, UiPopoverTarget::WifiDetails(ssid_str.clone(), active_f));
                        } else {
                            NavigationViewModel::navigate_to(&app_c, UiPopoverTarget::WifiPassword(ssid_str.clone()));
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
            }
            Err(_) => {
                let err_lbl = Label::builder()
                    .label("Impossibile interrogare NetworkManager")
                    .css_classes(["cc-label-sub"])
                    .build();
                list_box_clone.append(&err_lbl);
            }
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
        .css_classes(["liquid-surface"])
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

    WifiViewModel::fetch_initial_state(move |enabled| {
        wifi_sw_clone.set_active(enabled);
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
        WifiViewModel::execute_intent(WifiIntent::SetWifiPowered(state));
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
        WifiViewModel::execute_intent(WifiIntent::LaunchWifiSettings);
    });
    footer_box.append(&settings_wifi_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&list_box);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}
