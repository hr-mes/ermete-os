use crate::core::*;
use crate::ui::spotlight::*;
use crate::ui::topbar::setup_popup_autoclose;
use glib::clone;
use zbus::proxy;
use futures_util::StreamExt;



use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Calendar, Entry,
    Label, Orientation, PasswordEntry, ProgressBar, Scale, ScrolledWindow, Switch,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::process::Command;

pub(crate) fn build_cc_row(badge_class: &str, icon_glyph: &str, title: &str, sub: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-tile-row"]).build();
    let row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(14)
        .valign(Align::Center)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(8)
        .margin_end(8)
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

    row_box.append(&badge);
    row_box.append(&text_box);
    btn.set_child(Some(&row_box));
    btn
}

pub(crate) fn build_cc_compact_tile(badge_class: &str, icon_glyph: &str, title: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-tile"]).hexpand(true).build();
    let row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .valign(Align::Center)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    let badge = Label::builder()
        .label(icon_glyph)
        .css_classes([badge_class])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let lbl = Label::builder()
        .label(title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();

    row_box.append(&badge);
    row_box.append(&lbl);
    btn.set_child(Some(&row_box));
    btn
}

fn build_quick_toggle_content(icon: &str, text: &str) -> GtkBox {
    let box_ = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).halign(Align::Center).build();
    box_.append(&Label::builder().label(icon).build());
    box_.append(&Label::builder().label(text).build());
    box_
}

pub fn show_system_monitor_modal(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Monitor Risorse")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "sys-monitor");
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

    let header = Label::builder()
        .label("MONITOR DI SISTEMA — ERMETE OS")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    // CPU Metric Card
    let (cpu_text, cpu_frac) = get_cpu_load();
    let cpu_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["metric-card"])
        .build();
    let cpu_top = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let cpu_val_lbl = Label::builder()
        .label(&format!("{:.0}%", cpu_frac * 100.0))
        .css_classes(["metric-value"])
        .halign(Align::Start)
        .build();
    let cpu_desc = Label::builder()
        .label(&format!("Processore\n{}", cpu_text))
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .build();
    cpu_top.append(&cpu_val_lbl);
    cpu_top.append(&cpu_desc);
    let cpu_bar = ProgressBar::builder()
        .fraction(cpu_frac)
        .css_classes(["cc-progress-blue"])
        .build();
    cpu_card.append(&cpu_top);
    cpu_card.append(&cpu_bar);

    // RAM Metric Card
    let (ram_text, ram_frac) = get_ram_info();
    let ram_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["metric-card"])
        .build();
    let ram_top = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let ram_val_lbl = Label::builder()
        .label(&format!("{:.0}%", ram_frac * 100.0))
        .css_classes(["metric-value"])
        .halign(Align::Start)
        .build();
    let ram_desc = Label::builder()
        .label(&format!("Memoria RAM\n{}", ram_text))
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .build();
    ram_top.append(&ram_val_lbl);
    ram_top.append(&ram_desc);
    let ram_bar = ProgressBar::builder()
        .fraction(ram_frac)
        .css_classes(["cc-progress-indigo"])
        .build();
    ram_card.append(&ram_top);
    ram_card.append(&ram_bar);

    let sys_info = Label::builder()
        .label("Wayland / Niri Compositor — Forgia Atomica RPM")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    let close_btn = Button::builder()
        .label("Chiudi")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header);
    card.append(&cpu_card);
    card.append(&ram_card);
    card.append(&sys_info);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}

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
            let ctrl = crate::core::system_proxies::get_global_controller();
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
        let ctrl = crate::core::system_proxies::get_global_controller();
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
            let ctrl = crate::core::system_proxies::get_global_controller();
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
            let ctrl = crate::core::system_proxies::get_global_controller();
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
            let ctrl = crate::core::system_proxies::get_global_controller();
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
        let ctrl = crate::core::system_proxies::get_global_controller();
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
        let ctrl = crate::core::system_proxies::get_global_controller();
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
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.toggle_wifi().await;
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
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "wifi"])
            .spawn();
    });
    footer_box.append(&settings_wifi_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&list_box);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}

