pub mod pages;
pub mod niri_client;
pub mod settings_proxy;
use gtk4::prelude::*;
pub mod style;
use gtk4::{
    gio, Application, ApplicationWindow, Box as GtkBox, Orientation, Label, Stack, Align, ListBox,
    ListBoxRow, Image, SearchEntry,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    pub static DBUS_CONN: RefCell<Option<zbus::Connection>> = RefCell::new(None);
}

pub async fn get_connection() -> Result<zbus::Connection, zbus::Error> {
    if let Some(conn) = DBUS_CONN.with(|c| c.borrow().clone()) {
        return Ok(conn);
    }
    let conn = zbus::Connection::session().await?;
    DBUS_CONN.with(|c| *c.borrow_mut() = Some(conn.clone()));
    Ok(conn)
}


pub struct AppModel {}

#[relm4::component]
impl relm4::SimpleComponent for AppModel {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Impostazioni di Sistema"),
            set_default_width: 960,
            set_default_height: 700,
            
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                
                gtk4::Label {
                    set_label: "Relm4 Scaffold Active",
                    add_css_class: "title-1",
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppModel {};
        let widgets = view_output!();
        relm4::ComponentParts { model, widgets }
    }
}

fn main() {
    style::load_global_css();
    let app = relm4::RelmApp::new("os.ermete.Settings");
    app.run::<AppModel>(());
}
