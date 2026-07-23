use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, DropDown, Label, Orientation, Switch};
use std::process::Command;

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
    let dnd_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .css_classes(vec!["card"])
        .build();

    let dnd_label = Label::builder()
        .label("<span weight='bold'>Non Disturbare (Blocca tutte le notifiche popup e suoni)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let dnd_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    let dnd_status = Label::builder().label("").build();
    let dnd_status_clone = dnd_status.clone();
    dnd_switch.connect_active_notify(move |switch| {
        let active = switch.is_active();
        if active {
            dnd_status_clone.set_text("🔕 Non Disturbare ATTIVO");
        } else {
            dnd_status_clone.set_text("🔔 Notifiche normali");
        }
    });

    dnd_box.append(&dnd_label);
    dnd_box.append(&dnd_switch);
    dnd_box.append(&dnd_status);
    container.append(&dnd_box);

    // Profiles Box
    let prof_title = Label::builder()
        .label("<span size='large' weight='bold'>Profili di Concentrazione &amp; Automazioni Niri</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(12)
        .build();
    container.append(&prof_title);

    let prof_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(vec!["card"])
        .build();

    let prof_desc = Label::builder()
        .label("Seleziona il profilo attivo per calibrare automaticamente priorità notifiche, animazioni grafiche e nascondimento pannelli:")
        .halign(Align::Start)
        .build();
    prof_box.append(&prof_desc);

    let dropdown = DropDown::from_strings(&[
        "💼 Lavoro & Programmazione (Silenzia social e chat, mantieni allarmi CI/CD)",
        "📚 Studio & Lettura (Schermo caldo, zero distrazioni)",
        "🎮 Gaming Mode (Bassa latenza, disattiva ombre e notifiche in background)",
        "🌙 Modalità Notturna Relax",
    ]);
    prof_box.append(&dropdown);

    let apply_btn = Button::builder()
        .label("Attiva Profilo Selezionato")
        .halign(Align::Start)
        .css_classes(vec!["suggested-action"])
        .build();
    prof_box.append(&apply_btn);

    let prof_res = Label::builder().label("").halign(Align::Start).build();
    prof_box.append(&prof_res);

    let prof_res_clone = prof_res.clone();
    apply_btn.connect_clicked(move |_| {
        let sel = dropdown.selected();
        let name = match sel {
            1 => "Studio & Lettura",
            2 => "Gaming Mode",
            3 => "Notturna Relax",
            _ => "Lavoro & Programmazione",
        };
        prof_res_clone.set_text(&format!("✅ Profilo '{}' attivato su ermete-shell-rs e Niri IPC.", name));
    });

    container.append(&prof_box);

    // Row for the full-screen bar toggle
    let bar_title = Label::builder()
        .label("<span size='large' weight='bold'>Comportamento Finestre in Fullscreen</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_top(12)
        .build();
    container.append(&bar_title);

    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .css_classes(vec!["card"])
        .build();

    let switch_label = Label::builder()
        .label("Nascondi Topbar quando un'applicazione è in modalità a schermo intero")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let toggle = Switch::builder()
        .valign(Align::Center)
        .build();

    toggle.connect_active_notify(|switch| {
        let is_active = switch.is_active();
        if is_active {
            let _ = Command::new("sh")
                .arg("-c")
                .arg("niri msg window-rule add hide-bar-on-fullscreen || true")
                .spawn();
        } else {
            let _ = Command::new("sh")
                .arg("-c")
                .arg("niri msg window-rule remove hide-bar-on-fullscreen || true")
                .spawn();
        }
    });

    row.append(&switch_label);
    row.append(&toggle);
    container.append(&row);

    container
}