pub fn show_bluetooth_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Bluetooth")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "bluetooth");
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
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let header_lbl = Label::builder().label("Bluetooth").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let bt_sw = Switch::builder().active(true).valign(Align::Center).build();
    let bt_sw_clone = bt_sw.clone();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::system_proxies::get_global_controller();
        if let Ok(enabled) = ctrl.is_bluetooth_enabled().await {
            bt_sw_clone.set_active(enabled);
        }
    });
    bt_sw.connect_state_set(move |_, state| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.toggle_bluetooth().await;
            let _ = ctrl.set_bluetooth_powered(state).await;
        });
        glib::Propagation::Proceed
    });
    header_card.append(&header_icon);
    header_card.append(&header_lbl);
    header_card.append(&bt_sw);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let list_box_clone = list_box.clone();
    glib::MainContext::default().spawn_local(async move {
        let ctrl = crate::core::system_proxies::get_global_controller();
        if let Ok(devices) = ctrl.list_bluetooth_devices().await {
            for dev in devices.iter().take(8) {
                let item_row = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(10)
                    .css_classes(["pro-applet-card"])
                    .build();

                let icon_lbl = Label::builder().label("").build();
                let texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
                let name_lbl = Label::builder()
                    .label(&dev.name)
                    .css_classes(["cc-label-main"])
                    .halign(Align::Start)
                    .build();
                let sub_lbl = Label::builder()
                    .label(if dev.connected { "Connesso" } else { "Dispositivo Rilevato" })
                    .css_classes(["cc-label-sub"])
                    .halign(Align::Start)
                    .build();
                texts.append(&name_lbl);
                texts.append(&sub_lbl);

                item_row.append(&icon_lbl);
                item_row.append(&texts);
                list_box_clone.append(&item_row);
            }
            if devices.is_empty() {
                let no_bt = Label::builder()
                    .label("Nessun dispositivo accoppiato")
                    .css_classes(["cc-label-sub"])
                    .build();
                list_box_clone.append(&no_bt);
            }
        }
    });

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let settings_bt_btn = Button::builder()
        .label("⚙ Impostazioni Bluetooth")
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let pop_bt_s = pop.clone();
    settings_bt_btn.connect_clicked(move |_| {
        pop_bt_s.close();
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "bluetooth"])
            .spawn();
    });
    footer_box.append(&settings_bt_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&list_box);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}

pub fn show_audio_mixer_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Mixer Audio")
        .css_classes(["popup-window"])
        .default_width(360)
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
    let header_icon = Label::builder().label("🎚️").css_classes(["cc-slider-icon"]).build();
    let header_texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
    let title_lbl = Label::builder().label("MIXER AUDIO ERMETE OS").css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder().label("PipeWire / WirePlumber").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    header_texts.append(&title_lbl);
    header_texts.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&header_texts);

    // Sezione Uscita Audio
    let out_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["pro-applet-card"])
        .build();
    let out_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let out_lbl = Label::builder().label("🔊  Uscita Audio (Speaker/Cuffie)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_out_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    mute_out_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.toggle_mute().await;
        });
    });
    out_header.append(&out_lbl);
    out_header.append(&mute_out_btn);

    let out_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    out_slider.set_value(80.0);
    out_slider.set_hexpand(true);
    out_slider.set_valign(Align::Center);
    out_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.set_volume(val).await;
        });
    });
    out_card.append(&out_header);
    out_card.append(&out_slider);

    // Sezione Ingresso Microfono
    let in_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["pro-applet-card"])
        .build();
    let in_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let in_lbl = Label::builder().label("🎙  Ingresso Audio (Microfono)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_in_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    mute_in_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.toggle_source_mute().await;
        });
    });
    in_header.append(&in_lbl);
    in_header.append(&mute_in_btn);

    let in_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    in_slider.set_value(75.0);
    in_slider.set_hexpand(true);
    in_slider.set_valign(Align::Center);
    in_slider.connect_value_changed(move |s| {
        let val = s.value() / 100.0;
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.set_source_volume(val).await;
        });
    });
    in_card.append(&in_header);
    in_card.append(&in_slider);

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let settings_audio_btn = Button::builder()
        .label("⚙ Impostazioni Audio")
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let pop_audio_s = pop.clone();
    settings_audio_btn.connect_clicked(move |_| {
        pop_audio_s.close();
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "audio"])
            .spawn();
    });
    footer_box.append(&settings_audio_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&out_card);
    card.append(&in_card);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
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
        let _ = Command::new("ermete-settings-rs").spawn();
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
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "wifi"])
            .spawn();
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
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "bluetooth"])
            .spawn();
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
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "displays"])
            .spawn();
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
        let _ = Command::new("ermete-settings-rs")
            .args(["--page", "audio"])
            .spawn();
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
        
        // Update sliders only if the difference is > 1.0 (to avoid fighting user input)
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

