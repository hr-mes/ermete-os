use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Entry, Label,
    Orientation, ScrolledWindow,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type)]
struct SnapshotInfo {
    pub id: String,
    pub timestamp: String,
    pub note: String,
    pub path: String,
    pub size_estimate: String,
}

#[zbus::proxy(
    interface = "org.ermete.Backup1",
    default_service = "org.ermete.Backup1",
    default_path = "/org/ermete/Backup1"
)]
trait Backup1 {
    fn create_snapshot(&self, note: &str) -> zbus::Result<SnapshotInfo>;
    fn list_snapshots(&self) -> zbus::Result<Vec<SnapshotInfo>>;
    fn delete_snapshot(&self, id: &str) -> zbus::Result<bool>;
    fn restore_snapshot(&self, id: &str) -> zbus::Result<bool>;
}

fn get_snapshots() -> Vec<SnapshotInfo> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/var/home/ermete".to_string());
    let mut path = PathBuf::from(&home);
    path.push(".snapshots");
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(info) = serde_json::from_str::<SnapshotInfo>(&content) {
                        list.push(info);
                    }
                }
            }
        }
    }
    list.sort_by(|a, b| b.id.cmp(&a.id));
    list
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
            font-size: 26px;
            font-weight: 800;
            color: #f8fafc;
        }
        .header-sub {
            font-size: 14px;
            color: #94a3b8;
            margin-bottom: 20px;
        }
        .card {
            background-color: #1e293b;
            border: 1px solid #334155;
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 12px;
        }
        .card-new {
            background-color: #1f2937;
            border: 1px solid #3b82f6;
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 20px;
        }
        .snap-title {
            font-size: 16px;
            font-weight: 700;
            color: #f1f5f9;
        }
        .snap-meta {
            font-size: 13px;
            color: #64748b;
        }
        .btn-action {
            background-color: #3b82f6;
            color: #ffffff;
            font-weight: 700;
            font-size: 14px;
            padding: 10px 20px;
            border-radius: 8px;
            border: none;
        }
        .btn-action:hover {
            background-color: #2563eb;
        }
        .btn-delete {
            background-color: #ef4444;
            color: #ffffff;
            font-weight: 600;
            font-size: 13px;
            padding: 8px 16px;
            border-radius: 6px;
            border: none;
        }
        .btn-restore {
            background-color: #10b981;
            color: #ffffff;
            font-weight: 700;
            font-size: 13px;
            padding: 8px 16px;
            border-radius: 6px;
            border: none;
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
        .title("Ermete Time Machine — Btrfs Snapshot Manager")
        .default_width(740)
        .default_height(600)
        .build();

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(28)
        .margin_end(28)
        .build();

    let title = Label::builder()
        .label("⏳ Ermete Time Machine — Gestore Istantanee Btrfs")
        .css_classes(["header-title"])
        .halign(Align::Start)
        .build();

    let subtitle = Label::builder()
        .label("Istantanee Copy-on-Write istantanee della cartella utente (/var/home/ermete). Ripristina versioni precedenti con un click.")
        .css_classes(["header-sub"])
        .halign(Align::Start)
        .wrap(true)
        .build();

    main_box.append(&title);
    main_box.append(&subtitle);

    let new_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["card-new"])
        .build();

    let note_entry = Entry::builder()
        .placeholder_text("Nota per l'istantanea es. 'Prima di installare dipendenze'...")
        .hexpand(true)
        .build();

    let create_btn = Button::builder()
        .label("📸 Scatta Istantanea Btrfs")
        .css_classes(["btn-action"])
        .build();

    let entry_clone = note_entry.clone();
    let win_clone = window.clone();
    create_btn.connect_clicked(move |_| {
        let note = entry_clone.text().to_string();
        let note = if note.is_empty() { "Snapshot manuale".to_string() } else { note };
        println!("[Time Machine] Requesting snapshot creation with note: {}", note);
        if let Ok(conn) = zbus::blocking::Connection::system() {
            if let Ok(proxy) = Backup1ProxyBlocking::new(&conn) {
                let _ = proxy.create_snapshot(&note);
            }
        }
        win_clone.close();
    });

    new_card.append(&note_entry);
    new_card.append(&create_btn);
    main_box.append(&new_card);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let snapshots = get_snapshots();
    if snapshots.is_empty() {
        let empty_lbl = Label::builder()
            .label("Nessuna istantanea presente. Scatta la prima istantanea ora.")
            .css_classes(["snap-meta"])
            .margin_top(40)
            .build();
        list_box.append(&empty_lbl);
    } else {
        for snap in &snapshots {
            let card = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(12)
                .css_classes(["card"])
                .build();

            let info_box = GtkBox::builder()
                .orientation(Orientation::Vertical)
                .spacing(4)
                .hexpand(true)
                .build();

            let title_lbl = Label::builder()
                .label(&format!("📸 {} ({})", snap.note, snap.id))
                .css_classes(["snap-title"])
                .halign(Align::Start)
                .build();

            let meta_lbl = Label::builder()
                .label(&format!("Creato il: {} | Spazio stimato: {}", snap.timestamp, snap.size_estimate))
                .css_classes(["snap-meta"])
                .halign(Align::Start)
                .build();

            info_box.append(&title_lbl);
            info_box.append(&meta_lbl);

            let act_box = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .valign(Align::Center)
                .build();

            let restore_btn = Button::builder()
                .label("🔄 Ripristina")
                .css_classes(["btn-restore"])
                .build();
            let snap_id = snap.id.clone();
            restore_btn.connect_clicked(move |_| {
                println!("[Time Machine] Restoring snapshot {}", snap_id);
                if let Ok(conn) = zbus::blocking::Connection::system() {
                    if let Ok(proxy) = Backup1ProxyBlocking::new(&conn) {
                        let _ = proxy.restore_snapshot(&snap_id);
                    }
                }
            });

            let delete_btn = Button::builder()
                .label("🗑️")
                .css_classes(["btn-delete"])
                .build();
            let snap_id_del = snap.id.clone();
            let w_clone = window.clone();
            delete_btn.connect_clicked(move |_| {
                if let Ok(conn) = zbus::blocking::Connection::system() {
                    if let Ok(proxy) = Backup1ProxyBlocking::new(&conn) {
                        let _ = proxy.delete_snapshot(&snap_id_del);
                    }
                }
                w_clone.close();
            });

            act_box.append(&restore_btn);
            act_box.append(&delete_btn);

            card.append(&info_box);
            card.append(&act_box);
            list_box.append(&card);
        }
    }

    scroll.set_child(Some(&list_box));
    main_box.append(&scroll);

    window.set_child(Some(&main_box));
    window.present();
}

fn main() {
    let app = Application::builder()
        .application_id("org.ermete.BackupUI")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_info_serialization_and_parsing() {
        let info = SnapshotInfo {
            id: "snap-20260715-120000".to_string(),
            timestamp: "15/07/2026 12:00:00".to_string(),
            note: "Test note".to_string(),
            path: "/var/home/ermete/.snapshots/snap-20260715-120000".to_string(),
            size_estimate: "0 B".to_string(),
        };
        let json = serde_json::to_string(&info).expect("Failed to serialize SnapshotInfo");
        let parsed: SnapshotInfo = serde_json::from_str(&json).expect("Failed to parse SnapshotInfo");
        assert_eq!(parsed.id, info.id);
        assert_eq!(parsed.note, info.note);
    }

    #[test]
    fn test_proxy_interface_and_structure() {
        assert_eq!(<Backup1Proxy as zbus::ProxyDefault>::DESTINATION, Some("org.ermete.Backup1"));
        assert_eq!(<Backup1Proxy as zbus::ProxyDefault>::PATH, Some("/org/ermete/Backup1"));
        assert_eq!(<Backup1Proxy as zbus::ProxyDefault>::INTERFACE, Some("org.ermete.Backup1"));
    }
}

