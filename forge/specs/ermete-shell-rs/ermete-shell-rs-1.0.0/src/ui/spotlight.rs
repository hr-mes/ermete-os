use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Image, Label, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::ui::popup_manager::setup_popup_autoclose;
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
    static SPOTLIGHT_INDEX: RefCell<Vec<SpotlightItem>> = const { RefCell::new(Vec::new()) };
    static LAST_INDEX_TIME: RefCell<Option<Instant>> = const { RefCell::new(None) };
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
    apps.sort_by_key(|a| a.display_name().to_lowercase());

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

fn try_parse_conversions(filter_lower: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = filter_lower.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    let val: f64 = parts[0].parse().ok()?;
    let from_unit = parts[1];
    let sep = parts[2];
    let to_unit = parts[3];

    if sep != "to" && sep != "in" && sep != "=" {
        return None;
    }

    let (res, symbol) = match (from_unit, to_unit) {
        ("usd", "eur") => (val * 0.92, "EUR"),
        ("eur", "usd") => (val * 1.09, "USD"),
        ("usd", "gbp") => (val * 0.79, "GBP"),
        ("gbp", "usd") => (val * 1.27, "USD"),
        ("eur", "gbp") => (val * 0.86, "GBP"),
        ("gbp", "eur") => (val * 1.16, "EUR"),
        ("usd", "jpy") => (val * 155.0, "JPY"),
        ("jpy", "usd") => (val * 0.0064, "USD"),
        ("km", "m") => (val * 1000.0, "m"),
        ("m", "km") => (val / 1000.0, "km"),
        ("km", "miles") | ("km", "mi") => (val * 0.621371, "mi"),
        ("miles", "km") | ("mi", "km") => (val * 1.60934, "km"),
        ("cm", "inch") | ("cm", "in") => (val * 0.393701, "in"),
        ("inch", "cm") | ("in", "cm") => (val * 2.54, "cm"),
        ("kg", "lbs") | ("kg", "lb") => (val * 2.20462, "lbs"),
        ("lbs", "kg") | ("lb", "kg") => (val * 0.453592, "kg"),
        ("c", "f") => (val * 9.0 / 5.0 + 32.0, "°F"),
        ("f", "c") => ((val - 32.0) * 5.0 / 9.0, "°C"),
        _ => return None,
    };

    let res_fmt = if (res - res.round()).abs() < 1e-9 {
        format!("{:.0}", res)
    } else {
        format!("{:.2}", res)
    };

    let title = format!("Conversione: {} {}", res_fmt, symbol);
    let desc = format!("{} {} = {} {} (Clicca per copiare)", val, from_unit.to_uppercase(), res_fmt, symbol);
    let copy_val = format!("{} {}", res_fmt, symbol);
    Some((title, desc, copy_val))
}

fn try_eval_math(_filter_text: &str, filter_lower: &str) -> Option<(String, String, String)> {
    let expr = filter_lower.trim_start_matches('=').trim_end_matches('=').trim();
    if expr.is_empty() {
        return None;
    }

    let has_explicit_equal = filter_lower.starts_with('=');
    let has_math_op = expr.contains('+') || expr.contains('*') || expr.contains('/') || expr.contains('^') || expr.contains('%')
        || expr.contains("sqrt") || expr.contains("sin") || expr.contains("cos") || expr.contains("abs");
    let has_subtraction = expr.contains('-') && expr.chars().any(|c| c.is_ascii_digit());

    if !has_explicit_equal && !has_math_op && !has_subtraction {
        return None;
    }

    if let Ok(res) = meval::eval_str(expr) {
        if res.is_finite() {
            let res_str = if (res - res.round()).abs() < 1e-9 {
                format!("{:.0}", res)
            } else {
                format!("{:.4}", res).trim_end_matches('0').trim_end_matches('.').to_string()
            };
            let title = format!("= {}", res_str);
            let desc = format!("Risultato calcolatrice: {} = {} (Clicca per copiare)", expr, res_str);
            let copy_val = res_str;
            return Some((title, desc, copy_val));
        }
    }
    None
}

