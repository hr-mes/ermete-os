use gtk4::prelude::*;
use gtk4::{Align, Box, Label, ListBox, Orientation, Switch};
use crate::components::action_row::ActionRow;

pub fn build_page() -> Box {
    let container = Box::new(Orientation::Vertical, 16);
    container.set_margin_start(24);
    container.set_margin_end(24);
    container.set_margin_top(24);
    container.set_margin_bottom(24);

    let title = Label::new(Some("Notifiche"));
    title.add_css_class("title-1");
    title.set_halign(Align::Start);
    container.append(&title);

    // Do Not Disturb section
    let dnd_switch = Switch::new();
    dnd_switch.set_valign(Align::Center);
    dnd_switch.connect_active_notify(|switch| {
        let is_active = switch.is_active();
        relm4::spawn_local(async move {
            let mode = if is_active { "dnd" } else { "default" };
            let _ = tokio::process::Command::new("makoctl")
                .args(["mode", "-s", mode])
                .output()
                .await;
            crate::crdt_store::update_dnd_crdt(is_active).await;
        });
    });

    let dnd_row = ActionRow::builder("Non Disturbare")
        .subtitle("Silenzia tutte le notifiche di sistema ed avvisi popup (mako)")
        .suffix(&dnd_switch)
        .build();

    container.append(&dnd_row);

    // Apps section
    let apps_title = Label::new(Some("Applicazioni"));
    apps_title.add_css_class("heading");
    apps_title.set_halign(Align::Start);
    apps_title.set_margin_top(24);
    container.append(&apps_title);

    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);
    list_box.add_css_class("boxed-list");

    let apps = vec![
        ("Discord", "Notifiche per messaggi diretti e canali"),
        ("Firefox", "Notifiche dai siti web e completamento download"),
        ("Slack", "Notifiche per menzioni e messaggi del team"),
    ];

    for (app_name, app_desc) in apps {
        let app_switch = Switch::new();
        app_switch.set_active(true);
        app_switch.set_valign(Align::Center);

        let app_n = app_name.to_string();
        app_switch.connect_active_notify(move |sw| {
            let active = sw.is_active();
            let name_c = app_n.clone();
            relm4::spawn_local(async move {
                let key = format!("notification_app_{}", name_c.to_lowercase());
                let _ = crate::crdt_store::update_setting_crdt(&key, &active.to_string()).await;
            });
        });

        let row = ActionRow::builder(app_name)
            .subtitle(app_desc)
            .suffix(&app_switch)
            .build();

        list_box.append(&row);
    }

    container.append(&list_box);

    container
}
