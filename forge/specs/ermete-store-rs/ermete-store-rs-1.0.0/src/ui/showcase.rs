use gtk4::glib;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use crate::backend::repository::{get_featured_catalog, AppItem};

#[derive(Default, Debug)]
pub struct FilterState {
    pub search_query: String,
    pub active_category: String,
}

/// Modello del componente Showcase (Homepage dello Store)
pub struct ShowcaseModel {
    pub apps: Vec<AppItem>,
    pub search_query: String,
    pub active_category: String,
    pub filter_state: std::rc::Rc<std::cell::RefCell<FilterState>>,
}

/// Messaggi di input per il componente Showcase
#[derive(Debug)]
pub enum ShowcaseMsg {
    Search(String),
    SelectCategory(String),
    Install(String),
    OpenApp(String),
}

#[relm4::component(pub)]
impl SimpleComponent for ShowcaseModel {
    type Init = ();
    type Input = ShowcaseMsg;
    type Output = ();

    view! {
        gtk4::ScrolledWindow {
            set_hscrollbar_policy: gtk4::PolicyType::Never,
            set_vscrollbar_policy: gtk4::PolicyType::Automatic,
            set_vexpand: true,
            set_hexpand: true,
            add_css_class: "showcase-scroll",

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 24,
                set_margin_start: 32,
                set_margin_end: 32,
                set_margin_top: 24,
                set_margin_bottom: 32,

                // Hero Banner Promozionale stile Windows 11 Store
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 12,
                    add_css_class: "hero-banner",
                    set_margin_bottom: 8,

                    gtk4::Label {
                        set_label: "Benvenuto su Ermete Store",
                        set_halign: gtk4::Align::Start,
                        add_css_class: "hero-title",
                    },

                    gtk4::Label {
                        set_label: "Esplora applicazioni curate per Ermete OS, pacchetti Flatpak e OCI nativi.",
                        set_halign: gtk4::Align::Start,
                        add_css_class: "hero-subtitle",
                    },

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 12,
                        set_margin_top: 8,

                        gtk4::Button {
                            set_label: "⚡ In Evidenza: Vivaldi Browser",
                            add_css_class: "hero-btn-primary",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::OpenApp("com.vivaldi.Vivaldi".to_string()));
                            }
                        },

                        gtk4::Button {
                            set_label: "🎮 Gaming Hub: Steam",
                            add_css_class: "hero-btn-secondary",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::OpenApp("com.valvesoftware.Steam".to_string()));
                            }
                        }
                    }
                },

                // Barra di ricerca & Filtri
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 16,

                    gtk4::SearchEntry {
                        set_placeholder_text: Some("Cerca app, giochi, strumenti di sviluppo..."),
                        set_hexpand: true,
                        add_css_class: "glass-search",
                        connect_search_changed[sender] => move |entry| {
                            sender.input(ShowcaseMsg::Search(entry.text().to_string()));
                        }
                    },
                },

                // Filtri a pillola per categoria
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 8,
                    add_css_class: "category-bar",

                    gtk4::Button {
                        set_label: "Tutti",
                        add_css_class: "pill-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(ShowcaseMsg::SelectCategory("All".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "Navigazione",
                        add_css_class: "pill-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(ShowcaseMsg::SelectCategory("Browser".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "Gaming",
                        add_css_class: "pill-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(ShowcaseMsg::SelectCategory("Gaming".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "Sviluppo",
                        add_css_class: "pill-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(ShowcaseMsg::SelectCategory("Development".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "Produttività",
                        add_css_class: "pill-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(ShowcaseMsg::SelectCategory("Productivity".to_string()));
                        }
                    },
                },

                // Intestazione sezione
                gtk4::Label {
                    set_label: "Applicazioni Consigliate",
                    set_halign: gtk4::Align::Start,
                    add_css_class: "section-heading",
                },

                // Grid Reattiva GTK4 FlowBox per Card Applicazioni
                #[name = "flow_box"]
                gtk4::FlowBox {
                    set_valign: gtk4::Align::Start,
                    set_max_children_per_line: 4,
                    set_min_children_per_line: 1,
                    set_selection_mode: gtk4::SelectionMode::None,
                    set_column_spacing: 16,
                    set_row_spacing: 16,
                    add_css_class: "app-grid",
                    invalidate_filter: (),
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let filter_state = std::rc::Rc::new(std::cell::RefCell::new(FilterState {
            search_query: String::new(),
            active_category: "All".to_string(),
        }));

        let model = ShowcaseModel {
            apps: get_featured_catalog(),
            search_query: String::new(),
            active_category: "All".to_string(),
            filter_state,
        };

        let widgets = view_output!();

        // Popola il FlowBox con le card in modo reattivo e sicuro ed imposta il filtro nativo GTK
        model.populate_flow_box(&widgets.flow_box, &sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ShowcaseMsg::Search(query) => {
                self.search_query = query.clone();
                self.filter_state.borrow_mut().search_query = query;
            }
            ShowcaseMsg::SelectCategory(cat) => {
                self.active_category = cat.clone();
                self.filter_state.borrow_mut().active_category = cat;
            }
            ShowcaseMsg::Install(app_id) => {
                println!("[OverlayFS/Nix] 🚀 Lancio immediato overlay fittizio per '{}'...", app_id);
                println!("[Demone] ⬇️ Avvio download in background per '{}' (Lazy Loading)...", app_id);
                
                if let Some(app) = self.apps.iter_mut().find(|a| a.id == app_id) {
                    app.installed = true; // Disponibile istantaneamente
                }
                
                // Simulazione asincrona del download in background
                let id_clone = app_id.clone();
                glib::spawn_future_local(async move {
                    glib::timeout_future(std::time::Duration::from_secs(3)).await;
                    println!("[Demone] ✅ Download completato per '{}'. Overlay sincronizzato e reso persistente.", id_clone);
                });
            }
            ShowcaseMsg::OpenApp(app_id) => {
                println!("[Store] Apertura applicazione '{}'", app_id);
            }
        }
    }
}

