pub mod components;
pub mod niri_client;
pub mod pages;
pub mod settings_proxy;
pub mod style;

use gtk4::prelude::*;
use relm4::{ComponentParts, RelmApp, SimpleComponent};

/// Helper async per la connessione DBus Session (Thread-safe, zbus usa una connessione condivisa interna)
pub async fn get_connection() -> Result<zbus::Connection, zbus::Error> {
    zbus::Connection::session().await
}

/// Helper async per la connessione DBus System
pub async fn get_system_connection() -> Result<zbus::Connection, zbus::Error> {
    zbus::Connection::system().await
}

/// Shell di base per Ermete Settings
pub struct AppModel {
    initial_page: Option<String>,
}

#[derive(Debug)]
pub enum AppMsg {
    SelectPage(String),
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = Option<String>;
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Impostazioni di Sistema"),
            set_default_width: 1024,
            set_default_height: 720,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,

                #[name = "sidebar"]
                gtk4::StackSidebar {
                    set_width_request: 240,
                    add_css_class: "sidebar-container",
                },

                #[name = "stack"]
                gtk4::Stack {
                    set_transition_type: gtk4::StackTransitionType::Crossfade,
                    set_hexpand: true,
                    set_vexpand: true,
                    add_css_class: "stack-container",
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            initial_page: init.clone(),
        };
        let widgets = view_output!();

        style::load_global_css();

        // Collega la StackSidebar alla Stack GTK4
        widgets.sidebar.set_stack(&widgets.stack);

        // Registro delle pagine delle impostazioni
        let pages: &[(&str, &str, fn() -> gtk4::Box)] = &[
            ("wifi", "Wi-Fi", crate::pages::network::build_page),
            ("bluetooth", "Bluetooth", crate::pages::bluetooth::build_page),
            ("network", "Rete", crate::pages::wired::build_page),
            ("audio", "Audio", crate::pages::audio::build_page),
            ("notifications", "Notifiche", crate::pages::notifications::build_page),
            ("focus", "Focus", crate::pages::focus::build_page),
            ("general", "Generali", crate::pages::general::build_page),
            ("appearance", "Aspetto", crate::pages::appearance::build_page),
            ("desktop", "Desktop & Dock", crate::pages::desktop::build_page),
            ("displays", "Schermi", crate::pages::displays::build_page),
            ("ecosystem", "Ecosistema", crate::pages::ecosystem::build_page),
            ("updates", "Aggiornamenti", crate::pages::updates::build_page),
            ("battery", "Batteria", crate::pages::battery::build_page),
            ("keyboard", "Tastiera", crate::pages::keyboard::build_page),
            ("mouse", "Mouse & Trackpad", crate::pages::mouse::build_page),
            ("accounts", "Account", crate::pages::accounts::build_page),
            ("privacy", "Privacy & Sicurezza", crate::pages::privacy::build_page),
        ];

        let target_page = model.initial_page.as_deref().unwrap_or("wifi");

        // Lazy Loading Architecture: create wrapper containers for all tabs,
        // build only active initial target page upfront
        for (id, title, build_fn) in pages {
            let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            container.set_hexpand(true);
            container.set_vexpand(true);

            if *id == target_page {
                let page_widget = build_fn();
                container.append(&page_widget);
            }
            widgets.stack.add_titled(&container, Some(id), title);
        }

        // Connect lazy page builder on stack tab switch
        widgets.stack.connect_visible_child_name_notify(move |stack| {
            if let Some(name) = stack.visible_child_name() {
                if let Some((_, _, build_fn)) = pages.iter().find(|(id, _, _)| *id == name.as_str()) {
                    if let Some(child) = stack.child_by_name(&name) {
                        if let Ok(container) = child.downcast::<gtk4::Box>() {
                            if container.first_child().is_none() {
                                let real_page = build_fn();
                                container.append(&real_page);
                            }
                        }
                    }
                }
            }
        });

        // Selezione pagina iniziale da argomenti CLI (--page=...)
        if let Some(ref page_id) = model.initial_page {
            widgets.stack.set_visible_child_name(page_id);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match msg {
            AppMsg::SelectPage(_page_id) => {
                // Gestione messaggi dinamici di cambio pagina se necessari
            }
        }
    }
}

fn main() {
    let mut page_id = None;
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if let Some(id) = arg.strip_prefix("--page=") {
            page_id = Some(id.to_string());
        } else if arg == "--page" {
            if let Some(next_arg) = iter.next() {
                page_id = Some(next_arg.clone());
            }
        }
    }

    let app = RelmApp::new("os.ermete.Settings");
    app.run::<AppModel>(page_id);
}
