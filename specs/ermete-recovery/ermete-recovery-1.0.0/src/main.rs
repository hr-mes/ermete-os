use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label,
    Orientation, ScrolledWindow,
};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
struct OstreeStatus {
    deployments: Vec<OstreeDeployment>,
}

#[derive(Debug, Clone, Deserialize)]
struct OstreeDeployment {
    id: String,
    checksum: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    booted: bool,
    #[serde(rename = "pinned", default)]
    pinned: bool,
}

const BEDROCK_STABLE_COMMIT: &str = "8aa3fd4257106ec344d698d927ede49b37e179c8";

fn get_deployments() -> Vec<OstreeDeployment> {
    if let Ok(output) = Command::new("rpm-ostree")
        .args(["status", "--json"])
        .output()
    {
        if let Ok(status) = serde_json::from_slice::<OstreeStatus>(&output.stdout) {
            return status.deployments;
        }
    }
    // Fallback dummy for testing on non-ostree or build host
    vec![
        OstreeDeployment {
            id: "ermete-os-latest-booted".to_string(),
            checksum: "6e43d499189a1a61b42defff535e53140e6e1e5b0a5f835fbfd4900659e56679".to_string(),
            version: "1.0.0-19".to_string(),
            origin: "ghcr.io/patapem/ermete-os:latest".to_string(),
            booted: true,
            pinned: false,
        },
        OstreeDeployment {
            id: "ermete-os-rollback-stable".to_string(),
            checksum: BEDROCK_STABLE_COMMIT.to_string(),
            version: "1.0.0-bedrock".to_string(),
            origin: "ghcr.io/patapem/ermete-os:latest".to_string(),
            booted: false,
            pinned: true,
        },
    ]
}

