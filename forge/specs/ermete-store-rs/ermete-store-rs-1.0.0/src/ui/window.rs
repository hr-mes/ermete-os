use gtk4::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp, SimpleComponent};

use crate::ui::showcase::ShowcaseModel;

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
                    set_margin_top: 16,
                    set_margin_bottom: 16,
                    set_margin_end: 16,
                    add_css_class: "central-stack",
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Applica le regole CSS Glassmorphism
        ermete_style::load_glass_theme();

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
