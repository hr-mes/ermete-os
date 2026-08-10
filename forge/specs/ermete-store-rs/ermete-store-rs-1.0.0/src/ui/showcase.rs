use gtk4::glib;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use crate::backend::repository::{get_featured_catalog, AppItem, SandboxTier};

#[derive(Default, Debug)]
pub struct FilterState {
    pub search_query: String,
    pub active_category: String,
}

/// Modello del componente Showcase (Homepage dello Store con Hero Carousel 16:9, Video Preview, Badges e Pay-What-You-Can)
pub struct ShowcaseModel {
    pub apps: Vec<AppItem>,
    pub hero_index: usize,
    pub search_query: String,
    pub active_category: String,
    pub filter_state: std::rc::Rc<std::cell::RefCell<FilterState>>,
    pub selected_donations: std::collections::HashMap<String, u32>,
    pub notification_toast: Option<String>,
}

/// Messaggi di input per il componente Showcase
#[derive(Debug)]
pub enum ShowcaseMsg {
    Search(String),
    SelectCategory(String),
    Install(String),
    OpenApp(String),
    NextHero,
    PrevHero,
    SelectHero(usize),
    SetDonationAmount { app_id: String, amount: u32 },
    Donate { app_id: String, amount: u32 },
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

                // Toast Notifica Donazioni
                gtk4::Box {
                    set_visible: model.notification_toast.is_some(),
                    add_css_class: "notification-toast",
                    set_halign: gtk4::Align::Center,

                    gtk4::Label {
                        set_label: model.notification_toast.as_deref().unwrap_or(""),
                    }
                },

                // Hero Banner Carousel 16:9 Promozionale stile Apple Store & Deepin Store
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_bottom: 8,

                    // Intestazione sezione Hero Banner
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 12,

                        gtk4::Label {
                            set_label: "⚡ In Evidenza su Ermete OS",
                            set_halign: gtk4::Align::Start,
                            add_css_class: "hero-title",
                        },

                        gtk4::Box { set_hexpand: true },

