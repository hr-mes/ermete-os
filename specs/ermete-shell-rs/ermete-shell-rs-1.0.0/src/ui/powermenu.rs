use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider,
    EventControllerKey, Label, Orientation,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::process::Command;

const POWERMENU_CSS: &str = r#"
window.powermenu-window {
    background-color: rgba(10, 10, 14, 0.75);
}

.powermenu-card {
    background: radial-gradient(circle, alpha(@surface_darker, 0.9), alpha(@surface_dim, 0.9));
    border-radius: 12px;
    box-shadow: inset 1px 2px 2px rgba(255, 255, 255, 0.2), 0 4px 12px rgba(0,0,0,0.5);
    margin: 10px;
    padding: 32px;
}

.powermenu-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    padding: 20px 24px;
    color: #e0def4;
    transition: all 0.2s ease;
}

.powermenu-btn:hover {
    background: rgba(235, 111, 146, 0.25);
    border-color: #eb6f92;
    color: #ffffff;
}

.powermenu-icon {
    font-size: 28px;
    margin-bottom: 8px;
}

.powermenu-label {
    font-size: 13px;
    font-weight: 600;
}
"#;

pub fn show_powermenu_modal(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Powermenu")
        .css_classes(["powermenu-window"])
        .build();

    window.init_layer_shell();
    window.set_namespace("powermenu");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);

    let provider = CssProvider::new();
    provider.load_from_data(POWERMENU_CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Display default"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let key_controller = EventControllerKey::new();
    let win_weak = window.downgrade();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            if let Some(w) = win_weak.upgrade() {
                w.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    let container = GtkBox::new(Orientation::Vertical, 24);
    container.set_valign(Align::Center);
    container.set_halign(Align::Center);
    container.add_css_class("powermenu-card");

    let title = Label::new(Some("Ermete OS Session Control"));
    title.set_css_classes(&["macos-app-title"]);
    container.append(&title);

    let btn_box = GtkBox::new(Orientation::Horizontal, 16);

    let actions = [
        ("", "Blocca", "loginctl lock-session"),
        ("󰍃", "Esci", "niri msg action quit"),
        ("󰒲", "Sospendi", "systemctl suspend"),
        ("󰜉", "Riavvia", "systemctl reboot"),
        ("", "Spegni", "systemctl poweroff"),
    ];

    for (icon, label, cmd) in actions {
        let btn = Button::new();
        btn.add_css_class("powermenu-btn");
        let box_inner = GtkBox::new(Orientation::Vertical, 4);
        let lbl_icon = Label::new(Some(icon));
        lbl_icon.add_css_class("powermenu-icon");
        let lbl_text = Label::new(Some(label));
        lbl_text.add_css_class("powermenu-label");
        box_inner.append(&lbl_icon);
        box_inner.append(&lbl_text);
        btn.set_child(Some(&box_inner));

        let cmd_str = cmd.to_string();
        let win_close = window.downgrade();
        btn.connect_clicked(move |_| {
            if let Some(w) = win_close.upgrade() {
                w.close();
            }
            if cmd_str == "niri msg action quit" {
                crate::core::niri_client::quit_niri();
            } else {
                let mut parts = cmd_str.split_whitespace();
                if let Some(prog) = parts.next() {
                    let _ = Command::new(prog).args(parts).spawn();
                }
            }
        });
        btn_box.append(&btn);
    }

    container.append(&btn_box);
    window.set_child(Some(&container));

    crate::ui::topbar::setup_popup_autoclose(&window, "powermenu");

    window.present();
}
