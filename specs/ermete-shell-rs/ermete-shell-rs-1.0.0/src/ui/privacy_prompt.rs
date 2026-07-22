use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn build_ui(app: &Application, request_info: &str) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Richiesta Permessi")
        .css_classes(["privacy-window"])
        .default_width(420)
        .build();

    window.init_layer_shell();
    window.set_namespace("privacy");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    window.auto_exclusive_zone_enable();
    
    // Top right corner like macOS notification
    window.set_margin(Edge::Top, 16);
    window.set_margin(Edge::Right, 16);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .css_classes(["privacy-card"])
        .build();

    let parts: Vec<&str> = request_info.splitn(2, ':').collect();
    let resource = parts.get(0).unwrap_or(&"Risorsa");
    let app_id = parts.get(1).unwrap_or(&"Applicazione Sconosciuta");

    let icon_name = match *resource {
        "Camera" => "camera-web-symbolic",
        "Microphone" => "audio-input-microphone-symbolic",
        "ScreenCast" => "video-display-symbolic",
        "Location" => "mark-location-symbolic",
        _ => "dialog-question-symbolic",
    };

    let icon = Image::builder()
        .icon_name(icon_name)
        .pixel_size(48)
        .css_classes(["privacy-icon"])
        .halign(Align::Center)
        .build();
    vbox.append(&icon);

    let title = Label::builder()
        .label(&format!("\"{}\" desidera accedere a {}", app_id, resource))
        .css_classes(["privacy-title"])
        .halign(Align::Center)
        .wrap(true)
        .build();
    vbox.append(&title);

    let desc = Label::builder()
        .label("Concedendo l'accesso, l'applicazione potrà utilizzare questa risorsa fino alla sua chiusura.")
        .css_classes(["privacy-desc"])
        .halign(Align::Center)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();
    vbox.append(&desc);

    let hbox = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .halign(Align::Center)
        .margin_top(16)
        .build();

    let btn_cancel = Button::builder()
        .label("Nega")
        .css_classes(["privacy-btn"])
        .build();
        
    let app_cancel = app.clone();
    btn_cancel.connect_clicked(move |_btn| {
        std::process::exit(1);
        app_cancel.quit();
    });

    let btn_approve = Button::builder()
        .label("Consenti")
        .css_classes(["suggested-action", "privacy-btn"])
        .build();

    let app_approve = app.clone();
    btn_approve.connect_clicked(move |_btn| {
        std::process::exit(0);
        app_approve.quit();
    });

    hbox.append(&btn_cancel);
    hbox.append(&btn_approve);
    vbox.append(&hbox);

    window.set_child(Some(&vbox));

    // Basic CSS
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        .privacy-window { background-color: transparent; }
        .privacy-card { 
            background: rgba(24, 27, 36, 0.75); 
            border-radius: 24px; 
            border: 1px solid rgba(255,255,255,0.25); 
            box-shadow: 0 24px 64px rgba(0,0,0,0.8), inset 0 1px 1px rgba(255,255,255,0.15);
            padding: 32px;
        }
        .privacy-title { 
            font-family: 'Inter', 'SF Pro Display', sans-serif;
            font-size: 18px; 
            font-weight: 800; 
            color: #ffffff; 
            margin-bottom: 8px;
            text-shadow: 0 2px 8px rgba(0,0,0,0.4);
        }
        .privacy-desc { 
            font-family: 'Inter', 'SF Pro Text', sans-serif;
            font-size: 14px; 
            color: rgba(255,255,255,0.8); 
            line-height: 1.4;
        }
        .privacy-icon { 
            color: #f9e2af; 
            margin-bottom: 16px;
            text-shadow: 0 4px 12px rgba(249, 226, 175, 0.4);
        }
        .privacy-btn { 
            font-family: 'Inter', 'SF Pro Text', sans-serif;
            padding: 10px 28px; 
            font-weight: 700; 
            border-radius: 999px; 
            font-size: 14px;
            transition: all 0.3s ease;
            box-shadow: 0 4px 16px rgba(0,0,0,0.3);
        }
        .privacy-btn:hover {
            background-color: rgba(255, 255, 255, 0.2);
        }
        .suggested-action.privacy-btn {
            background-color: rgba(110, 168, 254, 0.85);
            color: #000000;
            border: 1px solid rgba(110, 168, 254, 1.0);
        }
        .suggested-action.privacy-btn:hover {
            background-color: rgba(110, 168, 254, 1.0);
            box-shadow: 0 6px 20px rgba(110, 168, 254, 0.5);
        }
    ");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}
