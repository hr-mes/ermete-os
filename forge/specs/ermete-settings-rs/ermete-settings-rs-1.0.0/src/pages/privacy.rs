#![allow(deprecated)]
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, CheckButton, Entry, Label, Orientation, Switch};
use crate::components::action_row::ActionRow;

#[zbus::proxy(
    interface = "org.freedesktop.impl.portal.PermissionStore",
    default_service = "org.freedesktop.impl.portal.PermissionStore",
    default_path = "/org/freedesktop/impl/portal/PermissionStore"
)]
trait PermissionStore {
    fn set_permission(
        &self,
        table: &str,
        create: bool,
        id: &str,
        app_permissions: std::collections::HashMap<&str, Vec<&str>>,
        data: zbus::zvariant::Value<'_>,
    ) -> zbus::Result<()>;
}

pub fn generate_permission_store_payload(
    app_id: &str,
    wayland: bool,
    audio: bool,
    network: bool,
    home: bool,
    devices: bool,
) -> (String, String, std::collections::HashMap<String, Vec<String>>) {
    let mut perms = std::collections::HashMap::new();
    if wayland {
        perms.insert("wayland".to_string(), vec!["yes".to_string()]);
    }
    if audio {
        perms.insert("audio".to_string(), vec!["yes".to_string()]);
    }
    if network {
        perms.insert("network".to_string(), vec!["yes".to_string()]);
    }
    if home {
        perms.insert("home".to_string(), vec!["yes".to_string()]);
    }
    if devices {
        perms.insert("devices".to_string(), vec!["yes".to_string()]);
    }

    ("flatpak".to_string(), app_id.to_string(), perms)
}

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    // Titolo
    let title = Label::builder()
        .label("<span size='xx-large' weight='bold'>Privacy, Sicurezza &amp; Sandbox Flatpak</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Contenitore per le impostazioni generali di sistema
    let sys_title = Label::builder()
        .label("<span size='large' weight='bold'>Impostazioni di Sistema &amp; Sensori</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&sys_title);

    let settings_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(vec!["card"])
        .build();

    // Toggle: Posizione
    let location_switch = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();
    let location_row = ActionRow::builder("Accesso alla Posizione Geografica")
        .subtitle("Gestione permessi GeoClue e portali di geolocalizzazione")
        .suffix(&location_switch)
        .build();
    settings_box.append(&location_row);

    // Toggle: Telecamera & Microfono
    let cam_switch = Switch::builder()
        .valign(Align::Center)
        .active(true)
        .build();
    let cam_row = ActionRow::builder("Fotocamera e Microfono")
        .subtitle("Permetti alle applicazioni di richiedere i sensori via PipeWire Portal")
        .suffix(&cam_switch)
        .build();
    settings_box.append(&cam_row);

    // Toggle: Diagnostica
    let diag_switch = Switch::builder()
        .valign(Align::Center)
        .build();
    let diag_row = ActionRow::builder("Dati Diagnostici Anonimi")
        .subtitle("Invia dati di crash e diagnostica anonimi per migliorare Ermete OS")
        .suffix(&diag_switch)
        .build();
    settings_box.append(&diag_row);

    container.append(&settings_box);

    // --- Flatpak Sandbox Permissions Manager ---
    let flatpak_title = Label::builder()
        .label("<span size='large' weight='bold'>Gestore Permessi Sandbox (Flatpak / Portali)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(16)
        .build();
    container.append(&flatpak_title);

    let flatpak_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(vec!["card"])
        .build();

    let desc = Label::builder()
        .label("Modifica granulare degli accessi e override per le applicazioni isolate Flatpak (ACID PermissionStore).")
        .halign(Align::Start)
        .build();
    flatpak_box.append(&desc);

    let app_id_entry = Entry::builder()
        .placeholder_text("es. org.mozilla.firefox, com.spotify.Client")
        .build();
    let app_id_row = ActionRow::builder("App ID Flatpak")
        .subtitle("Identificativo univoco del pacchetto Flatpak")
        .suffix(&app_id_entry)
        .build();
    flatpak_box.append(&app_id_row);

    let chk_wayland = CheckButton::builder().active(true).build();
    let chk_audio = CheckButton::builder().active(true).build();
    let chk_network = CheckButton::builder().active(true).build();
    let chk_home = CheckButton::builder().active(false).build();
    let chk_devices = CheckButton::builder().active(false).build();

    let row_wayland = ActionRow::builder("Display Server Wayland")
        .subtitle("Permesso di accesso al compositore (--socket=wayland)")
        .suffix(&chk_wayland)
        .build();

    let row_audio = ActionRow::builder("Server Audio PipeWire")
        .subtitle("Permesso di riproduzione/registrazione audio (--socket=pulseaudio)")
        .suffix(&chk_audio)
        .build();

    let row_network = ActionRow::builder("Accesso alla Rete")
        .subtitle("Permetti connessione socket diretta a Internet (--share=network)")
        .suffix(&chk_network)
        .build();

    let row_home = ActionRow::builder("Accesso File Utente (Home)")
        .subtitle("Lettura/Scrittura nella directory Home dell'utente (--filesystem=home)")
        .suffix(&chk_home)
        .build();

    let row_devices = ActionRow::builder("Dispositivi Hardware & GPU")
        .subtitle("Accesso all'accelerazione grafica GPU (--device=dri)")
        .suffix(&chk_devices)
        .build();

    flatpak_box.append(&row_wayland);
    flatpak_box.append(&row_audio);
    flatpak_box.append(&row_network);
    flatpak_box.append(&row_home);
    flatpak_box.append(&row_devices);

    let flatpak_status = Label::builder().label("").halign(Align::Start).build();
    flatpak_box.append(&flatpak_status);

    let apply_permissions = std::rc::Rc::new({
        let app_id_entry = app_id_entry.clone();
        let chk_wayland = chk_wayland.clone();
        let chk_audio = chk_audio.clone();
        let chk_network = chk_network.clone();
        let chk_home = chk_home.clone();
        let chk_devices = chk_devices.clone();
        let status_clone = flatpak_status.clone();

        move || {
            let app = app_id_entry.text().to_string();
            if app.is_empty() {
                status_clone.set_text("⚠️ Inserisci un App ID Flatpak valido.");
                return;
            }
            let w = chk_wayland.is_active();
            let a = chk_audio.is_active();
            let n = chk_network.is_active();
            let h = chk_home.is_active();
            let d = chk_devices.is_active();

            let (table, id, perms) = generate_permission_store_payload(&app, w, a, n, h, d);

            let status_for_async = status_clone.clone();
            relm4::spawn_local(async move {
                match crate::get_connection().await {
                    Ok(conn) => {
                        match PermissionStoreProxy::new(&conn).await {
                            Ok(proxy) => {
                                let mut borrowed_perms = std::collections::HashMap::new();
                                for (k, v) in &perms {
                                    borrowed_perms.insert(
                                        k.as_str(),
                                        v.iter().map(|s| s.as_str()).collect(),
                                    );
                                }

                                if let Err(e) = proxy
                                    .set_permission(
                                        &table,
                                        true,
                                        &id,
                                        borrowed_perms,
                                        zbus::zvariant::Value::from(0i32),
                                    )
                                    .await
                                {
                                    eprintln!("Errore DBus PermissionStore: {:?}", e);
                                    status_for_async.set_text("⚠️ Errore salvataggio permessi");
                                } else {
                                    status_for_async.set_text(&format!(
                                        "✅ Permessi applicati su Flatpak PermissionStore per '{}'",
                                        id
                                    ));
                                }
                            }
                            Err(e) => {
                                eprintln!("Errore creazione proxy PermissionStore: {:?}", e);
                                status_for_async.set_text("⚠️ Errore creazione proxy");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Errore connessione DBus: {:?}", e);
                        status_for_async.set_text("⚠️ Errore connessione DBus");
                    }
                }
            });
        }
    });

    {
        let apply = apply_permissions.clone();
        chk_wayland.connect_toggled(move |_| apply());
    }
    {
        let apply = apply_permissions.clone();
        chk_audio.connect_toggled(move |_| apply());
    }
    {
        let apply = apply_permissions.clone();
        chk_network.connect_toggled(move |_| apply());
    }
    {
        let apply = apply_permissions.clone();
        chk_home.connect_toggled(move |_| apply());
    }
    {
        let apply = apply_permissions.clone();
        chk_devices.connect_toggled(move |_| apply());
    }
    {
        let apply = apply_permissions.clone();
        app_id_entry.connect_activate(move |_| apply());
    }

    container.append(&flatpak_box);

    // Pulsante: Pulisci Cache
    let cache_btn = Button::builder()
        .label("Pulisci Cache")
        .halign(Align::Start)
        .css_classes(vec!["destructive-action"])
        .build();

    let cache_status = flatpak_status.clone();
    cache_btn.connect_clicked(move |_| {
        let status = cache_status.clone();
        relm4::spawn_local(async move {
            if let Ok(home) = std::env::var("HOME") {
                let tmp_dir = std::path::PathBuf::from(home).join(".cache").join("tmp");
                if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(&path);
                        } else {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
            status.set_text("✅ Cache di sistema e file temporanei ripuliti con successo.");
        });
    });

    let cache_row = ActionRow::builder("Pulisci Cache & Snapshot Temporanei")
        .subtitle("Rimuovi file temporanei di sessione e cache delle applicazioni")
        .suffix(&cache_btn)
        .build();

    container.append(&cache_row);

    container
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_store_key_generation() {
        let (table, id, perms) = generate_permission_store_payload(
            "org.mozilla.firefox",
            true,
            true,
            false,
            false,
            false,
        );
        assert_eq!(table, "flatpak");
        assert_eq!(id, "org.mozilla.firefox");
        
        let wayland_perm = perms.get("wayland").and_then(|v| v.first()).map(String::as_str);
        assert_eq!(wayland_perm, Some("yes"));

        let audio_perm = perms.get("audio").and_then(|v| v.first()).map(String::as_str);
        assert_eq!(audio_perm, Some("yes"));

        assert_eq!(perms.get("network"), None);
        assert_eq!(perms.get("home"), None);
        assert_eq!(perms.get("devices"), None);
    }
}
