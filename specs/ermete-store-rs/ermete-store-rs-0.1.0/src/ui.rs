use std::rc::Rc;
use std::cell::Cell;
use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Button, HeaderBar, Label, ListBox, Orientation, ScrolledWindow, SearchEntry, Stack, StackSidebar};
use crate::backend;

fn create_app_row(app: &backend::AppInfo) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 12);
    row_box.set_margin_top(12);
    row_box.set_margin_bottom(12);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);

    let text_box = Box::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let title = Label::builder()
        .label(&app.name)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["heading"])
        .build();
    
    let subtitle = Label::builder()
        .label(&app.description)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label"])
        .wrap(true)
        .build();

    text_box.append(&title);
    text_box.append(&subtitle);

    let install_btn = Button::with_label("Installa");
    install_btn.set_valign(gtk4::Align::Center);
    
    let app_id = app.id.clone();
    install_btn.connect_clicked(glib::clone!(@weak install_btn => move |_| {
        let app_id_clone = app_id.clone();
        install_btn.set_sensitive(false);
        install_btn.set_label("Installando...");
        glib::spawn_future_local(glib::clone!(@weak install_btn => async move {
            match backend::install_app(&app_id_clone).await {
                Ok(_) => {
                    install_btn.set_label("Fatto");
                }
                Err(_) => {
                    install_btn.set_label("Errore");
                    install_btn.set_sensitive(true);
                }
            }
        }));
    }));

    row_box.append(&text_box);
    row_box.append(&install_btn);

    row_box
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Store")
        .default_width(900)
        .default_height(600)
        .build();

    let content_box = Box::new(Orientation::Vertical, 0);
    
    let header = HeaderBar::new();
    content_box.append(&header);

    let hbox = Box::new(Orientation::Horizontal, 0);
    let stack = Stack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_size_request(200, -1);
    
    hbox.append(&sidebar);
    hbox.append(&stack);
    content_box.append(&hbox);
    window.set_child(Some(&content_box));

    // Featured
    let featured_box = Box::new(Orientation::Vertical, 12);
    featured_box.set_margin_top(24);
    featured_box.set_margin_bottom(24);
    featured_box.set_margin_start(24);
    featured_box.set_margin_end(24);
    
    let featured_label = Label::builder()
        .label("In primo piano")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1"])
        .build();
    featured_box.append(&featured_label);
    
    let featured_list = ListBox::new();
    featured_list.set_selection_mode(gtk4::SelectionMode::None);
    featured_box.append(&featured_list);
    
    let featured_scroll = ScrolledWindow::new();
    featured_scroll.set_child(Some(&featured_box));
    stack.add_titled(&featured_scroll, Some("featured"), "Primo Piano");

    glib::spawn_future_local(glib::clone!(@weak featured_list => async move {
        match backend::get_featured_apps().await {
            Ok(apps) if !apps.is_empty() => {
                for app in apps {
                    let row = create_app_row(&app);
                    featured_list.append(&row);
                }
            }
            _ => {
                let error_label = Label::new(Some("Nessun risultato"));
                featured_list.append(&error_label);
            }
        }
    }));

    // Categories
    let categories_box = Box::new(Orientation::Vertical, 12);
    categories_box.set_margin_top(24);
    categories_box.set_margin_bottom(24);
    categories_box.set_margin_start(24);
    categories_box.set_margin_end(24);
    
    let categories_title = Label::builder()
        .label("Categorie")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1"])
        .build();
    categories_box.append(&categories_title);

    let categories_btn_box = Box::new(Orientation::Horizontal, 12);
    categories_box.append(&categories_btn_box);
    
    let categories_list = ListBox::new();
    categories_list.set_selection_mode(gtk4::SelectionMode::None);
    
    let categories_scroll = ScrolledWindow::new();
    categories_scroll.set_vexpand(true);
    categories_scroll.set_child(Some(&categories_list));
    categories_box.append(&categories_scroll);

    let cats = vec![
        ("Sviluppo", "development"),
        ("Giochi", "games"),
        ("Grafica", "graphics"),
        ("Ufficio", "office"),
    ];

    for (label_str, query_str) in cats {
        let btn = Button::with_label(label_str);
        let query = query_str.to_string();
        btn.connect_clicked(glib::clone!(@weak categories_list => move |_| {
            let q = query.clone();
            glib::spawn_future_local(glib::clone!(@weak categories_list => async move {
                while let Some(child) = categories_list.first_child() {
                    categories_list.remove(&child);
                }
                let loading = Label::new(Some("Caricamento..."));
                categories_list.append(&loading);
                
                if let Ok(apps) = backend::search_apps(&q).await {
                    categories_list.remove(&loading);
                    if apps.is_empty() {
                        categories_list.append(&Label::new(Some("Nessun risultato")));
                    } else {
                        for app in apps {
                            categories_list.append(&create_app_row(&app));
                        }
                    }
                } else {
                    categories_list.remove(&loading);
                    categories_list.append(&Label::new(Some("Errore di ricerca")));
                }
            }));
        }));
        categories_btn_box.append(&btn);
    }

    stack.add_titled(&categories_box, Some("categories_tab"), "Categorie");

    // Categories / Search
    let search_box = Box::new(Orientation::Vertical, 12);
    search_box.set_margin_top(24);
    search_box.set_margin_bottom(24);
    search_box.set_margin_start(24);
    search_box.set_margin_end(24);
    
    let search_entry = SearchEntry::new();
    search_box.append(&search_entry);
    
    let search_list = ListBox::new();
    search_list.set_selection_mode(gtk4::SelectionMode::None);
    let search_scroll = ScrolledWindow::new();
    search_scroll.set_child(Some(&search_list));
    search_scroll.set_vexpand(true);
    search_box.append(&search_scroll);
    
    stack.add_titled(&search_box, Some("categories"), "Cerca");

    let search_generation = Rc::new(Cell::new(0u32));
    search_entry.connect_search_changed(glib::clone!(@weak search_list => move |entry| {
        let query = entry.text().to_string();
        
        let current_gen = search_generation.get() + 1;
        search_generation.set(current_gen);
        let gen_clone = search_generation.clone();

        if query.len() < 3 { return; }
        
        glib::spawn_future_local(glib::clone!(@weak search_list => async move {
            if let Ok(apps) = backend::search_apps(&query).await {
                if gen_clone.get() != current_gen {
                    return;
                }
                
                while let Some(child) = search_list.first_child() {
                    search_list.remove(&child);
                }
                
                if apps.is_empty() {
                    search_list.append(&Label::new(Some("Nessun risultato")));
                } else {
                    for app in apps {
                        let row = create_app_row(&app);
                        search_list.append(&row);
                    }
                }
            }
        }));
    }));

    // Updates
    let updates_box = Box::new(Orientation::Vertical, 12);
    updates_box.set_margin_top(24);
    updates_box.set_margin_bottom(24);
    updates_box.set_margin_start(24);
    updates_box.set_margin_end(24);
    
    let update_title = Label::builder()
        .label("Aggiornamenti di Sistema")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1"])
        .build();
    updates_box.append(&update_title);
    
    let app_update_btn = Button::builder()
        .label("Aggiorna App (flatpak)")
        .css_classes(vec!["suggested-action"])
        .halign(gtk4::Align::Center)
        .build();

    let sys_update_btn = Button::builder()
        .label("Aggiorna Sistema (rpm-ostree)")
        .css_classes(vec!["suggested-action"])
        .halign(gtk4::Align::Center)
        .build();
        
    let update_status = Label::new(Some("Sistema pronto."));
    update_status.set_wrap(true);
    
    updates_box.append(&app_update_btn);
    updates_box.append(&sys_update_btn);
    updates_box.append(&update_status);
    
    app_update_btn.connect_clicked(glib::clone!(@weak app_update_btn, @weak update_status => move |_| {
        app_update_btn.set_sensitive(false);
        update_status.set_label("Aggiornamento app in corso...");
        
        glib::spawn_future_local(glib::clone!(@weak app_update_btn, @weak update_status => async move {
            match backend::update_apps().await {
                Ok(out) => update_status.set_label(&format!("Aggiornamento app completato:\n{}", out)),
                Err(e) => update_status.set_label(&format!("Errore durante l'aggiornamento app:\n{}", e)),
            }
            app_update_btn.set_sensitive(true);
        }));
    }));

    sys_update_btn.connect_clicked(glib::clone!(@weak sys_update_btn, @weak update_status => move |_| {
        sys_update_btn.set_sensitive(false);
        update_status.set_label("Aggiornamento sistema in corso. L'operazione potrebbe richiedere alcuni minuti...");
        
        glib::spawn_future_local(glib::clone!(@weak sys_update_btn, @weak update_status => async move {
            match backend::update_system().await {
                Ok(out) => update_status.set_label(&format!("Aggiornamento completato:\n{}", out)),
                Err(e) => update_status.set_label(&format!("Errore durante l'aggiornamento:\n{}", e)),
            }
            sys_update_btn.set_sensitive(true);
        }));
    }));
    
    let updates_scroll = ScrolledWindow::new();
    updates_scroll.set_child(Some(&updates_box));
    stack.add_titled(&updates_scroll, Some("updates"), "Aggiornamenti");

    window.present();
}
