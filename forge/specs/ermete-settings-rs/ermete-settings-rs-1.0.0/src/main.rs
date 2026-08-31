
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod accent_engine;
pub mod components;
pub mod crdt_store;
pub mod pages;
pub mod settings_proxy;

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
    active_page: String,
}

#[derive(Debug)]
pub enum AppMsg {
    SelectPage(String),
    RouteAi(String),
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
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 16,

                gtk4::Box {
                    set_halign: gtk4::Align::Center,
                    set_margin_top: 24,

                    #[name = "omnibox"]
                    gtk4::Entry {
                        set_placeholder_text: Some("Cosa vuoi configurare o risolvere? (es: 'Il mio audio non va')"),
                        set_width_request: 600,
                        add_css_class: "omnibox-input",
                        connect_activate[sender] => move |entry| {
                            sender.input(AppMsg::RouteAi(entry.text().to_string()));
                        }
                    }
                },

                #[name = "stack"]
                gtk4::Stack {
                    set_transition_type: gtk4::StackTransitionType::Crossfade,
                    set_hexpand: true,
                    set_vexpand: true,
                    add_css_class: "flat-canvas-container",
                    #[watch]
                    set_visible_child_name: model.active_page.as_str(),
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let initial = init.clone();
        let target_page = initial.as_deref().unwrap_or("wifi").to_string();

        let model = AppModel {
            initial_page: init,
            active_page: target_page,
        };
        let widgets = view_output!();

        ermete_style::load_glass_theme();

        // Registro delle pagine delle impostazioni
        #[allow(clippy::type_complexity)]
        let pages: &[(&str, &str, fn() -> gtk4::Box)] = &[
            ("wifi", "Wi-Fi", crate::pages::network::build_page),
            ("bluetooth", "Bluetooth", crate::pages::bluetooth::build_page),
            ("network", "Rete", crate::pages::wired::build_page),
            ("audio", "Audio", crate::pages::audio::build_page),
            ("notifications", "Notifiche", crate::pages::notifications::build_page),
            ("focus", "Focus", crate::pages::focus::build_page),
            ("general", "Generali", crate::pages::general::build_page),
            ("appearance", "Aspetto", crate::pages::appearance::build_page),
            ("layout", "Layout Switcher", crate::pages::layout_switcher::build_page),
            ("desktop", "Desktop & Dock", crate::pages::desktop::build_page),

            ("displays", "Schermi", crate::pages::displays::build_page),
            ("ecosystem", "Ecosistema", crate::pages::ecosystem::build_page),
            ("continuity", "Continuity & Handoff", crate::pages::continuity::build_page),
            ("updates", "Aggiornamenti", crate::pages::updates::build_page),
            ("battery", "Batteria", crate::pages::battery::build_page),
            ("keyboard", "Tastiera", crate::pages::keyboard::build_page),
            ("mouse", "Mouse & Trackpad", crate::pages::mouse::build_page),
            ("accounts", "Account", crate::pages::accounts::build_page),
            ("privacy", "Privacy & Sicurezza", crate::pages::privacy::build_page),
            ("a11y", "Accessibilità", crate::pages::a11y::build_page),
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

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        match msg {
            AppMsg::SelectPage(page_id) => {
                self.active_page = page_id;
            }
            AppMsg::RouteAi(query) => {
                // Chiamata all'AI Daemon per il natural language routing e propagazione CRDT
                let q_clone = query.clone();
                relm4::spawn_local(async move {
                    let _ = crate::crdt_store::update_setting_crdt("ai_routing_intent", &q_clone).await;
                });

                let q = query.to_lowercase();
                if q.contains("audio") || q.contains("suono") || q.contains("volume") {
                    sender.input(AppMsg::SelectPage("audio".to_string()));
                } else if q.contains("wifi") || q.contains("wi-fi") || q.contains("internet") {
                    sender.input(AppMsg::SelectPage("wifi".to_string()));
                } else if q.contains("bluetooth") {
                    sender.input(AppMsg::SelectPage("bluetooth".to_string()));
                } else if q.contains("schermo") || q.contains("display") || q.contains("monit") {
                    sender.input(AppMsg::SelectPage("displays".to_string()));
                } else if q.contains("batteria") || q.contains("power") || q.contains("energia") {
                    sender.input(AppMsg::SelectPage("battery".to_string()));
                } else if q.contains("aspetto") || q.contains("tema") || q.contains("dark") {
                    sender.input(AppMsg::SelectPage("appearance".to_string()));
                } else if q.contains("layout") || q.contains("zorin") || q.contains("dock") || q.contains("taskbar") {
                    sender.input(AppMsg::SelectPage("layout".to_string()));
                } else if q.contains("continuity") || q.contains("handoff") || q.contains("clipboard") || q.contains("appunti") {
                    sender.input(AppMsg::SelectPage("continuity".to_string()));
                } else if q.contains("accessib") || q.contains("a11y") || q.contains("sottotitol") || q.contains("daltonis") || q.contains("tts") || q.contains("voce") || q.contains("screen reader") {
                    sender.input(AppMsg::SelectPage("a11y".to_string()));
                }

            }
        }
    }
}

fn main() {
    // Forza il renderer GTK4 NGL (New GL) ad altissime prestazioni / Vulkan e backend puramente Wayland
    std::env::set_var("GSK_RENDERER", "ngl");
    std::env::set_var("GDK_BACKEND", "wayland");
    // Disabilita lo scaling X11 frazionario per evitare blur
    std::env::set_var("GDK_SCALE", "1");

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