                        gtk4::Label {
                            set_label: "Card 16:9 • MicroVM Sandboxed • Anteprime Video",
                            add_css_class: "hero-developer",
                        }
                    },

                    // Contenitore Hero Stack + Controlli di Navigazione Carousel
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 12,
                        set_valign: gtk4::Align::Center,

                        // Pulsante Precedente (<)
                        gtk4::Button {
                            set_label: "❮",
                            add_css_class: "carousel-nav-btn",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::PrevHero);
                            }
                        },

                        // Stack centrale per le Card 16:9 in Evidenza
                        #[name = "hero_stack"]
                        gtk4::Stack {
                            set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
                            set_transition_duration: 300,
                            set_hexpand: true,
                            set_visible_child_name: &format!("hero_{}", model.hero_index),
                        },

                        // Pulsante Successivo (>)
                        gtk4::Button {
                            set_label: "❯",
                            add_css_class: "carousel-nav-btn",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::NextHero);
                            }
                        }
                    },

                    // Indicator Dots per le Slide del Hero Carousel
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 8,
                        set_halign: gtk4::Align::Center,
                        set_margin_top: 4,

                        gtk4::Button {
                            set_label: if model.hero_index == 0 { "●" } else { "○" },
                            add_css_class: "carousel-dot",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::SelectHero(0));
                            }
                        },
                        gtk4::Button {
                            set_label: if model.hero_index == 1 { "●" } else { "○" },
                            add_css_class: "carousel-dot",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::SelectHero(1));
                            }
                        },
                        gtk4::Button {
                            set_label: if model.hero_index == 2 { "●" } else { "○" },
                            add_css_class: "carousel-dot",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::SelectHero(2));
                            }
                        },
                        gtk4::Button {
                            set_label: if model.hero_index == 3 { "●" } else { "○" },
                            add_css_class: "carousel-dot",
                            connect_clicked[sender] => move |_| {
                                sender.input(ShowcaseMsg::SelectHero(3));
                            }
                        },
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
                    set_label: "Tutte le Applicazioni (MicroVM & Flatpak)",
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
            hero_index: 0,
            search_query: String::new(),
            active_category: "All".to_string(),
            filter_state,
            selected_donations: std::collections::HashMap::new(),
            notification_toast: None,
        };

        let widgets = view_output!();

        // Popola hero_stack con le slide 16:9
        for (idx, app) in model.apps.iter().take(4).enumerate() {
            let slide = build_hero_slide(app, &sender);
            widgets.hero_stack.add_named(&slide, Some(&format!("hero_{}", idx)));
        }

        // Avvia il timer di auto-scorrimento del Hero Carousel ogni 6 secondi
        let sender_timer = sender.clone();
        glib::timeout_add_local(std::time::Duration::from_secs(6), move || {
            sender_timer.input(ShowcaseMsg::NextHero);
            glib::ControlFlow::Continue
        });

        // Popola il FlowBox con le card in modo reattivo ed imposta il filtro nativo GTK
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
            ShowcaseMsg::NextHero => {
                let count = self.apps.len().min(4);
                if count > 0 {
                    self.hero_index = (self.hero_index + 1) % count;
                }
            }
            ShowcaseMsg::PrevHero => {
                let count = self.apps.len().min(4);
                if count > 0 {
                    self.hero_index = if self.hero_index == 0 { count - 1 } else { self.hero_index - 1 };
                }
            }
            ShowcaseMsg::SelectHero(idx) => {
                let count = self.apps.len().min(4);
                if idx < count {
                    self.hero_index = idx;
                }
            }
            ShowcaseMsg::SetDonationAmount { app_id, amount } => {
                self.selected_donations.insert(app_id.clone(), amount);
                if let Some(app) = self.apps.iter_mut().find(|a| a.id == app_id) {
                    app.suggested_donation = amount;
                }
            }
            ShowcaseMsg::Donate { app_id, amount } => {
                let (app_name, dev) = self.apps.iter()
                    .find(|a| a.id == app_id)
                    .map(|a| (a.name.clone(), a.developer.clone()))
                    .unwrap_or_else(|| ("App".to_string(), "Developer".to_string()));
                
                tracing::info!("[Pay-What-You-Can] ❤️ Ricevuto contributo di ${} per l'applicazione '{}' ({}) via Kyber Mesh!", amount, app_name, dev);
                self.notification_toast = Some(format!("❤️ Donazione di ${} inviata a {} per '{}'!", amount, dev, app_name));
            }
            ShowcaseMsg::Install(app_id) => {
                tracing::info!("[OverlayFS/Nix] 🚀 Lancio immediato overlay fittizio per '{}'...", app_id);
                tracing::info!("[Demone] ⬇️ Avvio download in background per '{}' (Lazy Loading)...", app_id);
                
                if let Some(app) = self.apps.iter_mut().find(|a| a.id == app_id) {
                    app.installed = true;
                }
                
                let id_clone = app_id.clone();
                glib::spawn_future_local(async move {
                    glib::timeout_future(std::time::Duration::from_secs(3)).await;
                    tracing::info!("[Demone] ✅ Download completato per '{}'. Overlay sincronizzato e reso persistente.", id_clone);
                });
            }
            ShowcaseMsg::OpenApp(app_id) => {
                tracing::info!("[Store] Apertura applicazione '{}'", app_id);
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

            if !state.active_category.is_empty() && state.active_category != "All" && category != &state.active_category {
                return false;
            }

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

/// Visual Sandboxing Badge Helper Component
fn build_sandbox_badge(tier: &SandboxTier) -> gtk4::Box {
    let badge = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    badge.add_css_class("sandbox-badge");
    badge.add_css_class(tier.css_class());

    let icon = gtk4::Image::from_icon_name(tier.icon_name());
    icon.set_pixel_size(14);
    badge.append(&icon);

    let label = gtk4::Label::new(Some(tier.label()));
    label.add_css_class("badge-label");
    badge.append(&label);

    badge
}

/// Anteprime Video on Hover Widget Component
fn build_video_hover_widget(app: &AppItem) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    container.set_width_request(160);
    container.add_css_class("video-hover-container");

    let status_lbl = gtk4::Label::new(Some("▶ Anteprima Video"));
    status_lbl.add_css_class("video-preview-badge");
    status_lbl.set_halign(gtk4::Align::Center);
    container.append(&status_lbl);

    let video_widget = gtk4::Video::new();
    video_widget.set_size_request(160, 90); // 16:9 ratio
    video_widget.set_visible(false);
    container.append(&video_widget);

    // Dynamic Hover motion event controller
    let motion = gtk4::EventControllerMotion::new();
    let video_widget_c = video_widget.clone();
    let status_lbl_c = status_lbl.clone();
    let video_url = app.video_preview_url.clone();

    motion.connect_enter(move |_, _, _| {
        video_widget_c.set_visible(true);
        status_lbl_c.add_css_class("playing");
        status_lbl_c.set_label("🔴 Riproduzione Video");

        if let Some(ref uri) = video_url {
            let file = gtk4::gio::File::for_uri(uri);
            video_widget_c.set_file(Some(&file));
            video_widget_c.set_autoplay(true);
            video_widget_c.set_loop(true);
        }
    });

    let video_widget_c2 = video_widget.clone();
    let status_lbl_c2 = status_lbl.clone();

    motion.connect_leave(move |_| {
        video_widget_c2.set_visible(false);
        status_lbl_c2.remove_css_class("playing");
        status_lbl_c2.set_label("▶ Anteprima Video");
    });

    container.add_controller(motion);
    container
}

