use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::path::Path;

pub fn build_ui(app: &Application, app_path: &str) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gatekeeper")
        .css_classes(["gatekeeper-window"])
        .default_width(480)
        .build();

    window.init_layer_shell();
    window.set_namespace("gatekeeper");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    window.auto_exclusive_zone_enable();
    
    // Center it
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Bottom, 0);
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .css_classes(["gatekeeper-card"])
        .build();

    // Warning icon
    let icon = Image::builder()
        .icon_name("security-high-symbolic")
        .pixel_size(64)
        .css_classes(["gatekeeper-icon"])
        .halign(Align::Center)
        .build();
    vbox.append(&icon);

    let path_obj = Path::new(app_path);
    let filename = path_obj.file_name().unwrap_or_default().to_string_lossy();

    let title = Label::builder()
        .label(&format!("Esecuzione di \"{}\" Bloccata", filename))
        .css_classes(["gatekeeper-title"])
        .halign(Align::Center)
        .wrap(true)
        .build();
    vbox.append(&title);

    let desc_text = format!("Il file \n{}\nè stato scaricato da Internet e non è verificato.\n\nEseguire file non attendibili può compromettere la sicurezza del sistema o dei tuoi dati personali.", app_path);
    let desc = Label::builder()
        .label(&desc_text)
        .css_classes(["gatekeeper-desc"])
        .halign(Align::Center)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();
    vbox.append(&desc);

    let hbox = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .halign(Align::Center)
        .margin_top(24)
        .build();

    let btn_cancel = Button::builder()
        .label("Annulla")
        .css_classes(["suggested-action", "gatekeeper-btn"])
        .build();
        
    let app_cancel = app.clone();
    btn_cancel.connect_clicked(move |_btn| {
        // Exit with code 1 to deny
        std::process::exit_code(1);
        app_cancel.quit();
    });

    let btn_approve = Button::builder()
        .label("Sblocca ed Esegui")
        .css_classes(["destructive-action", "gatekeeper-btn"])
        .build();

    let app_approve = app.clone();
    btn_approve.connect_clicked(move |_btn| {
        // Future: PAM Authentication here!
        // For now, exit 0 to approve
        std::process::exit_code(0);
        app_approve.quit();
    });

    hbox.append(&btn_cancel);
    hbox.append(&btn_approve);
    vbox.append(&hbox);

    window.set_child(Some(&vbox));

    // Basic CSS
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        .gatekeeper-window { background-color: rgba(0, 0, 0, 0.5); }
        .gatekeeper-card { background-color: #1e1e2e; border-radius: 16px; border: 1px solid #f38ba8; }
        .gatekeeper-title { font-size: 20px; font-weight: bold; color: #f38ba8; }
        .gatekeeper-desc { font-size: 14px; color: #a6adc8; }
        .gatekeeper-icon { color: #f38ba8; }
        .gatekeeper-btn { padding: 10px 24px; font-weight: bold; border-radius: 8px; }
    ");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.present();
}
