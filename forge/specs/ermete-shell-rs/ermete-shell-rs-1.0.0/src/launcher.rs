//! GTK4 Grid App & File Launcher (Ermete OS - Phase 13)
//!
//! Features:
//! - Responsive FlowBox grid layout for installed applications and search results
//! - Integration with `ermete_style` for dynamic accent colors, contrast tokens, and glassmorphism styling
//! - GTK4 `SearchEntry` with real-time app filtering
//! - Non-blocking asynchronous file search using Tokio and GTK MainContext local spawning
//! - Category filter pills (Internet, Office, Graphics, Multimedia, Development, System, Games, Files)
//! - Power menu quick actions and keyboard navigation (Escape, Arrow keys)

use gtk4::gio::{self, AppInfo, DesktopAppInfo};
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, FlowBox, FlowBoxChild,
    Image, Label, Orientation, ScrolledWindow, SearchEntry, SelectionMode,
};
use gtk4_layer_shell::{Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{error, info};

// Import token colors from ermete_style
use ermete_style::accent_engine::{AccentEngineService, ColorRgb};

/// Represents an item displayed in the App Launcher grid
#[derive(Clone, Debug)]
pub enum LauncherItemKind {
    App(AppInfo),
    File { path: String, name: String, is_dir: bool },
    Setting { title: String, page: String },
}

#[derive(Clone, Debug)]
pub struct LauncherItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_name: String,
    pub category: String,
    pub keywords: String,
    pub kind: LauncherItemKind,
}

thread_local! {
    static LAUNCHER_WINDOW: RefCell<Option<ApplicationWindow>> = const { RefCell::new(None) };
}

/// Toggles launcher window visibility
pub fn toggle_launcher_visibility() {
    LAUNCHER_WINDOW.with(|win_ref| {
        if let Some(win) = win_ref.borrow().as_ref() {
            if win.is_visible() {
                win.close();
            } else {
                win.present();
            }
        }
    });
}

/// Scans desktop applications and system settings deeplinks
pub fn load_desktop_applications() -> Vec<LauncherItem> {
    let mut items = Vec::new();

    // 1. Settings deeplinks
    let settings_links = [
        ("Schermi", "preferences-desktop-display", "display monitor risoluzione hz niri", "displays"),
        ("Aspetto", "preferences-desktop-theme", "tema colori accent dark light wallpaper", "appearance"),
        ("Audio", "audio-card", "volume pipewire microfono cuffie", "audio"),
        ("Tastiera", "input-keyboard", "tastiera layout xkb keymaps", "keyboard"),
        ("Mouse & Touchpad", "input-mouse", "mouse touchpad gesture libinput", "mouse"),
        ("Wi-Fi & Rete", "network-wireless", "wifi rete ethernet networkmanager ip", "network"),
        ("Bluetooth", "bluetooth", "bluetooth bluez cuffie mouse pairing", "bluetooth"),
        ("Gestione Finestre", "preferences-system-windows", "niri windows focus workspace gaps", "focus"),
    ];

    for (name, icon, kw, page) in settings_links {
        items.push(LauncherItem {
            id: format!("setting:{}", page),
            title: name.to_string(),
            description: format!("Impostazioni Ermete OS - {}", name),
            icon_name: icon.to_string(),
            category: "Sistema".to_string(),
            keywords: format!("{} sistema impostazioni", kw),
            kind: LauncherItemKind::Setting {
                title: name.to_string(),
                page: page.to_string(),
            },
        });
    }

    // 2. Desktop applications from GIO AppInfo
    let apps: Vec<AppInfo> = AppInfo::all()
        .into_iter()
        .filter(|a| a.should_show())
        .collect();

    for app_info in apps {
        let title = app_info.display_name().to_string();
        let description = app_info
            .description()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let mut primary_category = "Tutte".to_string();
        let mut app_cats = String::new();

        if let Some(desktop_app) = app_info.downcast_ref::<DesktopAppInfo>() {
            if let Some(cats) = desktop_app.categories() {
                app_cats = cats.to_string().to_lowercase();
                if app_cats.contains("network") || app_cats.contains("webbrowser") {
                    primary_category = "Internet".to_string();
                } else if app_cats.contains("office") || app_cats.contains("wordprocessor") {
                    primary_category = "Ufficio".to_string();
                } else if app_cats.contains("graphics") || app_cats.contains("photography") {
                    primary_category = "Grafica".to_string();
                } else if app_cats.contains("audiovideo") || app_cats.contains("audio") || app_cats.contains("video") {
                    primary_category = "Multimedia".to_string();
                } else if app_cats.contains("development") {
                    primary_category = "Sviluppo".to_string();
                } else if app_cats.contains("game") {
                    primary_category = "Giochi".to_string();
                } else if app_cats.contains("system") || app_cats.contains("utility") {
                    primary_category = "Sistema".to_string();
                }
            }
        }

        let icon_name = match app_info.icon() {
            Some(gicon) => gtk4::prelude::IconExt::to_string(&gicon)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "application-x-executable".to_string()),
            None => "application-x-executable".to_string(),
        };

        let keywords = format!("{} {} {}", title.to_lowercase(), description.to_lowercase(), app_cats);

        items.push(LauncherItem {
            id: app_info.id().map(|s| s.to_string()).unwrap_or_else(|| title.clone()),
            title,
            description,
            icon_name,
            category: primary_category,
            keywords,
            kind: LauncherItemKind::App(app_info),
        });
    }

    items.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    items
}

