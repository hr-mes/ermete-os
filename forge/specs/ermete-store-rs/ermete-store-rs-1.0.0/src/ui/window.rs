use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, Controller, RelmApp, SimpleComponent};

use crate::ui::showcase::{ShowcaseModel, ShowcaseMsg};

/// CSS embedded per lo stile Glassmorphism elegante dello Store
const STORE_GLASS_CSS: &str = r#"
.window-glass {
    background-color: rgba(18, 18, 26, 0.95);
    color: #e2e8f0;
}

.store-sidebar {
    background-color: rgba(26, 27, 38, 0.65);
    border-radius: 16px;
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

.store-brand-title {
    font-weight: 800;
    font-size: 18px;
    color: #ffffff;
}

.nav-btn {
    background: transparent;
    border: none;
    border-radius: 10px;
    padding: 10px 14px;
    color: #cbd5e1;
    font-weight: 500;
    font-size: 14px;
    transition: all 0.2s ease-in-out;
}

.nav-btn:hover {
    background-color: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

.nav-btn:active, .nav-btn:checked {
    background-color: rgba(59, 130, 246, 0.25);
    color: #60a5fa;
    border: 1px solid rgba(96, 165, 250, 0.3);
}

.sidebar-divider {
    background-color: rgba(255, 255, 255, 0.06);
    margin: 12px 4px;
}

.sidebar-sep {
    background-color: rgba(255, 255, 255, 0.08);
    margin: 8px 0;
}

.hero-banner {
    background: linear-gradient(135deg, rgba(37, 99, 235, 0.3) 0%, rgba(147, 51, 234, 0.25) 100%);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 20px;
    padding: 28px;
    backdrop-filter: blur(16px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.hero-title {
    font-size: 24px;
    font-weight: 800;
    color: #ffffff;
}

.hero-subtitle {
    font-size: 14px;
    color: #94a3b8;
}

.hero-btn-primary {
    background-color: #2563eb;
    color: #ffffff;
    border-radius: 10px;
    padding: 8px 16px;
    font-weight: 600;
    border: none;
}

.hero-btn-primary:hover {
    background-color: #1d4ed8;
}

.hero-btn-secondary {
    background-color: rgba(255, 255, 255, 0.1);
    color: #ffffff;
    border-radius: 10px;
    padding: 8px 16px;
    font-weight: 600;
    border: 1px solid rgba(255, 255, 255, 0.15);
}

.hero-btn-secondary:hover {
    background-color: rgba(255, 255, 255, 0.18);
}

.glass-search {
    background-color: rgba(30, 41, 59, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: #f8fafc;
    padding: 8px 14px;
}

.pill-btn {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 20px;
    color: #cbd5e1;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 500;
}

.pill-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

.section-heading {
    font-size: 18px;
    font-weight: 700;
    color: #f1f5f9;
    margin-top: 8px;
}

.store-card {
    background: rgba(30, 41, 59, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 16px;
    padding: 16px;
    backdrop-filter: blur(12px);
    transition: transform 0.2s ease, background 0.2s ease, box-shadow 0.2s ease;
}

.store-card:hover {
    background: rgba(51, 65, 85, 0.6);
    border-color: rgba(96, 165, 250, 0.4);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
}

.app-card-title {
    font-weight: 700;
    font-size: 15px;
    color: #ffffff;
}

.app-card-category {
    font-size: 12px;
    color: #94a3b8;
}

.app-card-rating {
    font-size: 12px;
    color: #fbbf24;
    font-weight: 600;
}

.app-card-summary {
    font-size: 13px;
    color: #cbd5e1;
}

.btn-install {
    background-color: #3b82f6;
    color: #ffffff;
    border-radius: 8px;
    border: none;
    padding: 6px;
    font-weight: 600;
    font-size: 13px;
}

.btn-install:hover {
    background-color: #2563eb;
}

.btn-installed {
    background-color: rgba(255, 255, 255, 0.1);
    color: #34d399;
    border-radius: 8px;
    border: 1px solid rgba(52, 211, 153, 0.3);
    padding: 6px;
    font-weight: 600;
    font-size: 13px;
}

.placeholder-box {
    background-color: rgba(30, 41, 59, 0.3);
    border-radius: 16px;
    border: 1px dashed rgba(255, 255, 255, 0.15);
    margin: 32px;
    padding: 48px;
}

.placeholder-title {
    font-size: 22px;
    font-weight: 700;
    color: #f8fafc;
}

.placeholder-sub {
    font-size: 14px;
    color: #94a3b8;
}
"#;

/// Carica le regole CSS personalizzate nel display predefinito GTK4 (senza unwrap)
fn load_glass_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(STORE_GLASS_CSS);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Modello principale della finestra Root per Ermete Store
pub struct AppModel {
    showcase: Controller<ShowcaseModel>,
    active_page: String,
}

/// Messaggi dell'applicazione principale
#[derive(Debug)]
pub enum AppMsg {
    SelectPage(String),
    Quit,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Ermete Store"),
            set_default_width: 1180,
            set_default_height: 780,
            add_css_class: "window-glass",

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: true,
                set_vexpand: true,

                // Sidebar laterale di navigazione stile Windows 11 Store
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_width_request: 220,
                    set_spacing: 8,
                    set_margin_start: 12,
                    set_margin_end: 8,
                    set_margin_top: 16,
                    set_margin_bottom: 16,
                    add_css_class: "store-sidebar",

                    // Brand & Logo dello Store
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 12,
                        set_margin_start: 12,
                        set_margin_top: 8,
                        set_margin_bottom: 16,

                        gtk4::Image {
                            set_icon_name: Some("system-software-install"),
                            set_pixel_size: 28,
                            add_css_class: "store-logo",
                        },

                        gtk4::Label {
                            set_label: "Ermete Store",
                            add_css_class: "store-brand-title",
                            set_halign: gtk4::Align::Start,
                        }
                    },

                    // Pulsanti di Navigazione
                    gtk4::Button {
                        set_label: "🏠 Home",
                        add_css_class: "nav-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::SelectPage("showcase".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "📦 Applicazioni",
                        add_css_class: "nav-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::SelectPage("apps".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "🎮 Giochi",
                        add_css_class: "nav-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::SelectPage("gaming".to_string()));
                        }
                    },
                    gtk4::Button {
                        set_label: "📚 Libreria",
                        add_css_class: "nav-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::SelectPage("library".to_string()));
                        }
                    },

                    // Spaziatore verticale per allineare le Impostazioni in basso
                    gtk4::Box {
                        set_vexpand: true,
                    },

                    gtk4::Separator {
                        set_orientation: gtk4::Orientation::Horizontal,
                        add_css_class: "sidebar-sep",
                    },

                    gtk4::Button {
                        set_label: "⚙️ Impostazioni",
                        add_css_class: "nav-btn",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::SelectPage("settings".to_string()));
                        }
                    }
                },

                // Divisore di layout
                gtk4::Separator {
                    set_orientation: gtk4::Orientation::Vertical,
                    add_css_class: "sidebar-divider",
                },

                // Stack Centrale per il contenuto delle sezioni
                #[name = "main_stack"]
                gtk4::Stack {
                    set_transition_type: gtk4::StackTransitionType::SlideLeftRight,
                    set_transition_duration: 220,
                    set_hexpand: true,
                    set_vexpand: true,
                    add_css_class: "central-stack",
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Applica le regole CSS Glassmorphism
        load_glass_css();

        // Inizializza il sub-componente Showcase (Homepage)
        let showcase_controller = ShowcaseModel::builder().launch(()).detach();

        let model = AppModel {
            showcase: showcase_controller,
            active_page: "showcase".to_string(),
        };

        let widgets = view_output!();

        // Inserimento del widget Showcase nello Stack
        widgets.main_stack.add_named(model.showcase.widget(), Some("showcase"));

        // Pagine placeholder stilizzate per le altre sezioni dello Store
        widgets.main_stack.add_named(
            &build_placeholder_page("📦 Applicazioni", "Catalogo completo di software Flatpak, OCI ed EOPKG."),
            Some("apps"),
        );
        widgets.main_stack.add_named(
            &build_placeholder_page("🎮 Giochi & Hub Gaming", "Giochi nativi, Steam, Lutris ed emulatori su Ermete OS."),
            Some("gaming"),
        );
        widgets.main_stack.add_named(
            &build_placeholder_page("📚 Libreria Software", "Gestione delle app installate, dipendenze e aggiornamenti di sistema."),
            Some("library"),
        );
        widgets.main_stack.add_named(
            &build_placeholder_page("⚙️ Impostazioni Store", "Configurazione repository remoti, aggiornamenti automatici e preferenze."),
            Some("settings"),
        );

        widgets.main_stack.set_visible_child_name("showcase");

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SelectPage(page_id) => {
                self.active_page = page_id;
            }
            AppMsg::Quit => {
                relm4::main_application().quit();
            }
        }
    }
}

/// Helper per costruire pagine placeholder stilizzate con GTK4 pure (Glassmorphism)
fn build_placeholder_page(title: &str, subtitle: &str) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    container.set_halign(gtk4::Align::Center);
    container.set_valign(gtk4::Align::Center);
    container.add_css_class("placeholder-box");

    let title_label = gtk4::Label::new(Some(title));
    title_label.add_css_class("placeholder-title");
    container.append(&title_label);

    let sub_label = gtk4::Label::new(Some(subtitle));
    sub_label.add_css_class("placeholder-sub");
    container.append(&sub_label);

    container
}

/// Entry point helper per avviare l'applicazione `ermete-store-rs` UI
pub fn run_app() {
    let app = RelmApp::new("os.ermete.Store");
    app.run::<AppModel>(());
}