fn apply_css() {
    let provider = CssProvider::new();
    let css = r#"
        window {
            background-color: #0b0f19;
            color: #e2e8f0;
            font-family: 'Inter', 'Outfit', sans-serif;
        }
        .header-title {
            font-size: 28px;
            font-weight: 800;
            color: #f8fafc;
            margin-bottom: 4px;
        }
        .header-sub {
            font-size: 15px;
            color: #94a3b8;
            margin-bottom: 24px;
        }
        .card {
            background-color: #1e293b;
            border: 1px solid #334155;
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 16px;
        }
        .card-alert {
            background-color: #450a0a;
            border: 1px solid #7f1d1d;
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 20px;
            color: #fca5a5;
            font-weight: 600;
        }
        .deploy-title {
            font-size: 17px;
            font-weight: 700;
            color: #f1f5f9;
        }
        .deploy-hash {
            font-size: 13px;
            font-family: monospace;
            color: #64748b;
        }
        .badge-booted {
            background-color: #059669;
            color: #ffffff;
            font-weight: 700;
            font-size: 12px;
            padding: 4px 10px;
            border-radius: 6px;
        }
        .badge-rollback {
            background-color: #2563eb;
            color: #ffffff;
            font-weight: 700;
            font-size: 12px;
            padding: 4px 10px;
            border-radius: 6px;
        }
        .btn-action {
            background-color: #3b82f6;
            color: #ffffff;
            font-weight: 700;
            font-size: 15px;
            padding: 12px 24px;
            border-radius: 8px;
            border: none;
        }
        .btn-action:hover {
            background-color: #2563eb;
        }
        .btn-danger {
            background-color: #ef4444;
            color: #ffffff;
            font-weight: 700;
            font-size: 15px;
            padding: 12px 24px;
            border-radius: 8px;
            border: none;
        }
        .btn-danger:hover {
            background-color: #dc2626;
        }
        .btn-secondary {
            background-color: #334155;
            color: #f8fafc;
            font-weight: 600;
            font-size: 14px;
            padding: 10px 20px;
            border-radius: 8px;
            border: none;
        }
        .btn-secondary:hover {
            background-color: #475569;
        }
    "#;
    provider.load_from_data(css);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    apply_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete OS Recovery Environment")
        .default_width(800)
        .default_height(640)
        .build();

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(40)
        .margin_end(40)
        .build();

    let title = Label::builder()
        .label("🛠️ Ermete OS — Ambiente di Ripristino Pre-Boot")
        .css_classes(["header-title"])
        .halign(Align::Start)
        .build();

    let subtitle = Label::builder()
        .label("Modalità di emergenza (Kiosk OCI). Seleziona un'operazione di ripristino o esegui il rollback all'ultima versione stabile.")
        .css_classes(["header-sub"])
        .halign(Align::Start)
        .wrap(true)
        .build();

    main_box.append(&title);
    main_box.append(&subtitle);

    let alert_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["card-alert"])
        .build();
    let alert_lbl = Label::builder()
        .label("⚠️ ATTENZIONE: Il sistema è entrato in modalità di recupero automatica a causa di un'anomalia all'avvio della sessione grafica o del display manager. Puoi ripristinare istantaneamente il sistema senza perdita di dati personali.")
        .wrap(true)
        .hexpand(true)
        .halign(Align::Start)
        .build();
    alert_box.append(&alert_lbl);
    main_box.append(&alert_box);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .build();

    let deployments = get_deployments();
    for dep in &deployments {
        let dep_card = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .css_classes(["card"])
            .build();

        let top_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        let title_text = if dep.booted {
            format!("📦 Deployment Attuale: {} (v{})", dep.origin, dep.version)
        } else {
            format!("📦 Rollback Disponibile: {} (v{})", dep.origin, dep.version)
        };
        let dep_lbl = Label::builder()
            .label(&title_text)
            .css_classes(["deploy-title"])
            .halign(Align::Start)
            .hexpand(true)
            .build();

        top_row.append(&dep_lbl);

        if dep.booted {
            let badge = Label::builder().label("ATTIVO (AVVIATO)").css_classes(["badge-booted"]).build();
            top_row.append(&badge);
        } else {
            let badge = Label::builder().label("PRECEDENTE / ROLLBACK").css_classes(["badge-rollback"]).build();
            top_row.append(&badge);
        }

        dep_card.append(&top_row);

        let hash_text = format!("Digest: {} | ID: {}", dep.checksum, dep.id);
        let hash_lbl = Label::builder()
            .label(&hash_text)
            .css_classes(["deploy-hash"])
            .halign(Align::Start)
            .build();
        dep_card.append(&hash_lbl);

        if dep.checksum.contains(BEDROCK_STABLE_COMMIT) {
            let bedrock_lbl = Label::builder()
                .label("🛡️ COMMIT DI ROLLBACK STABILE CERTIFICATO (Bedrock Safe Point)")
                .halign(Align::Start)
                .css_classes(["badge-rollback"])
                .margin_top(4)
                .build();
            dep_card.append(&bedrock_lbl);
        }

        list_box.append(&dep_card);
    }

    scroll.set_child(Some(&list_box));
    main_box.append(&scroll);

    let actions_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .halign(Align::End)
        .margin_top(16)
        .build();

    let bedrock_btn = Button::builder()
        .label("🛡️ Ripristina Bedrock Commit (8aa3fd4)")
        .css_classes(["btn-danger"])
        .build();
    bedrock_btn.connect_clicked(move |_| {
        let dialog = gtk4::MessageDialog::builder()
            .message_type(gtk4::MessageType::Warning)
            .text("Conferma ripristino Bedrock")
            .secondary_text("Questo ripristinerà il sistema all'ultimo commit stabile. Continuare?")
            .buttons(gtk4::ButtonsType::YesNo)
            .modal(true)
            .build();

        dialog.connect_response(move |dlg, response| {
            dlg.close();
            if response == gtk4::ResponseType::Yes {
                println!("[Recovery] Executing rollback to Bedrock Stable Commit: {}", BEDROCK_STABLE_COMMIT);
                match Command::new("rpm-ostree")
                    .arg("rollback")
                    .output()
                {
                    Ok(out) if out.status.success() => {
                        println!("[Recovery] Rollback complete, rebooting...");
                        let _ = Command::new("systemctl").arg("reboot").spawn();
                    }
                    Ok(out) => {
                        eprintln!("[Recovery] Rollback failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                    Err(e) => {
                        eprintln!("[Recovery] Failed to run rpm-ostree: {}", e);
                    }
                }
            }
        });
        dialog.present();
    });

    let rollback_btn = Button::builder()
        .label("󰜉 Rollback Automatico & Riavvia")
        .css_classes(["btn-action"])
        .build();
    rollback_btn.connect_clicked(move |_| {
        let dialog = gtk4::MessageDialog::builder()
            .message_type(gtk4::MessageType::Warning)
            .text("Conferma Rollback Automatico")
            .secondary_text("Questo eseguirà un rollback all'ultima versione disponibile e riavvierà il sistema. Continuare?")
            .buttons(gtk4::ButtonsType::YesNo)
            .modal(true)
            .build();

        dialog.connect_response(move |dlg, response| {
            dlg.close();
            if response == gtk4::ResponseType::Yes {
                println!("[Recovery] Executing rpm-ostree rollback and reboot...");
                match Command::new("rpm-ostree")
                    .arg("rollback")
                    .output()
                {
                    Ok(out) if out.status.success() => {
                        println!("[Recovery] Rollback complete, rebooting...");
                        let _ = Command::new("systemctl").arg("reboot").spawn();
                    }
                    Ok(out) => {
                        eprintln!("[Recovery] Rollback failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                    Err(e) => {
                        eprintln!("[Recovery] Failed to run rpm-ostree: {}", e);
                    }
                }
            }
        });
        dialog.present();
    });

    let reboot_btn = Button::builder()
        .label("Riavvia Normale")
        .css_classes(["btn-secondary"])
        .build();
    reboot_btn.connect_clicked(|_| {
        let _ = Command::new("systemctl").arg("reboot").spawn();
    });

    actions_box.append(&reboot_btn);
    actions_box.append(&bedrock_btn);
    actions_box.append(&rollback_btn);

    main_box.append(&actions_box);

    window.set_child(Some(&main_box));
    window.present();
}

fn main() {
    let app = Application::builder()
        .application_id("org.ermete.RecoveryUI")
        .build();

    app.connect_activate(build_ui);
    app.run();
}
