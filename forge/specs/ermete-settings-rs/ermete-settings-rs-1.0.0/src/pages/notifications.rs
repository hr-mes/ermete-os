use gtk4::prelude::*;
use gtk4::{Align, Box, Label, ListBox, ListBoxRow, Orientation, Switch};
use std::process::Command;

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
    let dnd_box = Box::new(Orientation::Horizontal, 16);
    dnd_box.set_halign(Align::Fill);

    let dnd_label = Label::new(Some("Non Disturbare"));
    dnd_label.set_halign(Align::Start);
    dnd_label.set_hexpand(true);
    dnd_box.append(&dnd_label);

    let dnd_switch = Switch::new();
    dnd_switch.set_valign(Align::Center);
    dnd_switch.connect_active_notify(|switch| {
        if switch.is_active() {
            let _ = Command::new("makoctl").args(["mode", "-s", "dnd"]).spawn();
        } else {
            let _ = Command::new("makoctl").args(["mode", "-s", "default"]).spawn();
        }
    });
    dnd_box.append(&dnd_switch);
    container.append(&dnd_box);

    // Apps section
    let apps_title = Label::new(Some("Applicazioni"));
    apps_title.add_css_class("heading");
    apps_title.set_halign(Align::Start);
    apps_title.set_margin_top(24);
    container.append(&apps_title);

    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);
    list_box.add_css_class("boxed-list");

    let apps = vec!["Discord", "Firefox", "Slack"];
    for app in apps {
        let row = ListBoxRow::new();
        let row_box = Box::new(Orientation::Horizontal, 16);
        row_box.set_margin_start(16);
        row_box.set_margin_end(16);
        row_box.set_margin_top(12);
        row_box.set_margin_bottom(12);

        let app_label = Label::new(Some(app));
        app_label.set_halign(Align::Start);
        app_label.set_hexpand(true);
        row_box.append(&app_label);

        let app_switch = Switch::new();
        app_switch.set_active(true);
        app_switch.set_valign(Align::Center);
        row_box.append(&app_switch);

        row.set_child(Some(&row_box));
        list_box.append(&row);
    }

    container.append(&list_box);

    container
}
