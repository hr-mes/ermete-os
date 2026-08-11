use crate::core::*;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use tokio::sync::mpsc::UnboundedSender;

thread_local! {
    pub static ACTION_SENDER: RefCell<Option<UnboundedSender<(u32, String)>>> = const { RefCell::new(None) };
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
        .css_classes(["liquid-surface", "premium-notification", "toast-slide-in"])
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

#[allow(deprecated)]
pub fn spawn_notification_daemon(app: &Application) {
    

    load_notification_history();
    let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
    
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<(u32, String)>();
    ACTION_SENDER.with(|s| *s.borrow_mut() = Some(action_tx));

    glib::MainContext::default().spawn_local(async move {
        let server = NotificationServer {
            sender,
            counter: std::sync::atomic::AtomicU32::new(1),
        };

        let builder = match zbus::connection::Builder::session() {
            Ok(b) => b.max_queued(1024),
            Err(e) => {
                tracing::error!("Failed to get session bus: {}", e);
                return;
            }
        };

        let builder = match builder.name("org.freedesktop.Notifications") {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to request DBus name: {}", e);
                return;
            }
        };

        let builder = match builder.serve_at("/org/freedesktop/Notifications", server) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to serve DBus object: {}", e);
                return;
            }
        };

        let conn = match builder.build().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to build DBus connection: {}", e);
                return;
            }
        };
            
        while let Some((id, action_key)) = action_rx.recv().await {
            if let Err(e) = conn.emit_signal(
                None::<()>,
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "ActionInvoked",
                &(id, action_key),
            ).await {
                tracing::error!("Failed to emit ActionInvoked: {}", e);
            }
        }
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
        let focus_mode = crate::ipc::notifications::get_focus_mode();
        if focus_mode.should_allow_notification(&notif.app_name) {
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
        .css_classes(["liquid-surface"])
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
                    .css_classes(["liquid-surface"])
                    .build();

                let grp_header = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(8)
                    .build();

                let grp_title = Label::builder()
                    .label(format!("󰣆 {}", app_name))
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
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .build();

    let cur_focus = crate::ipc::notifications::get_focus_mode();

    let focus_status = Label::builder()
        .label(cur_focus.description())
        .css_classes(["cc-label-sub"])
        .halign(Align::Center)
        .build();

    let pills_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .halign(Align::Center)
        .build();

    let modes = [
        crate::ipc::notifications::FocusMode::Off,
        crate::ipc::notifications::FocusMode::Personal,
        crate::ipc::notifications::FocusMode::Work,
        crate::ipc::notifications::FocusMode::Sleep,
    ];

    let mut pill_btns = Vec::new();
    for m in modes {
        let btn = Button::builder()
            .label(format!("{} {}", m.icon(), m.name()))
            .css_classes(["cc-btn", "cc-quick-btn"])
            .build();
        if m == cur_focus {
            btn.add_css_class("cc-btn-active");
        }
        pill_btns.push((m, btn.clone()));
        pills_box.append(&btn);
    }

    let pill_btns_rc = std::rc::Rc::new(pill_btns);
    for (m, btn) in pill_btns_rc.iter() {
        let m_val = *m;
        let stat_clone = focus_status.clone();
        let btns_clone = pill_btns_rc.clone();
        btn.connect_clicked(move |_| {
            for (item_m, item_b) in btns_clone.iter() {
                if *item_m == m_val {
                    item_b.add_css_class("cc-btn-active");
                } else {
                    item_b.remove_css_class("cc-btn-active");
                }
            }
            stat_clone.set_text(m_val.description());
            crate::ipc::notifications::set_focus_mode(m_val);
        });
    }

    footer_card.append(&focus_status);
    footer_card.append(&pills_box);
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
