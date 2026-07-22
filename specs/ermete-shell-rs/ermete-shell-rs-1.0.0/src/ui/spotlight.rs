use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Image, Label, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::ui::topbar::setup_popup_autoclose;
use gtk4::gio::AppInfo;
use std::cell::RefCell;
use std::time::Instant;

#[derive(Clone, Debug)]
pub enum SpotlightAction {
    LaunchApp(AppInfo),
    OpenSettingsPage(String),
}

#[derive(Clone, Debug)]
pub struct SpotlightItem {
    pub title: String,
    pub description: String,
    pub icon_name: String,
    pub keywords: String,
    pub exec_action: SpotlightAction,
}

thread_local! {
    static SPOTLIGHT_INDEX: RefCell<Vec<SpotlightItem>> = RefCell::new(Vec::new());
    static LAST_INDEX_TIME: RefCell<Option<Instant>> = RefCell::new(None);
}

pub fn ensure_index_loaded() {
    let should_refresh = LAST_INDEX_TIME.with(|t| {
        match *t.borrow() {
            Some(time) => time.elapsed().as_secs() > 60,
            None => true,
        }
    });

    if !should_refresh {
        return;
    }

    let mut items = Vec::new();

    // 1. Index every page and feature of ermete-settings-rs (Settings Deeplinks)
    let deeplinks = [
        ("Schermi e Monitor", "Configurazione risoluzione, frequenza di aggiornamento e scala frazionaria", "preferences-desktop-display", "schermi monitor display risoluzione frazionaria scala hz hdr refresh niri vrr displays", "displays"),
        ("Aspetto e Colori", "Sfondo del desktop, colori accento Hex e modalità Scura/Chiara", "preferences-desktop-theme", "aspetto colori tema dark light scuro chiaro sfondo wallpaper matugen swww accent appearance", "appearance"),
        ("Audio e Suono", "Dispositivi di output/input audio, volume PipeWire e impostazioni microfono", "audio-card", "audio suono volume speaker microfono cuffie pipewire pulse get_volume mute", "audio"),
        ("Tastiera e Input", "Layout tastiera XKB, delay/rate ripetizione e rimappature", "input-keyboard", "tastiera keyboard input xkb layout ripetizione delay rate", "keyboard"),
        ("Mouse e Touchpad", "Velocità puntatore, accelerazione piatta o adattiva e gesture libinput", "input-mouse", "mouse touchpad puntatore libinput accelerazione adaptive flat gesture natural scroll", "mouse"),
        ("Reti Wi-Fi e Ethernet", "Gestione connessioni wireless e di rete NetworkManager", "network-wireless", "wifi wi-fi rete network ethernet ip dns router nmcli connessione", "network"),
        ("Bluetooth", "Accoppiamento auricolari, mouse e dispositivi BlueZ", "bluetooth", "bluetooth bluez auricolari cuffie mouse accoppiamento pairing", "bluetooth"),
        ("Gestione Finestre e Focus", "Comportamento focus-follows-mouse, bordi e shortcut Niri", "preferences-system-windows", "finestre focus niri workspace gap bordi angoli trasparenza", "focus"),
        ("Dock e Topbar", "Comportamento della dock a molla e indicatori di sistema", "utilities-terminal", "dock topbar barra menu launcher auto hide spring fisica desktop", "desktop"),
    ];

    for (title, desc, icon, kw, page) in deeplinks {
        items.push(SpotlightItem {
            title: format!("Impostazioni: {}", title),
            description: desc.to_string(),
            icon_name: icon.to_string(),
            keywords: kw.to_lowercase(),
            exec_action: SpotlightAction::OpenSettingsPage(page.to_string()),
        });
    }

    // 2. Index installed AppInfo catalog
    let mut apps: Vec<AppInfo> = AppInfo::all().into_iter().filter(|a| a.should_show()).collect();
    apps.sort_by(|a, b| a.display_name().to_lowercase().cmp(&b.display_name().to_lowercase()));

    for app_info in apps {
        let name = app_info.display_name().to_string();
        let desc = app_info.description().map(|s| s.to_string()).unwrap_or_default();
        let mut app_cats = String::new();
        if let Some(desktop_app) = app_info.downcast_ref::<gtk4::gio::DesktopAppInfo>() {
            if let Some(cats) = desktop_app.categories() {
                app_cats = cats.to_string().to_lowercase();
            }
        }
        let kw = format!("{} {} {}", name.to_lowercase(), desc.to_lowercase(), app_cats);
        let icon_name = match app_info.icon() {
            Some(gicon) => gtk4::prelude::IconExt::to_string(&gicon).map(|s| s.to_string()).unwrap_or_else(|| "application-x-executable".to_string()),
            None => "application-x-executable".to_string(),
        };

        items.push(SpotlightItem {
            title: name,
            description: desc,
            icon_name,
            keywords: kw,
            exec_action: SpotlightAction::LaunchApp(app_info),
        });
    }

    SPOTLIGHT_INDEX.with(|idx| {
        *idx.borrow_mut() = items;
    });
    LAST_INDEX_TIME.with(|t| {
        *t.borrow_mut() = Some(Instant::now());
    });
}

