use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, DropDown, Label, Orientation, Switch};
use crate::components::action_row::ActionRow;

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    // Title
    let title = Label::builder()
        .label("<span size='xx-large' weight='bold'>Focus Modes &amp; Non Disturbare (DND)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&title);

    // DND Master Toggle
    let dnd_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    let dnd_status = Label::builder().label("🔔 Notifiche normali").halign(Align::Start).build();
    let dnd_status_clone = dnd_status.clone();

    dnd_switch.connect_active_notify(move |switch| {
        if switch.is_active() {
            dnd_status_clone.set_text("🔕 Non Disturbare ATTIVO");
        } else {
            dnd_status_clone.set_text("🔔 Notifiche normali");
        }
    });

    dnd_switch.connect_state_set(move |_, state| {
        relm4::spawn_local(async move {
            if let Ok(conn) = crate::get_connection().await {
                let _ = conn.call_method(
                    Some("org.ermete.Settings"),
                    "/org/ermete/Settings",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.ermete.Settings", "DoNotDisturb", zbus::zvariant::Value::from(state))
                ).await;
            }
        });
        glib::Propagation::Proceed
    });

    let dnd_row = ActionRow::builder("Non Disturbare")
        .subtitle("Blocca tutte le notifiche popup ed i suoni di sistema")
        .suffix(&dnd_switch)
        .build();
    container.append(&dnd_row);
    container.append(&dnd_status);

    // Profiles
    let prof_title = Label::builder()
        .label("<span size='large' weight='bold'>Profili di Concentrazione &amp; Automazioni Niri</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(12)
        .build();
    container.append(&prof_title);

    let dropdown = DropDown::from_strings(&[
        "💼 Lavoro & Programmazione (Silenzia social e chat, mantieni allarmi CI/CD)",
        "📚 Studio & Lettura (Schermo caldo, zero distrazioni)",
        "🎮 Gaming Mode (Bassa latenza, disattiva ombre e notifiche in background)",
        "🌙 Modalità Notturna Relax",
    ]);

    let apply_btn = Button::builder()
        .label("Attiva Profilo")
        .halign(Align::Start)
        .css_classes(vec!["suggested-action"])
        .build();

    let prof_res = Label::builder().label("").halign(Align::Start).build();

    let dropdown_clone = dropdown.clone();
    let prof_res_clone = prof_res.clone();

    apply_btn.connect_clicked(move |_| {
        let sel = dropdown_clone.selected();
        let name = match sel {
            1 => "Studio & Lettura",
            2 => "Gaming Mode",
            3 => "Notturna Relax",
            _ => "Lavoro & Programmazione",
        };
        let name_str = name.to_string();
        let prof_res_c = prof_res_clone.clone();

        relm4::spawn_local(async move {
            crate::niri_client::update_niri_kdl_setting("focus-profile", &name_str).await;
            prof_res_c.set_text(&format!("✅ Profilo '{}' attivato su ermete-shell-rs e Niri IPC.", name_str));
        });
    });

    let prof_row = ActionRow::builder("Profilo Attivo")
        .subtitle("Seleziona le priorità notifiche, animazioni ed ombre grafiche")
        .suffix(&dropdown)
        .build();
    container.append(&prof_row);

    let apply_row = ActionRow::builder("Applicazione Regole")
        .subtitle("Invia le istruzioni a Niri IPC ed alla shell")
        .suffix(&apply_btn)
        .build();
    container.append(&apply_row);
    container.append(&prof_res);

    // Fullscreen behavior
    let bar_title = Label::builder()
        .label("<span size='large' weight='bold'>Comportamento Finestre in Fullscreen</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(12)
        .build();
    container.append(&bar_title);

    let toggle = Switch::builder().valign(Align::Center).build();

    toggle.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        relm4::spawn_local(async move {
            crate::niri_client::update_niri_kdl_setting("hide-bar-on-fullscreen", val).await;
        });
        glib::Propagation::Proceed
    });

    let fs_row = ActionRow::builder("Nascondi Topbar in Fullscreen")
        .subtitle("Nasconde automaticamente il pannello superiore quando un'app è a schermo intero")
        .suffix(&toggle)
        .build();
    container.append(&fs_row);

    container
}