/// Pay-What-You-Can Component (stile elementary OS)
fn build_pay_what_you_can_box(
    app: &AppItem,
    sender: &ComponentSender<ShowcaseModel>,
) -> gtk4::Box {
    let pay_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    pay_box.add_css_class("pay-container");
    pay_box.set_halign(gtk4::Align::Fill);

    let amounts = [0, 5, 10, 20];
    let app_id = app.id.clone();
    let current_donation = app.suggested_donation;

    for &amt in &amounts {
        let btn = gtk4::Button::new();
        let label = if amt == 0 { "Gratis".to_string() } else { format!("${}", amt) };
        btn.set_label(&label);
        btn.add_css_class("pay-pill");
        if amt == current_donation {
            btn.add_css_class("pay-pill-active");
        }

        let app_id_c = app_id.clone();
        let sender_c = sender.clone();
        btn.connect_clicked(move |_| {
            sender_c.input(ShowcaseMsg::SetDonationAmount {
                app_id: app_id_c.clone(),
                amount: amt,
            });
        });
        pay_box.append(&btn);
    }

    let donate_btn = gtk4::Button::new();
    let donate_label = if current_donation == 0 {
        "⬇️ Free".to_string()
    } else {
        format!("❤️ Dona ${}", current_donation)
    };
    donate_btn.set_label(&donate_label);
    donate_btn.add_css_class("btn-donate");

    let app_id_d = app.id.clone();
    let sender_d = sender.clone();
    donate_btn.connect_clicked(move |_| {
        sender_d.input(ShowcaseMsg::Donate {
            app_id: app_id_d.clone(),
            amount: current_donation,
        });
    });

    pay_box.append(&donate_btn);
    pay_box
}