fn try_parse_inline_action(filter_text: &str, filter_lower: &str) -> Option<(String, String, String)> {
    if let Some(res) = try_parse_conversions(filter_lower) {
        return Some(res);
    }
    if let Some(res) = try_eval_math(filter_text, filter_lower) {
        return Some(res);
    }
    None
}

fn try_parse_system_actions(list_box: &GtkBox, filter_lower: &str, pop: &ApplicationWindow) -> bool {
    let mut matched = false;

    // Dark Mode Action
    if filter_lower.contains("dark") || filter_lower.contains("light") || filter_lower.contains("tema") || filter_lower.contains("scuro") || filter_lower.contains("chiaro") {
        let row = Button::builder().css_classes(["spotlight-item"]).build();
        let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
        let img = Image::builder().icon_name("preferences-desktop-theme").pixel_size(40).build();
        hbox.append(&img);
        let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
        let name_lbl = Label::builder().label("Azione di Sistema: Toggle Dark Mode").halign(Align::Start).css_classes(["spotlight-item-title"]).build();
        vbox.append(&name_lbl);
        let desc_lbl = Label::builder().label("Alterna tra la modalità scura e chiara del desktop").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
        vbox.append(&desc_lbl);
        hbox.append(&vbox);
        row.set_child(Some(&hbox));
        row.connect_clicked(glib::clone!(@weak pop => move |_| {
            let _ = std::process::Command::new("ermete-settings-rs").arg("--toggle-dark-mode").spawn();
            let _ = std::process::Command::new("notify-send").arg("Ermete OS").arg("Modalità visiva aggiornata").spawn();
            pop.close();
        }));
        list_box.append(&row);
        matched = true;
    }

    // Kill Process Action
    if filter_lower.contains("kill") || filter_lower.contains("termina") || filter_lower.contains("process") || filter_lower.contains("processo") {
        let row = Button::builder().css_classes(["spotlight-item"]).build();
        let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
        let img = Image::builder().icon_name("process-stop").pixel_size(40).build();
        hbox.append(&img);
        let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
        let name_lbl = Label::builder().label("Azione di Sistema: Kill Process (Termina Processi)").halign(Align::Start).css_classes(["spotlight-item-title"]).build();
        vbox.append(&name_lbl);
        let desc_lbl = Label::builder().label("Finta azione Raycast: ricerca e interrompi processi in esecuzione").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
        vbox.append(&desc_lbl);
        hbox.append(&vbox);
        row.set_child(Some(&hbox));
        row.connect_clicked(glib::clone!(@weak pop => move |_| {
            let _ = std::process::Command::new("notify-send").arg("Ermete System Action").arg("Process Killer avviato").spawn();
            let _ = std::process::Command::new("foot").arg("-e").arg("htop").spawn();
            pop.close();
        }));
        list_box.append(&row);
        matched = true;
    }

    matched
}