pub fn populate_launcher_list(list_box: &GtkBox, filter_text: &str, category_filter: &str, is_spotlight: bool, pop: &ApplicationWindow) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    let filter_lower = filter_text.to_lowercase();

    if is_spotlight && filter_lower.starts_with('=') {
        let expr = filter_lower.trim_start_matches('=').trim();
        if let Ok(res) = meval::eval_str(expr) {
            let res_str = res.to_string();
            let row = Button::builder().css_classes(["spotlight-item"]).build();
            let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
            let img = Image::builder().icon_name("accessories-calculator").pixel_size(40).build();
            hbox.append(&img);
            let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
            let name_lbl = Label::builder().label(&format!("= {}", res_str)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
            vbox.append(&name_lbl);
            let desc_lbl = Label::builder().label("Risultato calcolatrice (clicca per copiare e chiudere)").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
            vbox.append(&desc_lbl);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            let pop_clone = pop.clone();
            row.connect_clicked(move |_| {
                let clipboard = pop_clone.clipboard();
                clipboard.set_text(&res_str);
                pop_clone.close();
            });
            list_box.append(&row);
        }
        return;
    }

    if is_spotlight && filter_lower.starts_with('>') {
        let cmd = filter_text.trim_start_matches('>').trim();
        let row = Button::builder().css_classes(["spotlight-item"]).build();
        let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
        let img = Image::builder().icon_name("utilities-terminal").pixel_size(40).build();
        hbox.append(&img);
        let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
        let name_lbl = Label::builder().label(&format!("Esegui: {}", cmd)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
        vbox.append(&name_lbl);
        let desc_lbl = Label::builder().label("Lancia comando nel terminale").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
        vbox.append(&desc_lbl);
        hbox.append(&vbox);
        row.set_child(Some(&hbox));
        let pop_clone = pop.clone();
        let cmd_clone = cmd.to_string();
        row.connect_clicked(move |_| {
            let _ = std::process::Command::new("foot").arg("-e").arg("sh").arg("-c").arg(&format!("{}; read -p '\nPremi Invio per chiudere...'", cmd_clone)).spawn();
            pop_clone.close();
        });
        list_box.append(&row);
        return;
    }

    if is_spotlight && filter_lower.starts_with('?') {
        let query = filter_text.trim_start_matches('?').trim();
        if !query.is_empty() {
            let row = Button::builder().css_classes(["spotlight-item"]).build();
            let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
            let img = Image::builder().icon_name("web-browser").pixel_size(40).build();
            hbox.append(&img);
            let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
            let name_lbl = Label::builder().label(&format!("Cerca sul Web: {}", query)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
            vbox.append(&name_lbl);
            let desc_lbl = Label::builder().label("Cerca con il browser predefinito").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
            vbox.append(&desc_lbl);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            let pop_clone = pop.clone();
            let query_encoded = query.replace(" ", "+");
            let search_url = format!("https://duckduckgo.com/?q={}", query_encoded);
            row.connect_clicked(move |_| {
                let _ = std::process::Command::new("xdg-open").arg(&search_url).spawn();
                pop_clone.close();
            });
            list_box.append(&row);
        }
        return;
    }

    if is_spotlight && filter_lower.starts_with('/') {
        let query = filter_text.trim_start_matches('/').trim();
        if !query.is_empty() {
            if let Ok(output) = std::process::Command::new("plocate")
                .arg("-l")
                .arg("5")
                .arg(&query)
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut count = 0;
                for line in stdout.lines() {
                    if line.trim().is_empty() { continue; }
                    if count >= 5 { break; }
                    let row = Button::builder().css_classes(["spotlight-item"]).build();
                    let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
                    let img = Image::builder().icon_name("text-x-generic").pixel_size(40).build();
                    hbox.append(&img);
                    let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
                    let path = std::path::Path::new(line);
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let name_lbl = Label::builder().label(&name.to_string()).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
                    vbox.append(&name_lbl);
                    let desc_lbl = Label::builder().label(line).halign(Align::Start).css_classes(["spotlight-item-desc"]).ellipsize(gtk4::pango::EllipsizeMode::Middle).build();
                    vbox.append(&desc_lbl);
                    hbox.append(&vbox);
                    row.set_child(Some(&hbox));
                    let pop_clone = pop.clone();
                    let file_path = line.to_string();
                    row.connect_clicked(move |_| {
                        let _ = std::process::Command::new("xdg-open").arg(&file_path).spawn();
                        pop_clone.close();
                    });
                    list_box.append(&row);
                    count += 1;
                }
                if count == 0 {
                    let no_res = Label::builder().label("Nessun file trovato.").css_classes(["cc-label-sub"]).margin_top(20).build();
                    list_box.append(&no_res);
                }
            }
        }
        return;
    }

    ensure_index_loaded();

    let mut count = 0;
    SPOTLIGHT_INDEX.with(|idx| {
        for item in idx.borrow().iter() {
            if category_filter != "Tutte" && !category_filter.is_empty() {
                let match_found = match category_filter {
                    "Internet" => item.keywords.contains("network") || item.keywords.contains("webbrowser"),
                    "Ufficio" => item.keywords.contains("office") || item.keywords.contains("wordprocessor"),
                    "Grafica" => item.keywords.contains("graphics") || item.keywords.contains("photography"),
                    "Multimedia" => item.keywords.contains("audiovideo") || item.keywords.contains("audio") || item.keywords.contains("video"),
                    "Sviluppo" => item.keywords.contains("development"),
                    "Sistema" => item.keywords.contains("system") || item.keywords.contains("utility") || item.keywords.contains("settings"),
                    "Giochi" => item.keywords.contains("game"),
                    _ => false,
                };
                if !match_found { continue; }
            }

            if !filter_lower.is_empty() && !item.title.to_lowercase().contains(&filter_lower) && !item.description.to_lowercase().contains(&filter_lower) && !item.keywords.contains(&filter_lower) {
                continue;
            }

            let row = Button::builder().css_classes(["spotlight-item"]).build();
            let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
            let img = Image::from_icon_name(&item.icon_name);
            img.set_pixel_size(40);
            hbox.append(&img);

            let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
            let name_lbl = Label::builder().label(&item.title).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
            vbox.append(&name_lbl);
            if !item.description.is_empty() {
                let desc_lbl = Label::builder().label(&item.description).halign(Align::Start).css_classes(["spotlight-item-desc"]).ellipsize(gtk4::pango::EllipsizeMode::End).build();
                vbox.append(&desc_lbl);
            }
            hbox.append(&vbox);
            row.set_child(Some(&hbox));

            let action_clone = item.exec_action.clone();
            let pop_clone = pop.clone();
            row.connect_clicked(move |_| {
                match &action_clone {
                    SpotlightAction::LaunchApp(app_info) => {
                        let _ = app_info.launch(&[], gtk4::gio::AppLaunchContext::NONE);
                    }
                    SpotlightAction::OpenSettingsPage(page) => {
                        let _ = std::process::Command::new("ermete-settings-rs").arg("--page").arg(page).spawn();
                    }
                }
                pop_clone.close();
            });
            list_box.append(&row);
            count += 1;

            if count >= 20 {
                break; // Cap UI generation for <1ms response time
            }
        }
    });

    if count == 0 {
        let no_res = Label::builder().label("Nessun risultato trovato.").css_classes(["cc-label-sub"]).margin_top(20).build();
        list_box.append(&no_res);
    }
}

pub fn show_spotlight_modal(app: &Application) {
    ensure_index_loaded();

    // Inietta lo stile Premium Seelen UI (Glassmorphism e Typography)
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        window.spotlight-window {
            background-color: transparent;
        }
        .spotlight-card {
            background-color: alpha(#1e1e20, 0.90);
            border-radius: 16px;
            border: 1px solid alpha(white, 0.15);
            box-shadow: 0 16px 48px alpha(black, 0.6);
            padding: 12px;
        }
        .spotlight-input {
            font-size: 26px;
            font-weight: 500;
            background: transparent;
            border: none;
            box-shadow: none;
            color: white;
            padding: 12px;
            caret-color: white;
        }
        .spotlight-input:focus {
            outline: none;
            box-shadow: none;
        }
        .spotlight-item {
            background: transparent;
            border: none;
            border-radius: 12px;
            padding: 12px;
            transition: all 0.2s ease-in-out;
        }
        .spotlight-item:hover, .spotlight-item:focus {
            background-color: rgba(255, 255, 255, 0.15);
        }
        .spotlight-item-title {
            font-size: 18px;
            font-weight: 600;
            color: white;
        }
        .spotlight-item-desc {
            font-size: 14px;
            font-weight: 400;
            color: rgba(255, 255, 255, 0.6);
        }
        "
    );
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Spotlight")
        .css_classes(["spotlight-window"])
        .default_width(620)
        .default_height(420)
        .build();

    pop.init_layer_shell();
    pop.set_namespace("spotlight");
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "spotlight");
    pop.set_margin(Edge::Top, 140);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .css_classes(["spotlight-card"])
        .build();

    let entry = Entry::builder()
        .placeholder_text("Cerca applicazioni, impostazioni (=, >, /)...")
        .css_classes(["spotlight-input"])
        .hexpand(true)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(300)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    populate_launcher_list(&list_box, "", "", true, &pop);

    let list_clone = list_box.clone();
    let pop_clone2 = pop.clone();
    entry.connect_changed(move |e| {
        populate_launcher_list(&list_clone, &e.text(), "", true, &pop_clone2);
    });

    // Navigazione tastiera (Freccia Giù) per saltare ai risultati
    let key_controller = gtk4::EventControllerKey::new();
    let list_focus_clone = list_box.clone();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Down {
            if let Some(first) = list_focus_clone.first_child() {
                first.grab_focus();
                return glib::Propagation::Stop;
            }
        }
        glib::Propagation::Proceed
    });
    entry.add_controller(key_controller);

    scroll.set_child(Some(&list_box));
    card.append(&entry);
    card.append(&scroll);

    pop.set_child(Some(&card));
    pop.present();
    entry.grab_focus();
}