/// Costruisce una Slide 16:9 per il Hero Banner Carousel
fn build_hero_slide(app: &AppItem, sender: &ComponentSender<ShowcaseModel>) -> gtk4::AspectFrame {
    let aspect_frame = gtk4::AspectFrame::builder()
        .ratio(1.7777778) // 16:9 Aspect Ratio
        .xalign(0.5)
        .yalign(0.5)
        .obey_child(false)
        .build();
    aspect_frame.add_css_class("hero-aspect-frame");

    let overlay = gtk4::Overlay::new();

    let card_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    card_box.add_css_class("hero-card-16-9");

    // Intestazione Hero: Rating + Sandboxing Badge
    let top_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    
    let rating_lbl = gtk4::Label::new(Some(&format!("★ {:.1} Rating", app.rating)));
    rating_lbl.add_css_class("hero-rating");
    top_row.append(&rating_lbl);

    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    top_row.append(&spacer);

    let sandbox_badge = build_sandbox_badge(&app.sandbox);
    top_row.append(&sandbox_badge);

    card_box.append(&top_row);

    // Sezione Centrale: Icona + Titolo/Descrizione + Video Preview Widget
    let mid_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 20);

    let icon = gtk4::Image::from_icon_name(&app.icon);
    icon.set_pixel_size(64);
    icon.add_css_class("hero-app-icon");
    mid_row.append(&icon);

    let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    info_box.set_hexpand(true);

    let title_lbl = gtk4::Label::new(Some(&app.name));
    title_lbl.set_halign(gtk4::Align::Start);
    title_lbl.add_css_class("hero-title");
    info_box.append(&title_lbl);

    let dev_lbl = gtk4::Label::new(Some(&format!("Sviluppato da {}", app.developer)));
    dev_lbl.set_halign(gtk4::Align::Start);
    dev_lbl.add_css_class("hero-developer");
    info_box.append(&dev_lbl);

    let sub_lbl = gtk4::Label::new(Some(&app.summary));
    sub_lbl.set_halign(gtk4::Align::Start);
    sub_lbl.set_wrap(true);
    sub_lbl.set_lines(2);
    sub_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    sub_lbl.add_css_class("hero-subtitle");
    info_box.append(&sub_lbl);

    mid_row.append(&info_box);

    let video_preview_container = build_video_hover_widget(app);
    mid_row.append(&video_preview_container);

    card_box.append(&mid_row);

    // Sezione Azioni in Basso: Pulsante Installa/Apri + Pay-What-You-Can
    let bot_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    bot_row.set_margin_top(8);

    let action_btn = gtk4::Button::new();
    if app.installed {
        action_btn.set_label("🚀 Apri Ora");
        action_btn.add_css_class("btn-installed");
    } else {
        action_btn.set_label("⚡ Installa Istantaneamente");
        action_btn.add_css_class("btn-install");
    }

    let app_id = app.id.clone();
    let sender_c = sender.clone();
    let installed = app.installed;
    action_btn.connect_clicked(move |_| {
        if installed {
            sender_c.input(ShowcaseMsg::OpenApp(app_id.clone()));
        } else {
            sender_c.input(ShowcaseMsg::Install(app_id.clone()));
            sender_c.input(ShowcaseMsg::OpenApp(app_id.clone()));
        }
    });

    bot_row.append(&action_btn);

    let pay_box = build_pay_what_you_can_box(app, sender);
    bot_row.append(&pay_box);

    card_box.append(&bot_row);

    overlay.set_child(Some(&card_box));
    aspect_frame.set_child(Some(&overlay));

    aspect_frame
}

/// Costruisce una Card per applicazione con stile Glassmorphism usando widget pure GTK4
fn build_app_card(app: &AppItem, sender: &ComponentSender<ShowcaseModel>) -> gtk4::Box {
    let card_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    card_box.add_css_class("store-card");
    card_box.set_width_request(260);

    // Intestazione Card: Icona + Meta Info
    let top_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);

    let icon = gtk4::Image::from_icon_name(&app.icon);
    icon.set_pixel_size(48);
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

    // Visual Sandboxing Badge
    let badge = build_sandbox_badge(&app.sandbox);
    card_box.append(&badge);

    // Descrizione sintetica
    let summary_lbl = gtk4::Label::new(Some(&app.summary));
    summary_lbl.set_wrap(true);
    summary_lbl.set_lines(2);
    summary_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    summary_lbl.set_halign(gtk4::Align::Start);
    summary_lbl.add_css_class("app-card-summary");
    card_box.append(&summary_lbl);

    // Anteprima Video On Hover widget
    let video_preview = build_video_hover_widget(app);
    card_box.append(&video_preview);

    // Pay-What-You-Can (elementary OS style)
    let pay_box = build_pay_what_you_can_box(app, sender);
    card_box.append(&pay_box);

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
            sender_clone.input(ShowcaseMsg::OpenApp(app_id.clone()));
        }
    });

    card_box.append(&action_btn);

    // Controller di movimento per Hover Glow Effect
    let card_box_c = card_box.clone();
    let motion = gtk4::EventControllerMotion::new();
    motion.connect_enter(move |_, _, _| {
        card_box_c.add_css_class("card-hover-active");
    });
    let card_box_c2 = card_box.clone();
    motion.connect_leave(move |_| {
        card_box_c2.remove_css_class("card-hover-active");
    });
    card_box.add_controller(motion);

    card_box
}
