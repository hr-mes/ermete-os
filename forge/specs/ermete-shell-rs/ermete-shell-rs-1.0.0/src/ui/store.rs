use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Image, Label, Orientation, ScrolledWindow, FlowBox, HeaderBar, Spinner};
use glib::clone;
use serde_json::Value;

#[zbus::proxy(
    interface = "os.ermete.Store",
    default_service = "os.ermete.Store",
    default_path = "/os/ermete/Store"
)]
trait Store {
    async fn search_apps(&self, query: &str) -> zbus::Result<String>;
    async fn install_app(&self, app_id: &str) -> zbus::Result<()>;
}

pub fn show_store_modal(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("App Store Universale")
        .default_width(900)
        .default_height(600)
        .css_classes(["app-store-window"])
        .build();

    let header = HeaderBar::new();
    window.set_titlebar(Some(&header));

    let search_entry = Entry::builder()
        .placeholder_text("Cerca app (es. Firefox, GIMP, Spotify)...")
        .width_request(400)
        .build();
    header.set_title_widget(Some(&search_entry));

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    let flowbox = FlowBox::builder()
        .valign(Align::Start)
        .max_children_per_line(4)
        .min_children_per_line(1)
        .selection_mode(gtk4::SelectionMode::None)
        .row_spacing(16)
        .column_spacing(16)
        .build();

    let spinner = Spinner::builder()
        .valign(Align::Center)
        .halign(Align::Center)
        .hexpand(true)
        .vexpand(true)
        .visible(false)
        .build();

    scroll.set_child(Some(&flowbox));
    
    let overlay_box = GtkBox::builder().orientation(Orientation::Vertical).build();
    overlay_box.append(&scroll);
    overlay_box.append(&spinner);

    main_box.append(&overlay_box);
    window.set_child(Some(&main_box));

    // Async search logic
    let flowbox_clone = flowbox.clone();
    let spinner_clone = spinner.clone();

    search_entry.connect_activate(clone!(@weak window => move |entry| {
        let query = entry.text().to_string();
        if query.is_empty() { return; }

        let flowbox = flowbox_clone.clone();
        let spinner = spinner_clone.clone();

        spinner.start();
        spinner.set_visible(true);
        flowbox.set_visible(false);

        while let Some(child) = flowbox.first_child() {
            flowbox.remove(&child);
        }

        glib::spawn_future_local(async move {
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(proxy) = StoreProxy::new(&conn).await {
                    if let Ok(res_str) = proxy.search_apps(&query).await {
                        if let Ok(apps) = serde_json::from_str::<Vec<Value>>(&res_str) {
                            for app in apps {
                                if let (Some(id), Some(name), Some(summary)) = (
                                    app.get("id").and_then(|v| v.as_str()),
                                    app.get("name").and_then(|v| v.as_str()),
                                    app.get("summary").and_then(|v| v.as_str()),
                                ) {
                                    let card = create_app_card(id, name, summary, &conn);
                                    flowbox.append(&card);
                                }
                            }
                        }
                    }
                }
            }
            spinner.stop();
            spinner.set_visible(false);
            flowbox.set_visible(true);
        });
    }));

    window.present();
}

fn create_app_card(id: &str, name: &str, summary: &str, conn: &zbus::Connection) -> GtkBox {
    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["store-app-card"])
        .width_request(200)
        .spacing(8)
        .build();

    let img = Image::builder()
        .icon_name("application-x-executable") // Placeholder
        .pixel_size(64)
        .halign(Align::Center)
        .margin_top(16)
        .build();

    let title_lbl = Label::builder()
        .label(name)
        .css_classes(["store-app-title"])
        .halign(Align::Center)
        .build();

    let desc_lbl = Label::builder()
        .label(summary)
        .css_classes(["store-app-desc"])
        .halign(Align::Center)
        .wrap(true)
        .wrap_mode(gtk4::pango::WrapMode::Word)
        .lines(2)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();

    let install_btn = Button::builder()
        .label("Installa (Ottieni)")
        .css_classes(["suggested-action", "store-install-btn"])
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let id_string = id.to_string();
    let conn_clone = conn.clone();
    
    install_btn.connect_clicked(move |btn| {
        btn.set_label("Installazione...");
        btn.set_sensitive(false);
        let app_id = id_string.clone();
        let btn_clone = btn.clone();
        let conn = conn_clone.clone();

        glib::spawn_future_local(async move {
            if let Ok(proxy) = StoreProxy::new(&conn).await {
                match proxy.install_app(&app_id).await {
                    Ok(_) => {
                        btn_clone.set_label("Apri");
                        btn_clone.set_sensitive(true);
                        btn_clone.remove_css_class("suggested-action");
                    }
                    Err(_) => {
                        btn_clone.set_label("Errore");
                        btn_clone.set_sensitive(true);
                    }
                }
            }
        });
    });

    card.append(&img);
    card.append(&title_lbl);
    card.append(&desc_lbl);
    card.append(&install_btn);

    card
}
