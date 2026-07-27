use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

/// Struttura dati per rappresentare un'applicazione nello Store
#[derive(Debug, Clone)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub icon: String,
    pub rating: f32,
    pub installed: bool,
}

/// Modello del componente Showcase (Homepage dello Store)
pub struct ShowcaseModel {
    pub apps: Vec<AppItem>,
    pub search_query: String,
    pub active_category: String,
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
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let default_apps = vec![
            AppItem {
                id: "com.vivaldi.Vivaldi".to_string(),
                name: "Vivaldi".to_string(),
                summary: "Browser web potente, altamente personalizzabile e orientato alla privacy.".to_string(),
                category: "Browser".to_string(),
                icon: "web-browser".to_string(),
                rating: 4.9,
                installed: true,
            },
            AppItem {
                id: "com.valvesoftware.Steam".to_string(),
                name: "Steam".to_string(),
                summary: "La piattaforma di distribuzione digitale e gaming di riferimento su Linux.".to_string(),
                category: "Gaming".to_string(),
                icon: "input-gaming".to_string(),
                rating: 4.8,
                installed: false,
            },
            AppItem {
                id: "org.mozilla.firefox".to_string(),
                name: "Firefox".to_string(),
                summary: "Navigazione web veloce, sicura e indipendente.".to_string(),
                category: "Browser".to_string(),
                icon: "firefox".to_string(),
                rating: 4.7,
                installed: true,
            },
            AppItem {
                id: "com.visualstudio.code".to_string(),
                name: "VS Code".to_string(),
                summary: "Ambiente di sviluppo leggero, potente ed estendibile.".to_string(),
                category: "Development".to_string(),
                icon: "text-editor".to_string(),
                rating: 4.9,
                installed: false,
            },
            AppItem {
                id: "com.discordapp.Discord".to_string(),
                name: "Discord".to_string(),
                summary: "Piattaforma di comunicazione vocale e testuale per community.".to_string(),
                category: "Social".to_string(),
                icon: "call-start".to_string(),
                rating: 4.6,
                installed: false,
            },
            AppItem {
                id: "com.spotify.Client".to_string(),
                name: "Spotify".to_string(),
                summary: "Streaming musicale in alta qualità con milioni di brani e podcast.".to_string(),
                category: "Media".to_string(),
                icon: "audio-x-generic".to_string(),
                rating: 4.7,
                installed: false,
            },
            AppItem {
                id: "md.obsidian.Obsidian".to_string(),
                name: "Obsidian".to_string(),
                summary: "Base di conoscenza personale basata su file Markdown locali.".to_string(),
                category: "Productivity".to_string(),
                icon: "accessories-text-editor".to_string(),
                rating: 4.9,
                installed: false,
            },
            AppItem {
                id: "org.gimp.GIMP".to_string(),
                name: "GIMP".to_string(),
                summary: "Editor di grafica raster avanzato e fotoritocco professionale.".to_string(),
                category: "Graphics".to_string(),
                icon: "image-x-generic".to_string(),
                rating: 4.5,
                installed: false,
            },
        ];

        let model = ShowcaseModel {
            apps: default_apps,
            search_query: String::new(),
            active_category: "All".to_string(),
        };

        let widgets = view_output!();

        // Popola il FlowBox con le card in modo reattivo e sicuro
        model.populate_flow_box(&widgets.flow_box, &sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ShowcaseMsg::Search(query) => {
                self.search_query = query;
            }
            ShowcaseMsg::SelectCategory(cat) => {
                self.active_category = cat;
            }
            ShowcaseMsg::Install(app_id) => {
                if let Some(app) = self.apps.iter_mut().find(|a| a.id == app_id) {
                    app.installed = !app.installed;
                }
            }
            ShowcaseMsg::OpenApp(_app_id) => {
                // Notifica o avvio app
            }
        }
    }
}

impl ShowcaseModel {
    /// Popola il `gtk4::FlowBox` in base ai filtri attivi (ricerca e categoria)
    pub fn populate_flow_box(
        &self,
        flow_box: &gtk4::FlowBox,
        sender: &ComponentSender<Self>,
    ) {
        // Rimuove in sicurezza eventuali figli esistenti (zero unwrap)
        while let Some(child) = flow_box.first_child() {
            flow_box.remove(&child);
        }

        for app in &self.apps {
            // Filtro Categoria
            if !self.active_category.is_empty() && self.active_category != "All" {
                if app.category != self.active_category {
                    continue;
                }
            }

            // Filtro Ricerca
            if !self.search_query.is_empty() {
                let q = self.search_query.to_lowercase();
                let name_match = app.name.to_lowercase().contains(&q);
                let summary_match = app.summary.to_lowercase().contains(&q);
                if !name_match && !summary_match {
                    continue;
                }
            }

            let card = build_app_card(app, sender);
            flow_box.insert(&card, -1);
        }
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
        action_btn.set_label("Ottieni");
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
        }
    });

    card_box.append(&action_btn);

    card_box
}
