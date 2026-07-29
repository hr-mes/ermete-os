use crate::ui::spotlight::populate_launcher_list;
use crate::ui::topbar::setup_popup_autoclose;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_start_menu_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Start Menu")
        .css_classes(["popup-window"])
        .default_width(560)
        .default_height(480)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "launcher");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Left, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Left, 8);

    let main_hbox = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .css_classes(["cc-card"])
        .build();

    let sidebar = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    sidebar.set_margin_top(14);
    sidebar.set_margin_bottom(14);
    sidebar.set_margin_start(14);
    sidebar.set_margin_end(14);

    let cats_lbl = Label::builder().label("CATEGORIE").css_classes(["cc-label-sub"]).halign(Align::Start).margin_bottom(6).build();
    sidebar.append(&cats_lbl);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    let search = Entry::builder()
        .placeholder_text("Cerca nel menu...")
        .css_classes(["spotlight-input"])
        .build();

    let current_category = std::rc::Rc::new(std::cell::RefCell::new("Tutte".to_string()));
    let cats = ["Tutte", "Internet", "Ufficio", "Grafica", "Multimedia", "Sviluppo", "Sistema", "Giochi"];
    
    for cat in cats {
        let btn = Button::builder().label(cat).css_classes(["spotlight-item"]).halign(Align::Fill).build();
        let cat_str = cat.to_string();
        let list_clone = list_box.clone();
        let entry_clone = search.clone();
        let pop_clone = pop.clone();
        let curr_cat = current_category.clone();
        btn.connect_clicked(move |_| {
            *curr_cat.borrow_mut() = cat_str.clone();
            populate_launcher_list(&list_clone, &entry_clone.text(), &cat_str, false, &pop_clone);
        });
        sidebar.append(&btn);
    }

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();
    card.set_margin_top(14);
    card.set_margin_bottom(14);
    card.set_margin_end(14);

    let title = Label::builder()
        .label("◈  MENU APPLICAZIONI ERMETE OS")
        .css_classes(["cc-title"])
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(310)
        .build();

    populate_launcher_list(&list_box, "", "Tutte", false, &pop);

    let list_clone2 = list_box.clone();
    let pop_clone2 = pop.clone();
    let curr_cat2 = current_category.clone();
    search.connect_changed(move |e| {
        populate_launcher_list(&list_clone2, &e.text(), &curr_cat2.borrow(), false, &pop_clone2);
    });

    scroll.set_child(Some(&list_box));

    let footer = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    let off_btn = Button::builder()
        .label("⏻  Spegni")
        .css_classes(["cc-btn-danger"])
        .hexpand(true)
        .build();
    off_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.power_off().await;
        });
    });

    let reb_btn = Button::builder()
        .label("↻  Riavvia")
        .css_classes(["cc-btn"])
        .hexpand(true)
        .build();
    reb_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.reboot().await;
        });
    });

    let susp_btn = Button::builder()
        .label("💤  Sospendi")
        .css_classes(["cc-btn"])
        .hexpand(true)
        .build();
    susp_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::system_proxies::get_global_controller();
            let _ = ctrl.suspend().await;
        });
    });

    footer.append(&off_btn);
    footer.append(&reb_btn);
    footer.append(&susp_btn);

    card.append(&title);
    card.append(&search);
    card.append(&scroll);
    card.append(&footer);

    main_hbox.append(&sidebar);
    
    let sep = gtk4::Separator::new(Orientation::Vertical);
    sep.set_margin_start(4);
    sep.set_margin_end(10);
    main_hbox.append(&sep);
    
    main_hbox.append(&card);

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);

    pop.set_child(Some(&main_hbox));
    pop.present();
    search.grab_focus();
}