fn try_parse_terminal_command(list_box: &GtkBox, filter_text: &str, filter_lower: &str, pop: &ApplicationWindow) -> bool {
    if filter_lower.starts_with('>') {
        let cmd = filter_text.trim_start_matches('>').trim();
        let row = Button::builder().css_classes(["spotlight-item"]).build();
        let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
        let img = Image::builder().icon_name("utilities-terminal").pixel_size(40).build();
        hbox.append(&img);
        let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
        let name_lbl = Label::builder().label(format!("Esegui: {}", cmd)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
        vbox.append(&name_lbl);
        let desc_lbl = Label::builder().label("Lancia comando nel terminale").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
        vbox.append(&desc_lbl);
        hbox.append(&vbox);
        row.set_child(Some(&hbox));
        let cmd_clone = cmd.to_string();
        row.connect_clicked(glib::clone!(@weak pop => move |_| {
            let _ = std::process::Command::new("foot").arg("-e").arg("sh").arg("-c").arg(format!("{}; read -p '\nPremi Invio per chiudere...'", cmd_clone)).spawn();
            pop.close();
        }));
        list_box.append(&row);
        return true;
    }
    false
}

fn try_parse_web_search(list_box: &GtkBox, filter_text: &str, filter_lower: &str, pop: &ApplicationWindow) -> bool {
    if filter_lower.starts_with('?') {
        let query = filter_text.trim_start_matches('?').trim();
        if !query.is_empty() {
            let row = Button::builder().css_classes(["spotlight-item"]).build();
            let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
            let img = Image::builder().icon_name("web-browser").pixel_size(40).build();
            hbox.append(&img);
            let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
            let name_lbl = Label::builder().label(format!("Cerca sul Web: {}", query)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
            vbox.append(&name_lbl);
            let desc_lbl = Label::builder().label("Cerca con il browser predefinito").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
            vbox.append(&desc_lbl);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            let query_encoded = query.replace(' ', "+");
            let search_url = format!("https://duckduckgo.com/?q={}", query_encoded);
            row.connect_clicked(glib::clone!(@weak pop => move |_| {
                let _ = std::process::Command::new("xdg-open").arg(&search_url).spawn();
                pop.close();
            }));
            list_box.append(&row);
        }
        return true;
    }
    false
}

fn run_fuzzy_file_search(list_box: &GtkBox, query: &str, pop: &ApplicationWindow) {
    if query.trim().is_empty() {
        return;
    }
    let query_str = query.to_string();
    let list_box_clone = list_box.clone();
    let pop_clone = pop.clone();

    glib::MainContext::default().spawn_local(async move {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home/ermete".to_string());
        
        let output_res = tokio::process::Command::new("fd")
            .arg("--max-results")
            .arg("5")
            .arg("--hidden")
            .arg("--exclude")
            .arg(".git")
            .arg("--exclude")
            .arg("target")
            .arg(&query_str)
            .arg(&home_dir)
            .output()
            .await;

        let stdout = match output_res {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            _ => {
                if let Ok(output) = tokio::process::Command::new("plocate")
                    .arg("-l")
                    .arg("5")
                    .arg(&query_str)
                    .output()
                    .await
                {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    String::new()
                }
            }
        };

        let mut count = 0;
        for line in stdout.lines() {
            let path_str = line.trim();
            if path_str.is_empty() { continue; }
            if count >= 5 { break; }

            let row = Button::builder().css_classes(["spotlight-item"]).build();
            let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
            let img = Image::builder().icon_name("text-x-generic").pixel_size(40).build();
            hbox.append(&img);

            let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
            let path = std::path::Path::new(path_str);
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let name_lbl = Label::builder().label(format!("File: {}", name)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
            vbox.append(&name_lbl);
            let desc_lbl = Label::builder().label(path_str).halign(Align::Start).css_classes(["spotlight-item-desc"]).ellipsize(gtk4::pango::EllipsizeMode::Middle).build();
            vbox.append(&desc_lbl);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));

            let file_path = path_str.to_string();
            row.connect_clicked(glib::clone!(@weak pop_clone => move |_| {
                let _ = std::process::Command::new("xdg-open").arg(&file_path).spawn();
                pop_clone.close();
            }));
            list_box_clone.append(&row);
            count += 1;
        }
    });
}

fn try_parse_ai_suggestion(list_box: &GtkBox, filter_text: &str, filter_lower: &str, pop: &ApplicationWindow) {
    if !filter_lower.is_empty() && !filter_lower.starts_with('/') && !filter_lower.starts_with('?') && !filter_lower.starts_with('=') && !filter_lower.starts_with('>') {
        let query = filter_text.trim_start_matches("ai:").trim();
        let row = Button::builder().css_classes(["spotlight-item"]).build();
        let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
        let img = Image::builder().icon_name("system-run").pixel_size(40).build();
        hbox.append(&img);
        let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
        let name_lbl = Label::builder().label(format!("Chiedi ad AI: {}", query)).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
        vbox.append(&name_lbl);
        let desc_lbl = Label::builder().label("Ermete AI capirà l'intento e aprirà il pannello corretto").halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
        vbox.append(&desc_lbl);
        hbox.append(&vbox);
        row.set_child(Some(&hbox));
        let query_str = query.to_string();
        row.connect_clicked(glib::clone!(@weak pop => move |_| {
            let json_query = format!(r#"{{"text": "{}", "intent": "auto"}}"#, query_str.replace('"', "\\\""));
            let _ = std::process::Command::new("ermete-ai-daemon")
                .arg("--query")
                .arg(&json_query)
                .spawn();
            pop.close();
        }));
        list_box.append(&row);
    }
}

fn populate_indexed_items(list_box: &GtkBox, filter_lower: &str, category_filter: &str, pop: &ApplicationWindow) {
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

            if !filter_lower.is_empty() && !item.title.to_lowercase().contains(filter_lower) && !item.description.to_lowercase().contains(filter_lower) && !item.keywords.contains(filter_lower) {
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
            row.connect_clicked(glib::clone!(@weak pop => move |_| {
                match &action_clone {
                    SpotlightAction::LaunchApp(app_info) => {
                        let _ = app_info.launch(&[], gtk4::gio::AppLaunchContext::NONE);
                    }
                    SpotlightAction::OpenSettingsPage(page) => {
                        let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", page));
                    }
                }
                pop.close();
            }));
            list_box.append(&row);
            count += 1;

            if count >= 20 {
                break; // Cap UI generation for <1ms response time
            }
        }
    });

    if count == 0 && list_box.first_child().is_none() {
        let no_res = Label::builder().label("Nessun risultato trovato.").css_classes(["cc-label-sub"]).margin_top(20).build();
        list_box.append(&no_res);
    }
}

