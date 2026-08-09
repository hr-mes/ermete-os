#![allow(deprecated)]
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DropDown, Entry, Label, ListBox, Orientation};
use crate::components::action_row::ActionRow;

#[zbus::proxy(
    interface = "os.ermete.Bedrock.Network",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock/Network"
)]
trait Network {
    fn scan_networks(&self) -> zbus::Result<Vec<String>>;
    fn check_connectivity(&self) -> zbus::Result<String>;
    fn connect_enterprise_wifi(
        &self,
        ssid: String,
        identity: String,
        password: String,
        eap_method: String,
        ca_cert_path: String,
    ) -> zbus::Result<String>;
    fn add_vpn_tunnel(
        &self,
        name: String,
        vpn_type: String,
        config_path: String,
    ) -> zbus::Result<String>;
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
    title.set_halign(Align::Start);
    container.append(&title);

    // Connectivity & Captive Portal Status Card
    let check_btn = Button::with_label("Aggiorna Stato");
    let status_subtitle = Label::new(Some("Verifica in corso..."));
    status_subtitle.set_halign(Align::Start);
    status_subtitle.add_css_class("action-row-subtitle");

    let conn_row = ActionRow::builder("Stato Connettività")
        .subtitle("Verifica in corso...")
        .suffix(&check_btn)
        .build();

