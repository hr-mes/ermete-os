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
            background-color: rgba(26, 26, 26, 0.4);
            border: 4px solid rgba(0, 255, 0, 0.5); /* YubiKey detection visual cue */
            border-radius: 16px;
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
        }
        .flowchart-box {
            background-color: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(20px);
            -gtk-backdrop-filter: blur(20px);
            border-radius: 16px;
            padding: 24px;
            color: #ffffff;
            font-weight: bold;
            font-size: 14pt;
            border: 1px solid rgba(255, 255, 255, 0.2);
            box-shadow: inset 0 0 10px rgba(255, 255, 255, 0.05);
            transition: all 0.3s ease;
        }
        .flowchart-box:hover {
            background-color: rgba(255, 255, 255, 0.2);
            border: 1px solid rgba(0, 255, 0, 0.5);
            box-shadow: 0 0 20px rgba(0, 255, 0, 0.2);
        }
        .arrow {
            color: rgba(255, 255, 255, 0.6);
            font-size: 32pt;
            font-weight: bold;
            text-shadow: 0 0 10px rgba(255, 255, 255, 0.3);
        }
        .allow {
            background-color: rgba(25, 135, 84, 0.6);
            backdrop-filter: blur(15px);
            -gtk-backdrop-filter: blur(15px);
            color: white;
            font-weight: bold;
            border-radius: 12px;
            padding: 12px 30px;
            font-size: 14pt;
            border: 1px solid rgba(25, 135, 84, 0.9);
            box-shadow: 0 4px 15px rgba(25, 135, 84, 0.3);
        }
        .allow:hover {
            background-color: rgba(25, 135, 84, 0.9);
            box-shadow: 0 6px 20px rgba(25, 135, 84, 0.5);
        }
        .deny {
            background-color: rgba(220, 53, 69, 0.6);
            backdrop-filter: blur(15px);
            -gtk-backdrop-filter: blur(15px);
            color: white;
            font-weight: bold;
            border-radius: 12px;
            padding: 12px 30px;
            font-size: 14pt;
            border: 1px solid rgba(220, 53, 69, 0.9);
            box-shadow: 0 4px 15px rgba(220, 53, 69, 0.3);
        }
        .deny:hover {
            background-color: rgba(220, 53, 69, 0.9);
            box-shadow: 0 6px 20px rgba(220, 53, 69, 0.5);
        }
        .status {
            color: #00ff00;
            font-weight: bold;
            font-size: 14pt;
            text-shadow: 0 0 10px rgba(0, 255, 0, 0.4);
        }
        .title {
            color: #ffffff;
            font-weight: bold;
            font-size: 24pt;
            text-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
        }
    ");
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
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
