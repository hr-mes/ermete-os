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
    if let Ok(mut dir) = tokio::fs::read_dir("/sys/class/net").await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            if (name.starts_with("eth") || name.starts_with("en"))
                && !name.starts_with("enx")
                && !name.starts_with("lo")
            {
                let state_path = format!("/sys/class/net/{}/operstate", name);
                if let Ok(state) = tokio::fs::read_to_string(state_path).await {
                    let st = state.trim();
                    let st_label = match st {
                        "up" => "Connesso",
                        "down" => "Scollegato",
                        "unknown" => "Stato sconosciuto",
                        other => other,
                    };
                    return format!("{} - {}", name, st_label);
                }
            }
        }
    }
    "Nessuna rete cablata rilevata".to_string()
}
