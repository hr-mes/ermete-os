use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ScrolledWindow, TextView};
use std::process::Command;

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Centro Sicurezza e Audit (Zero-Trust)")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();

    let desc = Label::builder()
        .label("Visualizza in tempo reale i tentativi di accesso bloccati dal Firewall XDP e dai Portali Fail-Closed.")
        .halign(Align::Start)
        .wrap(true)
        .css_classes(["dim-label"])
        .build();

    let log_view = TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(gtk4::WrapMode::WordChar)
        .build();
        
    // Fetch logs from portal.rs (Fail-Closed blocks)
    let output = Command::new("journalctl")
        .args(["-t", "xdg-desktop-portal-ermete", "-p", "warning", "-n", "50"])
        .output();
        
    let log_text = match output {
        Ok(out) => {
            let logs = String::from_utf8_lossy(&out.stdout).to_string();
            if logs.trim().is_empty() {
                "Nessuna violazione di sicurezza rilevata di recente.".to_string()
            } else {
                logs
            }
        },
        Err(_) => "Impossibile leggere il registro di sistema.".to_string(),
    };
    
    log_view.buffer().set_text(&log_text);

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&log_view)
        .build();

    container.append(&title);
    container.append(&desc);
    container.append(&scroll);

    container
}
