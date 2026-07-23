use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, ListBox, Orientation, DropDown};

#[zbus::dbus_proxy(
    interface = "os.ermete.Bedrock.Network",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock/Network"
)]
trait Network {
    fn scan_networks(&self) -> zbus::Result<Vec<String>>;
    fn check_connectivity(&self) -> zbus::Result<String>;
    fn connect_enterprise_wifi(&self, ssid: String, identity: String, password: String, eap_method: String, ca_cert_path: String) -> zbus::Result<String>;
    fn add_vpn_tunnel(&self, name: String, vpn_type: String, config_path: String) -> zbus::Result<String>;
}

pub fn build_page() -> Box {
    let container = Box::new(Orientation::Vertical, 20);
    container.set_margin_top(24);
    container.set_margin_bottom(32);
    container.set_margin_start(24);
    container.set_margin_end(24);

    // Title
    let title = Label::new(Some("Rete, Wi-Fi Aziendale & VPN"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);
    container.append(&title);

    // Connectivity & Captive Portal status box
    let status_box = Box::new(Orientation::Horizontal, 12);
    status_box.add_css_class("card");
    let status_label = Label::new(Some("Stato Connettività: Verifica in corso..."));
    status_label.set_hexpand(true);
    status_label.set_halign(gtk4::Align::Start);
    let check_btn = Button::with_label("Aggiorna Stato");
    status_box.append(&status_label);
    status_box.append(&check_btn);
    container.append(&status_box);

    let status_label_clone = status_label.clone();
    check_btn.connect_clicked(move |_| {
        let label = status_label_clone.clone();
        label.set_text("Stato Connettività: Interrogazione D-Bus...");
        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = NetworkProxy::new(&conn).await {
                    if let Ok(status) = proxy.check_connectivity().await {
                        let text = match status.as_str() {
                            "FULL" => "Stato Connettività: 🌐 Connesso (Accesso Completo a Internet)".to_string(),
                            "PORTAL" => "Stato Connettività: ⚠️ Captive Portal Rilevato (Richiesto Login)".to_string(),
                            "LIMITED" => "Stato Connettività: ⚠️ Connessione Limitata".to_string(),
                            "NONE" => "Stato Connettività: ❌ Nessuna Connessione".to_string(),
                            _ => format!("Stato Connettività: {}", status),
                        };
                        label.set_text(&text);
                    }
                }
            }
        });
    });

    // --- Standard Wi-Fi Scan Section ---
    let wifi_title = Label::new(Some("Reti Wi-Fi Disponibili"));
    wifi_title.add_css_class("title-2");
    wifi_title.set_halign(gtk4::Align::Start);
    container.append(&wifi_title);

    let scan_btn = Button::with_label("Scansiona Reti");
    scan_btn.set_halign(gtk4::Align::Start);
    container.append(&scan_btn);

    let list_box = ListBox::new();
    list_box.add_css_class("boxed-list");
    container.append(&list_box);

    let list_box_clone = list_box.clone();
    scan_btn.connect_clicked(move |_| {
        let list_box = list_box_clone.clone();
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        let loading_label = Label::new(Some("Scansione access point via NetworkManager..."));
        loading_label.set_margin_top(12);
        loading_label.set_margin_bottom(12);
        list_box.append(&loading_label);

        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = NetworkProxy::new(&conn).await {
                    if let Ok(networks) = proxy.scan_networks().await {
                        while let Some(child) = list_box.first_child() {
                            list_box.remove(&child);
                        }
                        for ssid in networks {
                            let label = Label::new(Some(&ssid));
                            label.set_halign(gtk4::Align::Start);
                            label.set_margin_top(10);
                            label.set_margin_bottom(10);
                            label.set_margin_start(12);
                            list_box.append(&label);
                        }
                    }
                }
            }
        });
    });

    // --- Enterprise Wi-Fi 802.1x (EAP-TLS / PEAP) Section ---
    let ent_title = Label::new(Some("Configurazione Wi-Fi Aziendale (802.1x EAP-TLS / PEAP)"));
    ent_title.add_css_class("title-2");
    ent_title.set_halign(gtk4::Align::Start);
    ent_title.set_margin_top(12);
    container.append(&ent_title);

    let ent_box = Box::new(Orientation::Vertical, 10);
    ent_box.add_css_class("card");
    
    let ent_ssid = Entry::builder().placeholder_text("Nome Rete (SSID)").build();
    let ent_id = Entry::builder().placeholder_text("Identità / Nome Utente o Certificato").build();
    let ent_pwd = Entry::builder().placeholder_text("Password o PIN Token").visibility(false).build();
    let ent_eap = DropDown::from_strings(&["PEAP (MSCHAPv2)", "EAP-TLS (Certificato)", "TTLS"]);
    let ent_ca = Entry::builder().placeholder_text("Percorso Certificato CA (/etc/pki/tls/cert.pem)").build();
    
    ent_box.append(&ent_ssid);
    ent_box.append(&ent_id);
    ent_box.append(&ent_pwd);
    ent_box.append(&ent_eap);
    ent_box.append(&ent_ca);

    let ent_btn = Button::with_label("Attiva Profilo 802.1x Aziendale");
    ent_btn.add_css_class("suggested-action");
    ent_btn.set_halign(gtk4::Align::Start);
    ent_box.append(&ent_btn);
    container.append(&ent_box);

    let ent_status = Label::new(None);
    ent_status.set_halign(gtk4::Align::Start);
    container.append(&ent_status);

    let ent_status_clone = ent_status.clone();
    ent_btn.connect_clicked(move |_| {
        let ssid = ent_ssid.text().to_string();
        let id = ent_id.text().to_string();
        let pwd = ent_pwd.text().to_string();
        let eap = match ent_eap.selected() {
            1 => "tls".to_string(),
            2 => "ttls".to_string(),
            _ => "peap".to_string(),
        };
        let ca = ent_ca.text().to_string();
        let status = ent_status_clone.clone();

        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = NetworkProxy::new(&conn).await {
                    match proxy.connect_enterprise_wifi(ssid, id, pwd, eap, ca).await {
                        Ok(res) => status.set_text(&format!("✅ {}", res)),
                        Err(e) => status.set_text(&format!("❌ Errore: {:?}", e)),
                    }
                }
            }
        });
    });

    // --- VPN Section (WireGuard & OpenVPN) ---
    let vpn_title = Label::new(Some("Tunnel VPN Nativi (WireGuard & OpenVPN)"));
    vpn_title.add_css_class("title-2");
    vpn_title.set_halign(gtk4::Align::Start);
    vpn_title.set_margin_top(12);
    container.append(&vpn_title);

    let vpn_box = Box::new(Orientation::Vertical, 10);
    vpn_box.add_css_class("card");

    let vpn_name = Entry::builder().placeholder_text("Nome Tunnel (es. Azienda-WG)").build();
    let vpn_type = DropDown::from_strings(&["WireGuard (wg-quick)", "OpenVPN"]);
    let vpn_path = Entry::builder().placeholder_text("Percorso file configurazione (.conf o .ovpn)").build();

    vpn_box.append(&vpn_name);
    vpn_box.append(&vpn_type);
    vpn_box.append(&vpn_path);

    let vpn_btn = Button::with_label("Aggiungi e Connetti VPN");
    vpn_btn.add_css_class("suggested-action");
    vpn_btn.set_halign(gtk4::Align::Start);
    vpn_box.append(&vpn_btn);
    container.append(&vpn_box);

    let vpn_status = Label::new(None);
    vpn_status.set_halign(gtk4::Align::Start);
    container.append(&vpn_status);

    let vpn_status_clone = vpn_status.clone();
    vpn_btn.connect_clicked(move |_| {
        let name = vpn_name.text().to_string();
        let v_type = if vpn_type.selected() == 1 { "openvpn".to_string() } else { "wireguard".to_string() };
        let path = vpn_path.text().to_string();
        let status = vpn_status_clone.clone();

        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = NetworkProxy::new(&conn).await {
                    match proxy.add_vpn_tunnel(name, v_type, path).await {
                        Ok(res) => status.set_text(&format!("✅ {}", res)),
                        Err(e) => status.set_text(&format!("❌ Errore: {:?}", e)),
                    }
                }
            }
        });
    });

    container
}