/// Applies user-selected color tokens and dynamic accents from `ermete_style`
fn apply_launcher_theme_styles(window: &ApplicationWindow) {
    let accent_hex = AccentEngineService::new().get_accent_color();
    let accent_rgb = ColorRgb::parse_hex(&accent_hex);
    let contrasting_fg = accent_rgb.contrasting_fg().to_hex();
    let accent_border = accent_rgb.to_rgba_string(0.40);
    let accent_hover = accent_rgb.to_rgba_string(0.25);
    let accent_glow = accent_rgb.to_rgba_string(0.15);

    let custom_css = format!(
        "
        .launcher-grid-card {{
            background-color: rgba(20, 22, 34, 0.88);
            backdrop-filter: blur(28px);
            border-radius: 24px;
            border: 1px solid rgba(255, 255, 255, 0.12);
            box-shadow: 0 20px 48px rgba(0, 0, 0, 0.6);
        }}
        .launcher-search {{
            background-color: rgba(32, 34, 50, 0.75);
            border-radius: 14px;
            border: 1px solid {accent_border};
            color: #ffffff;
            font-size: 15px;
            padding: 10px 14px;
        }}
        .launcher-search:focus {{
            border-color: {accent_hex};
            box-shadow: 0 0 0 3px {accent_glow};
        }}
        .launcher-app-tile {{
            border-radius: 16px;
            padding: 14px 10px;
            transition: all 180ms cubic-bezier(0.2, 0.8, 0.2, 1.0);
            background-color: rgba(255, 255, 255, 0.03);
            border: 1px solid transparent;
        }}
        .launcher-app-tile:hover, .launcher-app-tile:focus {{
            background-color: {accent_hover};
            border: 1px solid {accent_border};
            box-shadow: 0 6px 18px {accent_glow};
        }}
        .launcher-cat-btn {{
            border-radius: 12px;
            padding: 7px 16px;
            font-weight: 600;
            font-size: 13px;
            background-color: rgba(255, 255, 255, 0.05);
            color: #a9b1d6;
            border: 1px solid transparent;
            transition: all 150ms ease;
        }}
        .launcher-cat-btn:hover {{
            background-color: rgba(255, 255, 255, 0.12);
            color: #ffffff;
        }}
        .launcher-cat-btn.active {{
            background-color: {accent_hex};
            color: {contrasting_fg};
            box-shadow: 0 4px 12px {accent_glow};
        }}
        .launcher-tile-title {{
            color: #c0caf5;
            font-size: 13px;
            font-weight: 500;
        }}
        .launcher-tile-desc {{
            color: #787c99;
            font-size: 11px;
        }}
        "
    );

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(&custom_css);

    let display = gtk4::prelude::WidgetExt::display(window);
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 10,
    );
}