impl ShowcaseModel {
    /// Popola il `gtk4::FlowBox` una sola volta ed imposta il filtro nativo GTK (`set_filter_func`)
    pub fn populate_flow_box(
        &self,
        flow_box: &gtk4::FlowBox,
        sender: &ComponentSender<Self>,
    ) {
        // Rimuove in sicurezza eventuali figli esistenti (zero unwrap)
        while let Some(child) = flow_box.first_child() {
            flow_box.remove(&child);
        }

        let apps_data: Vec<(String, String)> = self
            .apps
            .iter()
            .map(|a| {
                (
                    a.category.clone(),
                    format!("{} {}", a.name, a.summary).to_lowercase(),
                )
            })
            .collect();

        for app in &self.apps {
            let card = build_app_card(app, sender);
            flow_box.insert(&card, -1);
        }

        let filter_state = self.filter_state.clone();
        flow_box.set_filter_func(move |child| {
            let idx = child.index();
            if idx < 0 || (idx as usize) >= apps_data.len() {
                return true;
            }
            let (category, search_text) = &apps_data[idx as usize];
            let state = filter_state.borrow();

            // Filtro Categoria
            if !state.active_category.is_empty() && state.active_category != "All" {
                if category != &state.active_category {
                    return false;
                }
            }

            // Filtro Ricerca
            if !state.search_query.is_empty() {
                let q = state.search_query.to_lowercase();
                if !search_text.contains(&q) {
                    return false;
                }
            }

            true
        });
    }
}

/// Costruisce una Card per applicazione con stile Glassmorphism usando widget pure GTK4
fn build_app_card(app: &AppItem, sender: &ComponentSender<ShowcaseModel>) -> gtk4::Box {
    let card_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    card_box.add_css_class("store-card");
    card_box.set_width_request(230);

    // Intestazione Card: Icona + Meta
    let top_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

    let icon = gtk4::Image::from_icon_name(&app.icon);
    icon.set_pixel_size(44);
    icon.add_css_class("app-icon");
    top_box.append(&icon);

    let meta_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    meta_box.set_hexpand(true);

    let title_lbl = gtk4::Label::new(Some(&app.name));
    title_lbl.set_halign(gtk4::Align::Start);
    title_lbl.add_css_class("app-card-title");
    meta_box.append(&title_lbl);

    let cat_lbl = gtk4::Label::new(Some(&app.category));
    cat_lbl.set_halign(gtk4::Align::Start);
    cat_lbl.add_css_class("app-card-category");
    meta_box.append(&cat_lbl);

    let rating_lbl = gtk4::Label::new(Some(&format!("★ {:.1}", app.rating)));
    rating_lbl.set_halign(gtk4::Align::Start);
    rating_lbl.add_css_class("app-card-rating");
    meta_box.append(&rating_lbl);

    top_box.append(&meta_box);
    card_box.append(&top_box);

    // Descrizione sintetica
    let summary_lbl = gtk4::Label::new(Some(&app.summary));
    summary_lbl.set_wrap(true);
    summary_lbl.set_lines(2);
    summary_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    summary_lbl.set_halign(gtk4::Align::Start);
    summary_lbl.add_css_class("app-card-summary");
    card_box.append(&summary_lbl);

    // Pulsante di Azione (Installa / Apri)
    let action_btn = gtk4::Button::new();
    if app.installed {
        action_btn.set_label("Apri");
        action_btn.add_css_class("btn-installed");
    } else {
        action_btn.set_label("Usa Istantaneamente");
        action_btn.add_css_class("btn-install");
    }

    let app_id = app.id.clone();
    let sender_clone = sender.clone();
    let is_installed = app.installed;
    action_btn.connect_clicked(move |_| {
        if is_installed {
            sender_clone.input(ShowcaseMsg::OpenApp(app_id.clone()));
        } else {
            sender_clone.input(ShowcaseMsg::Install(app_id.clone()));
            // Simula anche qui l'apertura immediata
            sender_clone.input(ShowcaseMsg::OpenApp(app_id.clone()));
        }
    });

    card_box.append(&action_btn);

    card_box
}