pub fn show_start_menu_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Start Menu")
        .css_classes(["popup-window"])
        .default_width(560)
        .default_height(480)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "launcher");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Left, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Left, 8);

    let main_hbox = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .css_classes(["cc-card"])
        .build();

    let sidebar = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    sidebar.set_margin_top(14);
    sidebar.set_margin_bottom(14);
    sidebar.set_margin_start(14);
    sidebar.set_margin_end(14);

    let cats_lbl = Label::builder().label("CATEGORIE").css_classes(["cc-label-sub"]).halign(Align::Start).margin_bottom(6).build();
    sidebar.append(&cats_lbl);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    let search = Entry::builder()
        .placeholder_text("Cerca nel menu...")
        .css_classes(["spotlight-input"])
        .build();

    let current_category = std::rc::Rc::new(std::cell::RefCell::new("Tutte".to_string()));
    let cats = ["Tutte", "Internet", "Ufficio", "Grafica", "Multimedia", "Sviluppo", "Sistema", "Giochi"];
    
    for cat in cats {
        let btn = Button::builder().label(cat).css_classes(["spotlight-item"]).halign(Align::Fill).build();
        let cat_str = cat.to_string();
        let list_clone = list_box.clone();
        let entry_clone = search.clone();
        let pop_clone = pop.clone();
        let curr_cat = current_category.clone();
        btn.connect_clicked(move |_| {
            *curr_cat.borrow_mut() = cat_str.clone();
            populate_launcher_list(&list_clone, &entry_clone.text(), &cat_str, false, &pop_clone);
        });
        sidebar.append(&btn);
    }

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();
    card.set_margin_top(14);
    card.set_margin_bottom(14);
    card.set_margin_end(14);

    let title = Label::builder()
        .label("◈  MENU APPLICAZIONI ERMETE OS")
        .css_classes(["cc-title"])
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(310)
        .build();

    populate_launcher_list(&list_box, "", "Tutte", false, &pop);

    let list_clone2 = list_box.clone();
    let pop_clone2 = pop.clone();
    let curr_cat2 = current_category.clone();
    search.connect_changed(move |e| {
        populate_launcher_list(&list_clone2, &e.text(), &curr_cat2.borrow(), false, &pop_clone2);
    });

    scroll.set_child(Some(&list_box));

    let footer = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    let off_btn = Button::builder()
        .label("⏻  Spegni")
        .css_classes(["cc-btn-danger"])
        .hexpand(true)
        .build();
    off_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.power_off().await;
        });
    });

    let reb_btn = Button::builder()
        .label("↻  Riavvia")
        .css_classes(["cc-btn"])
        .hexpand(true)
        .build();
    reb_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.reboot().await;
        });
    });

    let susp_btn = Button::builder()
        .label("💤  Sospendi")
        .css_classes(["cc-btn"])
        .hexpand(true)
        .build();
    susp_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.suspend().await;
        });
    });

    footer.append(&off_btn);
    footer.append(&reb_btn);
    footer.append(&susp_btn);

    card.append(&title);
    card.append(&search);
    card.append(&scroll);
    card.append(&footer);

    main_hbox.append(&sidebar);
    
    let sep = gtk4::Separator::new(Orientation::Vertical);
    sep.set_margin_start(4);
    sep.set_margin_end(10);
    main_hbox.append(&sep);
    
    main_hbox.append(&card);

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

    pop.set_child(Some(&main_hbox));
    pop.present();
    search.grab_focus();
}

pub fn show_calendar_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Calendar")
        .css_classes(["popup-window"])
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "calendar");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Right, 10);

    let main_vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let notifs_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["cc-card"])
        .build();
    
    let title_hbox = GtkBox::builder().orientation(Orientation::Horizontal).build();
    let notifs_title = Label::builder().label("Notifiche").css_classes(["cc-title"]).halign(Align::Start).hexpand(true).build();
    let clear_btn = Button::builder().label("Cancella").css_classes(["cc-btn"]).build();
    
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(300)
        .propagate_natural_height(true)
        .build();
    
    let list_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    
    let pop_clone_clear = pop.clone();
    clear_btn.connect_clicked(move |_| {
        NOTIFICATIONS.with(|n| n.borrow_mut().clear());
        pop_clone_clear.close();
    });
    
    title_hbox.append(&notifs_title);
    title_hbox.append(&clear_btn);
    notifs_card.append(&title_hbox);
    
    NOTIFICATIONS.with(|n| {
        let history = n.borrow();
        if history.is_empty() {
            list_box.append(&Label::builder().label("Nessuna nuova notifica").css_classes(["cc-label-sub"]).margin_top(10).margin_bottom(10).build());
        } else {
            for notif in history.iter() {
                let row = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).build();
                let sum = Label::builder().label(&notif.summary).halign(Align::Start).css_classes(["cc-label-main"]).build();
                let bod = Label::builder().label(&notif.body).halign(Align::Start).css_classes(["cc-label-sub"]).wrap(true).max_width_chars(30).build();
                row.append(&sum);
                row.append(&bod);
                list_box.append(&row);
            }
        }
    });

    scroll.set_child(Some(&list_box));
    notifs_card.append(&scroll);

    let cal_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["cc-card"])
        .build();

    let cal = Calendar::builder().build();
    cal_card.append(&cal);

    main_vbox.append(&notifs_card);
    main_vbox.append(&cal_card);

    pop.set_child(Some(&main_vbox));
    pop.present();
}
