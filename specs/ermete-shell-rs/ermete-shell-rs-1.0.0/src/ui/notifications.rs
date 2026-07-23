use crate::core::*;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use tokio::sync::mpsc::UnboundedSender;

thread_local! {
    pub static ACTION_SENDER: RefCell<Option<UnboundedSender<(u32, String)>>> = RefCell::new(None);
}

pub fn format_action_invoked_payload(id: u32, text: &str) -> (u32, String) {
    (id, text.to_string())
}

pub fn send_action_invoked(id: u32, action_key: &str) {
    ACTION_SENDER.with(|sender| {
        if let Some(tx) = sender.borrow().as_ref() {
            let _ = tx.send(format_action_invoked_payload(id, action_key));
        }
    });
}

pub fn show_toast_popup(app: &Application, notif: &NotificationData) {
    let toast = ApplicationWindow::builder()
        .application(app)
        .css_classes(["transparent-window"])
        .build();

    toast.init_layer_shell();
    toast.set_namespace("notifications");
    toast.set_layer(Layer::Overlay);
    toast.set_anchor(Edge::Top, true);
    toast.set_anchor(Edge::Right, true);
    toast.set_margin(Edge::Top, 40);
    toast.set_margin(Edge::Right, 10);

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(["cc-card", "premium-notification", "toast-slide-in"])
        .build();
    
    let title = Label::builder().label(&notif.summary).css_classes(["cc-title"]).halign(Align::Start).build();
    let body = Label::builder().label(&notif.body).css_classes(["cc-label-sub"]).halign(Align::Start).wrap(true).max_width_chars(30).build();
    
    vbox.append(&title);
    vbox.append(&body);

    if !notif.actions.is_empty() {
        let act_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).build();
        for (ak, al) in &notif.actions {
            if ak != "inline-reply" && !ak.contains("reply") {
                let btn = Button::builder().label(al).css_classes(["cc-btn"]).build();
                let toast_clone = toast.clone();
                let id = notif.id;
                let key_clone = ak.clone();
                btn.connect_clicked(move |_| {
                    crate::ui::notifications::send_action_invoked(id, &key_clone);
                    toast_clone.close();
                });
                act_box.append(&btn);
            }
        }
        vbox.append(&act_box);
    }

    if notif.has_inline_reply {
        let reply_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).margin_top(4).build();
        let entry = Entry::builder().placeholder_text("Rispondi rapidamente...").hexpand(true).build();
        let send_btn = Button::builder().label("󰇀").css_classes(["cc-quick-btn"]).build();
        
        let entry_clone = entry.clone();
        let toast_clone = toast.clone();
        let id = notif.id;
        let send_action = std::rc::Rc::new(move || {
            let text = entry_clone.text().to_string();
            if !text.is_empty() {
                crate::ui::notifications::send_action_invoked(id, &text);
                toast_clone.close();
            }
        });
        let act_btn = send_action.clone();
        send_btn.connect_clicked(move |_| act_btn());
        let act_entry = send_action.clone();
        entry.connect_activate(move |_| act_entry());

        reply_box.append(&entry);
        reply_box.append(&send_btn);
        vbox.append(&reply_box);
    }

    toast.set_child(Some(&vbox));
    toast.present();

    let has_reply = notif.has_inline_reply;
    let slide_out_ms = if has_reply { 11600 } else { 4600 };
    let vbox_clone = vbox.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(slide_out_ms), move || {
        vbox_clone.add_css_class("toast-slide-out");
        glib::ControlFlow::Break
    });

    let close_ms = if has_reply { 12000 } else { 5000 };
    glib::timeout_add_local(std::time::Duration::from_millis(close_ms), clone!(@weak toast => @default-return glib::ControlFlow::Break, move || {
        toast.close();
        glib::ControlFlow::Break
    }));
}