pub fn populate_launcher_list(list_box: &GtkBox, filter_text: &str, category_filter: &str, is_spotlight: bool, pop: &ApplicationWindow) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    let filter_lower = filter_text.to_lowercase();
    let filter_trimmed = filter_text.trim();

    if is_spotlight && !filter_trimmed.is_empty() {
        // 1. Inline parsing: Math & Conversions
        if let Some((title, desc, copy_val)) = try_parse_inline_action(filter_trimmed, &filter_lower) {
            let row = Button::builder().css_classes(["spotlight-item"]).build();
            let hbox = GtkBox::builder().orientation(Orientation::Horizontal).spacing(16).build();
            let img = Image::builder().icon_name("accessories-calculator").pixel_size(40).build();
            hbox.append(&img);
            let vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
            let name_lbl = Label::builder().label(&title).halign(Align::Start).css_classes(["spotlight-item-title"]).build();
            vbox.append(&name_lbl);
            let desc_lbl = Label::builder().label(&desc).halign(Align::Start).css_classes(["spotlight-item-desc"]).build();
            vbox.append(&desc_lbl);
            hbox.append(&vbox);
            row.set_child(Some(&hbox));
            row.connect_clicked(glib::clone!(@weak pop => move |_| {
                let clipboard = pop.clipboard();
                clipboard.set_text(&copy_val);
                pop.close();
            }));
            list_box.append(&row);
        }

        // 2. System Actions ("Dark", "Kill", etc.)
        let is_system_action = try_parse_system_actions(list_box, &filter_lower, pop);

        // 3. Command & Web Search prefixes
        if try_parse_terminal_command(list_box, filter_text, &filter_lower, pop) {
            return;
        }
        if try_parse_web_search(list_box, filter_text, &filter_lower, pop) {
            return;
        }

        // 4. Direct File Search when starting with '/'
        if filter_lower.starts_with('/') {
            let query = filter_text.trim_start_matches('/').trim();
            run_fuzzy_file_search(list_box, query, pop);
            return;
        }

        // 5. AI suggestion
        try_parse_ai_suggestion(list_box, filter_text, &filter_lower, pop);

        // 6. Non-system fuzzy file search alongside indexed items
        if !is_system_action && filter_trimmed.len() >= 2 {
            run_fuzzy_file_search(list_box, filter_trimmed, pop);
        }
    }

    populate_indexed_items(list_box, &filter_lower, category_filter, pop);
}

pub fn show_spotlight_modal(app: &Application) {
    ensure_index_loaded();

    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Spotlight")
        .css_classes(["spotlight-window", "transparent"])
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
        .css_classes(["spotlight-card", "popover"])
        .build();

    let entry = Entry::builder()
        .placeholder_text("Cerca app, azioni (Dark, Kill), calcoli (2+2, 100 usd to eur), file...")
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
    entry.connect_changed(glib::clone!(@weak pop => move |e| {
        populate_launcher_list(&list_clone, &e.text(), "", true, &pop);
    }));

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
