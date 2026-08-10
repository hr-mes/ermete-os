use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button,
    EventControllerKey, Label, Orientation,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};



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
        ("", "Blocca", "lock"),
        ("󰍃", "Esci", "quit"),
        ("󰒲", "Sospendi", "suspend"),
        ("󰜉", "Riavvia", "reboot"),
        ("", "Spegni", "poweroff"),
    ];

    for (icon, label, action) in actions {
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

        let action_str = action.to_string();
        let win_close = window.downgrade();
        btn.connect_clicked(move |_| {
            if let Some(w) = win_close.upgrade() {
                w.close();
            }
            let act = action_str.clone();
            glib::MainContext::default().spawn_local(async move {
                match act.as_str() {
                    "quit" => {
                        ermete_niri_ipc::async_client::quit_niri().await;
                    }
                    "lock" => {
                        if let Ok(conn) = zbus::Connection::system().await {
                            if let Ok(proxy) = crate::ipc::power::LogindProxy::new(&conn).await {
                                let _ = proxy.lock_sessions().await;
                            }
                        }
                    }
                    "suspend" => {
                        if let Ok(conn) = zbus::Connection::system().await {
                            if let Ok(proxy) = crate::ipc::power::LogindProxy::new(&conn).await {
                                let _ = proxy.suspend(true).await;
                            }
                        }
                    }
                    "reboot" => {
                        if let Ok(conn) = zbus::Connection::system().await {
                            if let Ok(proxy) = crate::ipc::power::LogindProxy::new(&conn).await {
                                let _ = proxy.reboot(true).await;
                            }
                        }
                    }
                    "poweroff" => {
                        if let Ok(conn) = zbus::Connection::system().await {
                            if let Ok(proxy) = crate::ipc::power::LogindProxy::new(&conn).await {
                                let _ = proxy.power_off(true).await;
                            }
                        }
                    }
                    _ => {}
                }
            });
        });
        btn_box.append(&btn);
    }

    container.append(&btn_box);
    window.set_child(Some(&container));

    crate::ui::popup_manager::setup_popup_autoclose(&window, "powermenu");

    window.present();
}