pub fn spawn_notification_daemon(app: &Application) {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(r#"
        .premium-notification {
            background: radial-gradient(circle, alpha(@surface_darker, 0.9), alpha(@surface_dim, 0.9));
            border-radius: 12px;
            box-shadow: inset 1px 2px 2px rgba(255, 255, 255, 0.2), 0 4px 12px rgba(0,0,0,0.5);
            margin: 10px;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        .toast-slide-in {
            animation: slide-in-toast 0.4s cubic-bezier(0.25, 1, 0.5, 1);
        }
        .toast-slide-out {
            animation: slide-out-toast 0.4s cubic-bezier(0.25, 1, 0.5, 1) forwards;
        }
        @keyframes slide-in-toast {
            0% { transform: translateX(100%); opacity: 0; }
            100% { transform: translateX(0); opacity: 1; }
        }
        @keyframes slide-out-toast {
            0% { transform: translateX(0); opacity: 1; }
            100% { transform: translateX(100%); opacity: 0; }
        }
        
        .notification-center-window {
            background-color: rgba(30, 30, 30, 0.65);
            border-radius: 20px;
            box-shadow: -5px 0 30px rgba(0, 0, 0, 0.5);
            border-left: 1px solid rgba(255, 255, 255, 0.1);
            animation: slide-in-nc 0.4s cubic-bezier(0.25, 1, 0.5, 1);
        }
        @keyframes slide-in-nc {
            0% { transform: translateX(100%); opacity: 0; }
            100% { transform: translateX(0); opacity: 1; }
        }
        .transparent-window {
            background-color: transparent;
        }
    "#);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    load_notification_history();
    let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
    
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<(u32, String)>();
    ACTION_SENDER.with(|s| *s.borrow_mut() = Some(action_tx));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = NotificationServer {
                sender,
                counter: std::sync::atomic::AtomicU32::new(1),
            };
            let conn = zbus::connection::Builder::session()
                .unwrap()
                .name("org.freedesktop.Notifications")
                .unwrap()
                .serve_at("/org/freedesktop/Notifications", server)
                .unwrap()
                .build()
                .await
                .unwrap();
                
            while let Some((id, action_key)) = action_rx.recv().await {
                if let Err(e) = conn.emit_signal(
                    None::<()>,
                    "/org/freedesktop/Notifications",
                    "org.freedesktop.Notifications",
                    "ActionInvoked",
                    &(id, action_key),
                ).await {
                    eprintln!("Failed to emit ActionInvoked: {}", e);
                }
            }
        });
    });

    let app_clone = app.clone();
    receiver.attach(None, move |notif| {
        NOTIFICATIONS.with(|n| {
            let mut list = n.borrow_mut();
            if let Some(pos) = list.iter().position(|x| x.id == notif.id) {
                list[pos] = notif.clone();
            } else {
                list.insert(0, notif.clone());
            }
        });
        save_notification_history();
        if !crate::core::DND_ACTIVE.load(std::sync::atomic::Ordering::SeqCst) {
            show_toast_popup(&app_clone, &notif);
        }
        glib::ControlFlow::Continue
    });
}

