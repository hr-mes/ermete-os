use crate::core::NOTIFICATIONS;
use crate::ui::popup_manager::setup_popup_autoclose;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Calendar, Label, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_calendar_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Calendar")
        .css_classes(["popup-window"])
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "calendar");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Right, 10);

    let main_vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let notifs_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["liquid-surface"])
        .build();
    
    let title_hbox = GtkBox::builder().orientation(Orientation::Horizontal).build();
    let notifs_title = Label::builder().label("Notifiche").css_classes(["cc-title"]).halign(Align::Start).hexpand(true).build();
    let clear_btn = Button::builder().label("Cancella").css_classes(["cc-btn"]).build();
    
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(300)
        .propagate_natural_height(true)
        .build();
    
    let list_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    
    let pop_clone_clear = pop.clone();
    clear_btn.connect_clicked(move |_| {
        NOTIFICATIONS.with(|n| n.borrow_mut().clear());
        pop_clone_clear.close();
    });
    
    title_hbox.append(&notifs_title);
    title_hbox.append(&clear_btn);
    notifs_card.append(&title_hbox);
    
    NOTIFICATIONS.with(|n| {
        let history = n.borrow();
        if history.is_empty() {
            list_box.append(&Label::builder().label("Nessuna nuova notifica").css_classes(["cc-label-sub"]).margin_top(10).margin_bottom(10).build());
        } else {
            for notif in history.iter() {
                let row = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).build();
                let sum = Label::builder().label(&notif.summary).halign(Align::Start).css_classes(["cc-label-main"]).build();
                let bod = Label::builder().label(&notif.body).halign(Align::Start).css_classes(["cc-label-sub"]).wrap(true).max_width_chars(30).build();
                row.append(&sum);
                row.append(&bod);
                list_box.append(&row);
            }
        }
    });

    scroll.set_child(Some(&list_box));
    notifs_card.append(&scroll);

    let cal_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["liquid-surface"])
        .build();

    let cal = Calendar::builder().build();
    cal_card.append(&cal);

    main_vbox.append(&notifs_card);
    main_vbox.append(&cal_card);

    pop.set_child(Some(&main_vbox));
    pop.present();
}
