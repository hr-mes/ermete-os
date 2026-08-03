use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Label, Orientation, Align, CssProvider};

const APP_ID: &str = "org.ermete.GatekeeperUI";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(build_ui);

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data("
        window {
            background-color: #1a1a1a;
            border: 4px solid #00ff00; /* YubiKey detection visual cue */
            border-radius: 12px;
        }
        .flowchart-box {
            background-color: #2a2a2a;
            border-radius: 12px;
            padding: 20px;
            color: #ffffff;
            font-weight: bold;
            font-size: 14pt;
            border: 1px solid #444444;
        }
        .arrow {
            color: #888888;
            font-size: 28pt;
            font-weight: bold;
        }
        .allow {
            background-color: #198754;
            color: white;
            font-weight: bold;
            border-radius: 8px;
            padding: 12px 30px;
            font-size: 14pt;
        }
        .allow:hover {
            background-color: #157347;
        }
        .deny {
            background-color: #dc3545;
            color: white;
            font-weight: bold;
            border-radius: 8px;
            padding: 12px 30px;
            font-size: 14pt;
        }
        .deny:hover {
            background-color: #bb2d3b;
        }
        .status {
            color: #00ff00;
            font-weight: bold;
            font-size: 12pt;
        }
        .title {
            color: #ffffff;
            font-weight: bold;
            font-size: 20pt;
        }
    ");
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(30)
        .margin_top(40)
        .margin_bottom(40)
        .margin_start(50)
        .margin_end(50)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let title = Label::builder()
        .label("Ermete Gatekeeper")
        .build();
    title.add_css_class("title");
    main_box.append(&title);

    // Flowchart
    let flowchart_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(20)
        .halign(Align::Center)
        .build();

    let app_box = Label::builder()
        .label("App Mittente\n(es. Terminale)")
        .justify(gtk::Justification::Center)
        .build();
    app_box.add_css_class("flowchart-box");

    let arrow1 = Label::builder()
        .label("➔")
        .build();
    arrow1.add_css_class("arrow");

    let action_box = Label::builder()
        .label("Azione Pericolosa\n(Installazione Pacchetto)")
        .justify(gtk::Justification::Center)
        .build();
    action_box.add_css_class("flowchart-box");

    let arrow2 = Label::builder()
        .label("➔")
        .build();
    arrow2.add_css_class("arrow");

    let os_box = Label::builder()
        .label("Sistema OS\n(Ermete Core)")
        .justify(gtk::Justification::Center)
        .build();
    os_box.add_css_class("flowchart-box");

    flowchart_box.append(&app_box);
    flowchart_box.append(&arrow1);
    flowchart_box.append(&action_box);
    flowchart_box.append(&arrow2);
    flowchart_box.append(&os_box);

    main_box.append(&flowchart_box);

    let status_label = Label::builder()
        .label("🔑 YubiKey rilevata. Polkit in attesa di approvazione.")
        .build();
    status_label.add_css_class("status");
    main_box.append(&status_label);

    let button_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(24)
        .halign(Align::Center)
        .build();

    let btn_deny = Button::builder()
        .label("Nega")
        .build();
    btn_deny.add_css_class("deny");
    
    let btn_allow = Button::builder()
        .label("Consenti (Polkit)")
        .build();
    btn_allow.add_css_class("allow");

    button_box.append(&btn_deny);
    button_box.append(&btn_allow);

    main_box.append(&button_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Gatekeeper")
        .default_width(750)
        .default_height(450)
        .child(&main_box)
        .build();

    window.present();
}