    let conn_row_clone = conn_row.clone();
    check_btn.connect_clicked(move |_| {
        let _row = conn_row_clone.clone();
        let _ctx = gtk4::glib::MainContext::default();
        relm4::spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = NetworkProxy::new(&conn).await {
                    if let Ok(status) = proxy.check_connectivity().await {
                        let text = match status.as_str() {
                            "FULL" => "🌐 Connesso (Accesso Completo a Internet)",
                            "PORTAL" => "⚠️ Captive Portal Rilevato (Richiesto Login)",
                            "LIMITED" => "⚠️ Connessione Limitata",
                            "NONE" => "❌ Nessuna Connessione",
                            other => other,
                        };
                        // Aggiorna la subtitle se possibile ri-creando o impostando il testo
                        let sub_label = Label::new(Some(text));
                        sub_label.set_halign(Align::Start);
                        sub_label.add_css_class("action-row-subtitle");
                    }
                }
            }
        });
    });
    container.append(&conn_row);

    // --- Standard Wi-Fi Scan Section ---
    let wifi_title = Label::new(Some("Reti Wi-Fi Disponibili"));
    wifi_title.add_css_class("title-2");
    wifi_title.set_halign(Align::Start);
    wifi_title.set_margin_top(12);
    container.append(&wifi_title);

    let scan_btn = Button::with_label("Scansiona Reti");
    scan_btn.set_halign(Align::Start);

    let wifi_scan_row = ActionRow::builder("Scansione Wi-Fi")
        .subtitle("Cerca access point nelle vicinanze via NetworkManager")
        .suffix(&scan_btn)
        .build();
    container.append(&wifi_scan_row);

    let list_box = ListBox::new();
    list_box.add_css_class("boxed-list");
    container.append(&list_box);

    let list_box_clone = list_box.clone();
    scan_btn.connect_clicked(move |_| {
        let list_box = list_box_clone.clone();
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        let loading_row = ActionRow::builder("Scansione in corso...")
            .subtitle("Interrogazione access point via D-Bus...")
            .build();
        list_box.append(&loading_row);

        relm4::spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                if let Ok(proxy) = NetworkProxy::new(&conn).await {
                    if let Ok(networks) = proxy.scan_networks().await {
                        while let Some(child) = list_box.first_child() {
                            list_box.remove(&child);
                        }
                        if networks.is_empty() {
                            let empty_row = ActionRow::builder("Nessuna rete trovata")
                                .subtitle("Assicurati che l'interfaccia Wi-Fi sia attiva")
                                .build();
                            list_box.append(&empty_row);
                        } else {
                            for ssid in networks {
                                let connect_net_btn = Button::with_label("Connetti");
                                let row = ActionRow::builder(&ssid)
                                    .subtitle("Rete Wi-Fi Rilevata")
                                    .suffix(&connect_net_btn)
                                    .build();
                                list_box.append(&row);
                            }
                        }
                    }
                }
            }
        });
    });

    // --- Enterprise Wi-Fi 802.1x Section ---
    let ent_title = Label::new(Some("Configurazione Wi-Fi Aziendale (802.1x EAP-TLS / PEAP)"));
    ent_title.add_css_class("title-2");
    ent_title.set_halign(Align::Start);
    ent_title.set_margin_top(16);
    container.append(&ent_title);

    let ent_box = Box::new(Orientation::Vertical, 8);
    ent_box.add_css_class("card");

    let ent_ssid = Entry::builder().placeholder_text("es. Azienda-Corp").build();
    let ent_id = Entry::builder().placeholder_text("es. mario.rossi@azienda.it").build();
    let ent_pwd = Entry::builder().placeholder_text("Password o PIN Token").visibility(false).build();
    let ent_eap = DropDown::from_strings(&["PEAP (MSCHAPv2)", "EAP-TLS (Certificato)", "TTLS"]);
    let ent_ca = Entry::builder().placeholder_text("/etc/pki/tls/cert.pem").build();

    let row_ssid = ActionRow::builder("Nome Rete (SSID)")
        .subtitle("Identificativo SSID aziendale")
        .suffix(&ent_ssid)
        .build();
    let row_id = ActionRow::builder("Identità")
        .subtitle("Utente o nome certificato")
        .suffix(&ent_id)
        .build();
    let row_pwd = ActionRow::builder("Password")
        .subtitle("Credenziale di accesso")
        .suffix(&ent_pwd)
        .build();
    let row_eap = ActionRow::builder("Metodo EAP")
        .subtitle("Seleziona protocollo di autenticazione 802.1x")
        .suffix(&ent_eap)
        .build();
    let row_ca = ActionRow::builder("Certificato CA")
        .subtitle("Percorso del certificato CA di sistema")
        .suffix(&ent_ca)
        .build();

    ent_box.append(&row_ssid);
    ent_box.append(&row_id);
    ent_box.append(&row_pwd);
    ent_box.append(&row_eap);
    ent_box.append(&row_ca);

    let ent_btn = Button::with_label("Attiva Profilo 802.1x Aziendale");
    ent_btn.add_css_class("suggested-action");
    ent_btn.set_halign(Align::Start);

    let ent_status = Label::new(None);
    ent_status.set_halign(Align::Start);

    let row_ent_action = ActionRow::builder("Attivazione 802.1x")
        .subtitle("Salva e applica il profilo sulla scheda di rete")
        .suffix(&ent_btn)
        .build();
    ent_box.append(&row_ent_action);

    container.append(&ent_box);
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

        relm4::spawn_local(async move {
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

    // --- VPN Section ---
    let vpn_title = Label::new(Some("Tunnel VPN Nativi (WireGuard & OpenVPN)"));
    vpn_title.add_css_class("title-2");
    vpn_title.set_halign(Align::Start);
    vpn_title.set_margin_top(16);
    container.append(&vpn_title);

    let vpn_box = Box::new(Orientation::Vertical, 8);
    vpn_box.add_css_class("card");

    let vpn_name = Entry::builder().placeholder_text("es. Azienda-WG").build();
    let vpn_type = DropDown::from_strings(&["WireGuard (wg-quick)", "OpenVPN"]);
    let vpn_path = Entry::builder().placeholder_text("Percorso .conf o .ovpn").build();

    let row_vpn_name = ActionRow::builder("Nome Tunnel")
        .subtitle("Nome identificativo della VPN")
        .suffix(&vpn_name)
        .build();
    let row_vpn_type = ActionRow::builder("Tipo VPN")
        .subtitle("Tecnologia del tunnel")
        .suffix(&vpn_type)
        .build();
    let row_vpn_path = ActionRow::builder("File Configurazione")
        .subtitle("Percorso assoluto del file di configurazione")
        .suffix(&vpn_path)
        .build();

    vpn_box.append(&row_vpn_name);
    vpn_box.append(&row_vpn_type);
    vpn_box.append(&row_vpn_path);

    let vpn_btn = Button::with_label("Aggiungi e Connetti VPN");
    vpn_btn.add_css_class("suggested-action");
    vpn_btn.set_halign(Align::Start);

    let row_vpn_action = ActionRow::builder("Configura VPN")
        .subtitle("Crea ed attiva l'interfaccia VPN")
        .suffix(&vpn_btn)
        .build();
    vpn_box.append(&row_vpn_action);

    container.append(&vpn_box);

    let vpn_status = Label::new(None);
    vpn_status.set_halign(Align::Start);
    container.append(&vpn_status);

    let vpn_status_clone = vpn_status.clone();
    vpn_btn.connect_clicked(move |_| {
        let name = vpn_name.text().to_string();
        let v_type = if vpn_type.selected() == 1 {
            "openvpn".to_string()
        } else {
            "wireguard".to_string()
        };
        let path = vpn_path.text().to_string();
        let status = vpn_status_clone.clone();

        relm4::spawn_local(async move {
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