/// Builds a FlowBox tile widget for an item
fn create_app_tile(item: LauncherItem, window: &ApplicationWindow) -> FlowBoxChild {
    let child = FlowBoxChild::new();

    let tile = Button::builder()
        .css_classes(["launcher-app-tile"])
        .halign(Align::Fill)
        .valign(Align::Fill)
        .build();

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let img = Image::builder().pixel_size(52).halign(Align::Center).build();

    if item.icon_name.starts_with('/') {
        img.set_from_file(Some(&item.icon_name));
    } else {
        img.set_icon_name(Some(&item.icon_name));
    }

    let title_lbl = Label::builder()
        .label(&item.title)
        .css_classes(["launcher-tile-title"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .max_width_chars(14)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .wrap(true)
        .build();

    vbox.append(&img);
    vbox.append(&title_lbl);

    tile.set_child(Some(&vbox));
    child.set_child(Some(&tile));

    let win_clone = window.clone();
    let kind_clone = item.kind.clone();

    tile.connect_clicked(move |_| {
        match &kind_clone {
            LauncherItemKind::App(app_info) => {
                info!(app = %app_info.display_name(), "Launching application from GTK4 Launcher");
                if let Err(e) = app_info.launch(&[], gio::AppLaunchContext::NONE) {
                    error!(error = %e, "Failed to launch application");
                }
            }
            LauncherItemKind::File { path, .. } => {
                info!(path = %path, "Opening local file from GTK4 Launcher");
                let _ = std::process::Command::new("xdg-open").arg(path).spawn();
            }
            LauncherItemKind::Setting { page, .. } => {
                info!(page = %page, "Opening settings page from GTK4 Launcher");
                let _ = gtk4::glib::spawn_command_line_async(format!("ermete-settings-rs --page {}", page));
            }
        }
        win_clone.close();
    });

    child
}

/// Asynchronous non-blocking file search engine
fn perform_async_file_search(
    flow_box: FlowBox,
    query: String,
    window: ApplicationWindow,
    search_generation: Rc<RefCell<u64>>,
    current_gen: u64,
) {
    if query.trim().len() < 2 {
        return;
    }

    let query_clean = query.trim().to_string();
    let gen_check = search_generation.clone();

    glib::MainContext::default().spawn_local(async move {
        // Execute fast plocate index lookup asynchronously
        let output = tokio::process::Command::new("plocate")
            .arg("-i")
            .arg("-l")
            .arg("12")
            .arg(&query_clean)
            .output()
            .await;

        // Discard stale search responses
        if *gen_check.borrow() != current_gen {
            return;
        }

        let mut results = Vec::new();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let path = std::path::Path::new(trimmed);
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let is_dir = path.is_dir();

                let icon_name = if is_dir {
                    "folder".to_string()
                } else if name.ends_with(".pdf") {
                    "document-page-pdf".to_string()
                } else if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".svg") {
                    "image-x-generic".to_string()
                } else if name.ends_with(".rs") || name.ends_with(".py") || name.ends_with(".sh") {
                    "text-x-script".to_string()
                } else {
                    "text-x-generic".to_string()
                };

                results.push(LauncherItem {
                    id: format!("file:{}", trimmed),
                    title: name.clone(),
                    description: trimmed.to_string(),
                    icon_name,
                    category: "File & Documenti".to_string(),
                    keywords: format!("{} {}", name.to_lowercase(), trimmed.to_lowercase()),
                    kind: LauncherItemKind::File {
                        path: trimmed.to_string(),
                        name,
                        is_dir,
                    },
                });
            }
        }

        if *gen_check.borrow() != current_gen {
            return;
        }

        for item in results {
            let tile = create_app_tile(item, &window);
            flow_box.append(&tile);
        }
    });
}

/// Filters applications and triggers async search
fn filter_and_populate_grid(
    flow_box: &FlowBox,
    items: &[LauncherItem],
    query: &str,
    category: &str,
    window: &ApplicationWindow,
    search_generation: &Rc<RefCell<u64>>,
) {
    while let Some(child) = flow_box.first_child() {
        flow_box.remove(&child);
    }

    let query_lower = query.to_lowercase().trim().to_string();
    let is_file_category = category == "File & Documenti";

    if !is_file_category {
        for item in items {
            if category != "Tutte" && item.category != category {
                continue;
            }

            if !query_lower.is_empty()
                && !item.title.to_lowercase().contains(&query_lower)
                && !item.description.to_lowercase().contains(&query_lower)
                && !item.keywords.contains(&query_lower)
            {
                continue;
            }

            let tile = create_app_tile(item.clone(), window);
            flow_box.append(&tile);
        }
    }

    // Trigger async file search if query is typed or category is files
    if !query_lower.is_empty() || is_file_category {
        let new_gen = {
            let mut gen = search_generation.borrow_mut();
            *gen += 1;
            *gen
        };
        perform_async_file_search(
            flow_box.clone(),
            if query_lower.is_empty() { "*".to_string() } else { query_lower },
            window.clone(),
            search_generation.clone(),
            new_gen,
        );
    }
}

