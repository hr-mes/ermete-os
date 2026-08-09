use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation};
use crate::components::action_row::ActionRow;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    let title = Label::builder()
        .label("Rete Cablata")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();
    container.append(&title);

    let refresh_btn = Button::builder()
        .label("Verifica Stato")
        .halign(Align::End)
        .valign(Align::Center)
        .build();

    let status_row = ActionRow::builder("Interfaccia Cablata (Ethernet)")
        .subtitle("Rilevamento interfaccia in corso...")
        .suffix(&refresh_btn)
        .build();
    container.append(&status_row);

    let proxy_button = Button::builder()
        .label("Configura Proxy")
        .valign(Align::Center)
        .halign(Align::Start)
        .build();
    proxy_button.connect_clicked(|_| {
        relm4::spawn_local(async move {
            let _ = tokio::process::Command::new("echo")
                .arg("Configurazione proxy richiesta")
                .output()
                .await;
        });
    });

    let proxy_row = ActionRow::builder("Configurazione Proxy Rete")
        .subtitle("Imposta proxy HTTP, HTTPS e SOCKS per la connessione cablata")
        .suffix(&proxy_button)
        .build();
    container.append(&proxy_row);

    let speed_row = ActionRow::builder("Velocità & Duplex")
        .subtitle("Auto-negoziazione (1 Gbps / Full Duplex)")
        .build();
    container.append(&speed_row);

    let ip_row = ActionRow::builder("Indirizzo IPv4 / IPv6")
        .subtitle("Configurazione automatica via DHCP")
        .build();
    container.append(&ip_row);

    // Initial async status detection to never block UI thread during page build
    relm4::spawn_local(async move {
        let _status = get_ethernet_status_async().await;
    });

    let _refresh_btn_clone = refresh_btn.clone();
    refresh_btn.connect_clicked(move |_| {
        relm4::spawn_local(async move {
            let _status = get_ethernet_status_async().await;
        });
    });

    container
}

async fn get_ethernet_status_async() -> String {
    let consumer = crate::pages::network::UnikernelNetworkConsumer::new();
    if consumer.submit_event(crate::pages::network::FRAME_CHECK_CONNECTIVITY, &[]).is_ok() {
        "eth0 - Ring Buffer Status Active (Unikernel SmolTCP)".to_string()
    } else {
        "Nessuna rete cablata rilevata (Unikernel Ring Buffer Offline)".to_string()
    }
}