pub fn show_notification_center(app: &Application) {
    let sidebar = ApplicationWindow::builder()
        .application(app)
        .css_classes(["notification-center-window"])
        .build();

    sidebar.init_layer_shell();
    sidebar.set_namespace("notifications");
    sidebar.set_layer(Layer::Top);
    sidebar.set_anchor(Edge::Top, true);
    sidebar.set_anchor(Edge::Bottom, true);
    sidebar.set_anchor(Edge::Right, true);
    sidebar.set_width_request(380);

    let main_vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["cc-card"])
        .margin_top(16)
        .margin_bottom(16)
        .margin_end(16)
        .margin_start(16)
        .hexpand(true)
        .vexpand(true)
        .build();

    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let title = Label::builder()
        .label("󰂚 Centro Notifiche")
        .css_classes(["cc-title"])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let clear_all_btn = Button::builder()
        .label("Cancella tutto")
        .css_classes(["cc-btn"])
        .build();

    let sidebar_clone = sidebar.clone();
    clear_all_btn.connect_clicked(move |_| {
        NOTIFICATIONS.with(|n| n.borrow_mut().clear());
        save_notification_history();
        sidebar_clone.close();
    });

    header_box.append(&title);
    header_box.append(&clear_all_btn);
    main_vbox.append(&header_box);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .build();

    NOTIFICATIONS.with(|n| {
        let history = n.borrow();
        if history.is_empty() {
            let empty_lbl = Label::builder()
                .label("Nessuna notifica nello storico")
                .css_classes(["cc-label-sub"])
                .margin_top(40)
                .build();
            list_box.append(&empty_lbl);
        } else {
            let mut groups: std::collections::HashMap<String, Vec<NotificationData>> = std::collections::HashMap::new();
            for notif in history.iter() {
                groups.entry(notif.app_name.clone()).or_default().push(notif.clone());
            }

            for (app_name, items) in groups.iter() {
                let group_card = GtkBox::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(8)
                    .css_classes(["cc-card"])
                    .build();

                let grp_header = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(8)
                    .build();

                let grp_title = Label::builder()
                    .label(&format!("󰣆 {}", app_name))
                    .css_classes(["cc-label-main"])
                    .halign(Align::Start)
                    .hexpand(true)
                    .build();

                let dismiss_grp_btn = Button::builder()
                    .label("󰅖")
                    .css_classes(["greeter-icon-btn"])
                    .build();

                let app_name_clone = app_name.clone();
                let sb_clone = sidebar.clone();
                dismiss_grp_btn.connect_clicked(move |_| {
                    NOTIFICATIONS.with(|n| {
                        n.borrow_mut().retain(|x| x.app_name != app_name_clone);
                    });
                    save_notification_history();
                    sb_clone.close();
                });

                grp_header.append(&grp_title);
                grp_header.append(&dismiss_grp_btn);
                group_card.append(&grp_header);

                for item in items.iter() {
                    let item_box = GtkBox::builder()
                        .orientation(Orientation::Vertical)
                        .spacing(4)
                        .margin_start(12)
                        .build();

                    let sum_hdr = GtkBox::builder()
                        .orientation(Orientation::Horizontal)
                        .spacing(8)
                        .build();

                    let sum_lbl = Label::builder()
                        .label(&item.summary)
                        .css_classes(["cc-label-main"])
                        .halign(Align::Start)
                        .hexpand(true)
                        .build();

                    let time_lbl = Label::builder()
                        .label(&item.timestamp)
                        .css_classes(["cc-label-sub"])
                        .build();

                    sum_hdr.append(&sum_lbl);
                    sum_hdr.append(&time_lbl);

                    let body_lbl = Label::builder()
                        .label(&item.body)
                        .css_classes(["cc-label-sub"])
                        .halign(Align::Start)
                        .wrap(true)
                        .build();

                    item_box.append(&sum_hdr);
                    item_box.append(&body_lbl);

                    if !item.actions.is_empty() {
                        let act_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).margin_top(4).build();
                        for (ak, al) in &item.actions {
                            if ak != "inline-reply" && !ak.contains("reply") {
                                let btn = Button::builder().label(al).css_classes(["cc-btn"]).build();
                                let sb_close = sidebar.clone();
                                let id = item.id;
                                let key_clone = ak.clone();
                                btn.connect_clicked(move |_| {
                                    crate::ui::notifications::send_action_invoked(id, &key_clone);
                                    sb_close.close();
                                });
                                act_box.append(&btn);
                            }
                        }
                        item_box.append(&act_box);
                    }

                    if item.has_inline_reply {
                        let reply_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(6).margin_top(4).build();
                        let entry = Entry::builder().placeholder_text("Rispondi...").hexpand(true).build();
                        let send_btn = Button::builder().label("󰇀").css_classes(["cc-quick-btn"]).build();
                        let entry_clone = entry.clone();
                        let id = item.id;
                        let sb_close = sidebar.clone();
                        let send_action = std::rc::Rc::new(move || {
                            let text = entry_clone.text().to_string();
                            if !text.is_empty() {
                                crate::ui::notifications::send_action_invoked(id, &text);
                                sb_close.close();
                            }
                        });
                        let act_btn = send_action.clone();
                        send_btn.connect_clicked(move |_| act_btn());
                        let act_entry = send_action.clone();
                        entry.connect_activate(move |_| act_entry());
                        reply_box.append(&entry);
                        reply_box.append(&send_btn);
                        item_box.append(&reply_box);
                    }

                    group_card.append(&item_box);
                }

                list_box.append(&group_card);
            }
        }
    });

    scroll.set_child(Some(&list_box));
    main_vbox.append(&scroll);

    let footer_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .build();

    let is_dnd = crate::core::DND_ACTIVE.load(std::sync::atomic::Ordering::SeqCst);
    let dnd_btn = Button::builder()
        .label(if is_dnd { "󰂛 Non Disturbare: ATTIVO" } else { "󰂛 Non Disturbare: OFF" })
        .css_classes(["cc-btn"])
        .build();

    let dnd_status = Label::builder()
        .label(if is_dnd { "Notifiche popup bloccate" } else { "Notifiche popup attive" })
        .css_classes(["cc-label-sub"])
        .build();

    let dnd_btn_clone = dnd_btn.clone();
    let dnd_stat_clone = dnd_status.clone();
    dnd_btn.connect_clicked(move |_| {
        let curr = crate::core::DND_ACTIVE.load(std::sync::atomic::Ordering::SeqCst);
        let next = !curr;
        crate::core::DND_ACTIVE.store(next, std::sync::atomic::Ordering::SeqCst);
        dnd_btn_clone.set_label(if next { "󰂛 Non Disturbare: ATTIVO" } else { "󰂛 Non Disturbare: OFF" });
        dnd_stat_clone.set_text(if next { "Notifiche popup bloccate" } else { "Notifiche popup attive" });
    });

    footer_card.append(&dnd_btn);
    footer_card.append(&dnd_status);
    main_vbox.append(&footer_card);

    sidebar.set_child(Some(&main_vbox));
    sidebar.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_reply_payload_structure() {
        let id = 123;
        let text = "Hello Ermete!";
        
        let payload = format_action_invoked_payload(id, text);
        
        assert_eq!(payload.0, 123);
        assert_eq!(payload.1, "Hello Ermete!");
    }
}