/// Builds and displays the main GTK4 App Launcher Window
pub fn show_launcher_window(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete App Launcher")
        .css_classes(["launcher-window", "transparent"])
        .default_width(820)
        .default_height(600)
        .build();

    // GTK4 Layer Shell setup
    window.init_layer_shell();
    window.set_namespace("launcher");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    crate::ui::popup_manager::setup_popup_autoclose(&window, "launcher");

    apply_launcher_theme_styles(&window);

    let main_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["launcher-grid-card"])
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    // Header Search Bar
    let header_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(18)
        .margin_start(20)
        .margin_end(20)
        .build();

    let search_entry = SearchEntry::builder()
        .placeholder_text("Cerca applicazioni, file locali o impostazioni...")
        .css_classes(["launcher-search"])
        .hexpand(true)
        .build();

    header_box.append(&search_entry);

    // Category Selector Bar
    let cat_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_start(20)
        .margin_end(20)
        .build();

    let categories = [
        "Tutte",
        "Internet",
        "Ufficio",
        "Grafica",
        "Multimedia",
        "Sviluppo",
        "Sistema",
        "Giochi",
        "File & Documenti",
    ];

    let active_category = Rc::new(RefCell::new("Tutte".to_string()));
    let category_buttons: Rc<RefCell<Vec<(String, Button)>>> = Rc::new(RefCell::new(Vec::new()));
    let items_cache = Rc::new(load_desktop_applications());
    let search_gen = Rc::new(RefCell::new(0u64));

    // FlowBox Grid Widget
    let flow_box = FlowBox::builder()
        .selection_mode(SelectionMode::Single)
        .max_children_per_line(6)
        .min_children_per_line(4)
        .row_spacing(12)
        .column_spacing(12)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(16)
        .margin_end(16)
        .homogeneous(true)
        .halign(Align::Fill)
        .valign(Align::Start)
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(400)
        .build();

    scroll.set_child(Some(&flow_box));

    // Initial populate
    filter_and_populate_grid(
        &flow_box,
        &items_cache,
        "",
        "Tutte",
        &window,
        &search_gen,
    );

    // Build category buttons
    for cat in categories {
        let btn = Button::builder()
            .label(cat)
            .css_classes(["launcher-cat-btn"])
            .build();

        if cat == "Tutte" {
            btn.add_css_class("active");
        }

        let cat_str = cat.to_string();
        let flow_box_clone = flow_box.clone();
        let items_clone = items_cache.clone();
        let active_cat_clone = active_category.clone();
        let search_clone = search_entry.clone();
        let win_clone = window.clone();
        let gen_clone = search_gen.clone();
        let buttons_ref = category_buttons.clone();

        btn.connect_clicked(move |_| {
            *active_cat_clone.borrow_mut() = cat_str.clone();

            for (c_name, b_w) in buttons_ref.borrow().iter() {
                if c_name == &cat_str {
                    b_w.add_css_class("active");
                } else {
                    b_w.remove_css_class("active");
                }
            }

            filter_and_populate_grid(
                &flow_box_clone,
                &items_clone,
                &search_clone.text(),
                &cat_str,
                &win_clone,
                &gen_clone,
            );
        });

        category_buttons
            .borrow_mut()
            .push((cat.to_string(), btn.clone()));
        cat_box.append(&btn);
    }

    // Connect search entry
    let flow_box_search = flow_box.clone();
    let items_search = items_cache.clone();
    let active_cat_search = active_category.clone();
    let win_search = window.clone();
    let gen_search = search_gen.clone();

    search_entry.connect_search_changed(move |e| {
        filter_and_populate_grid(
            &flow_box_search,
            &items_search,
            &e.text(),
            &active_cat_search.borrow(),
            &win_search,
            &gen_search,
        );
    });

    // Keyboard Shortcuts (Escape to close, Down Arrow to move focus to grid)
    let key_ctrl = gtk4::EventControllerKey::new();
    let win_esc = window.clone();
    let flow_focus = flow_box.clone();

    key_ctrl.connect_key_pressed(move |_, keyval, _, _| match keyval {
        gtk4::gdk::Key::Escape => {
            win_esc.close();
            glib::Propagation::Stop
        }
        gtk4::gdk::Key::Down => {
            if let Some(child) = flow_focus.first_child() {
                child.grab_focus();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        }
        _ => glib::Propagation::Proceed,
    });
    window.add_controller(key_ctrl);

    // Footer with power menu
    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_bottom(16)
        .margin_start(20)
        .margin_end(20)
        .build();

    let user_lbl = Label::builder()
        .label("👤  Ermete OS User")
        .css_classes(["launcher-tile-title"])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let settings_btn = Button::builder()
        .label("⚙  Impostazioni")
        .css_classes(["launcher-cat-btn"])
        .build();

    let win_set = window.clone();
    settings_btn.connect_clicked(move |_| {
        let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs");
        win_set.close();
    });

    let power_btn = Button::builder()
        .label("⏻  Spegni")
        .css_classes(["launcher-cat-btn"])
        .build();

    let win_pow = window.clone();
    power_btn.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_power_controller();
            let _ = ctrl.power_off().await;
        });
        win_pow.close();
    });

    footer_box.append(&user_lbl);
    footer_box.append(&settings_btn);
    footer_box.append(&power_btn);

    // Assemble window layout
    main_card.append(&header_box);
    main_card.append(&cat_box);
    main_card.append(&scroll);
    main_card.append(&footer_box);

    window.set_child(Some(&main_card));
    window.present();
    search_entry.grab_focus();

    LAUNCHER_WINDOW.with(|ref_win| {
        *ref_win.borrow_mut() = Some(window);
    });
}
