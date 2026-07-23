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
    pub static MAIN_WINDOW: RefCell<Option<ApplicationWindow>> = RefCell::new(None);
    pub static MAIN_STACK: RefCell<Option<Stack>> = RefCell::new(None);
    pub static MAIN_LIST_BOX: RefCell<Option<ListBox>> = RefCell::new(None);
}

pub async fn get_connection() -> Result<zbus::Connection, zbus::Error> {
    if let Some(conn) = DBUS_CONN.with(|c| c.borrow().clone()) {
        return Ok(conn);
    }
    let conn = zbus::Connection::session().await?;
    DBUS_CONN.with(|c| *c.borrow_mut() = Some(conn.clone()));
    Ok(conn)
}

fn category_matches_query(id: &str, title: &str, query: &str) -> bool {
    let q = query.to_lowercase();
    let title_lower = title.to_lowercase();
    let id_lower = id.to_lowercase();
    if title_lower.contains(&q) || id_lower.contains(&q) {
        return true;
    }
    let keywords = match id {
        "wifi" => "wireless internet wlan rete network",
        "bluetooth" => "auricolari mouse cuffie dispositivi pair",
        "network" => "ethernet cavo lan proxy dns vpn rete wired",
        "audio" => "volume suoni microfono altoparlanti cuffie output input sound",
        "notifications" => "notifiche avvisi disturbare notifications alerts",
        "focus" => "notte sonno quiete disturbare do not disturb",
        "general" => "generali sistema info lingua data ora general about",
        "appearance" => "aspetto tema scuro chiaro colori sfondo wallpaper appearance theme dark",
        "desktop" => "dock icone barra pannello workspace",
        "displays" => "schermi monitor display risoluzione scaling",
        "battery" => "batteria energia risparmio alimentazione battery power",
        "keyboard" => "tastiera layout lingua scorciatoie keyboard shortcuts input",
        "mouse" => "mouse trackpad touchpad puntatore velocità scorrimento",
        "accounts" => "account utente password profilo users login",
        "privacy" => "privacy sicurezza permessi lock security",
        _ => "",
    };
    keywords.contains(&q)
}

fn switch_to_page(page_id: &str) {
    let mut found = false;
    MAIN_LIST_BOX.with(|lb| {
        if let Some(list_box) = lb.borrow().as_ref() {
            let mut i = 0;
            while let Some(row) = list_box.row_at_index(i) {
                if row.widget_name() == page_id {
                    list_box.select_row(Some(&row));
                    found = true;
                    break;
                }
                i += 1;
            }
        }
    });
    if found {
        MAIN_STACK.with(|s| {
            if let Some(stack) = s.borrow().as_ref() {
                stack.set_visible_child_name(page_id);
            }
        });
    }
}

struct AppModel {
    page_to_open: Option<String>,
}

#[relm4::component]
impl relm4::SimpleComponent for AppModel {
    type Init = Option<String>;
    type Input = ();
    type Output = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Impostazioni di Sistema"),
            set_default_width: 1024,
            set_default_height: 768,
            
            #[name="main_box"]
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppModel {
            page_to_open: init,
        };
        let widgets = view_output!();

        style::load_global_css();

        let main_box = &widgets.main_box;

        let sidebar_container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .width_request(260)
            .css_classes(["sidebar-container"])
            .build();

        let profile_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(16)
            .margin_bottom(12)
            .margin_start(16)
            .margin_end(16)
            .build();
        let profile_img = Image::builder().icon_name("avatar-default").pixel_size(48).build();
        let profile_vbox = GtkBox::builder().orientation(Orientation::Vertical).valign(Align::Center).build();
        let profile_name = Label::builder().label("Ermete User").css_classes(["profile-name"]).halign(Align::Start).build();
        let profile_role = Label::builder().label("Amministratore").css_classes(["profile-role"]).halign(Align::Start).build();
        profile_vbox.append(&profile_name);
        profile_vbox.append(&profile_role);
        profile_box.append(&profile_img);
        profile_box.append(&profile_vbox);
        sidebar_container.append(&profile_box);

        let search_entry = SearchEntry::builder()
            .placeholder_text("Cerca impostazione...")
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(8)
            .css_classes(["sidebar-search"])
            .build();
        sidebar_container.append(&search_entry);

        let list_box = ListBox::builder().css_classes(["sidebar-list"]).build();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        let stack = Stack::builder()
            .transition_type(gtk4::StackTransitionType::Crossfade)
            .hexpand(true)
            .vexpand(true)
            .css_classes(["stack-container"])
            .build();

        let categories = vec![
            ("Wi-Fi", "network-wireless", "wifi"),
            ("Bluetooth", "bluetooth-active", "bluetooth"),
            ("Rete", "network-wired", "network"),
            ("SEPARATOR", "", ""),
            ("Audio", "audio-volume-high", "audio"),
            ("Notifiche", "preferences-system-notifications", "notifications"),
            ("Focus", "weather-clear-night", "focus"),
            ("SEPARATOR", "", ""),
            ("Generali", "preferences-system", "general"),
            ("Aspetto", "preferences-desktop-theme", "appearance"),
            ("Desktop & Dock", "preferences-desktop", "desktop"),
            ("Schermi", "preferences-desktop-display", "displays"),
            ("SEPARATOR", "", ""),
            ("Ecosistema", "network-server", "ecosystem"),
            ("Ermete Labs", "applications-science", "labs"),
            ("Aggiornamenti", "software-update-available", "updates"),
            ("SEPARATOR", "", ""),
            ("Batteria", "battery", "battery"),
            ("Tastiera", "input-keyboard", "keyboard"),
            ("Mouse & Trackpad", "input-mouse", "mouse"),
            ("SEPARATOR", "", ""),
            ("Account", "system-users", "accounts"),
            ("Privacy & Sicurezza", "security-high", "privacy"),
        ];

        let mut first_row = None;

        for (title, icon, id) in categories {
            if title == "SEPARATOR" {
                let sep_row = ListBoxRow::builder().selectable(false).activatable(false).build();
                sep_row.set_widget_name("SEPARATOR");
                let sep = GtkBox::builder().height_request(10).build();
                sep_row.set_child(Some(&sep));
                list_box.append(&sep_row);
                continue;
            }

            let row = ListBoxRow::builder().build();
            row.set_widget_name(id);
            let hbox = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(12)
                .margin_start(12)
                .margin_top(6)
                .margin_bottom(6)
                .build();
            let img = Image::builder().icon_name(icon).pixel_size(24).build();
            let lbl = Label::builder().label(title).css_classes(["sidebar-label"]).build();
            hbox.append(&img);
            hbox.append(&lbl);
            row.set_child(Some(&hbox));

            list_box.append(&row);

            if first_row.is_none() {
                first_row = Some(row.clone());
            }

            let page = if id == "appearance" { crate::pages::appearance::build_page() }
            else if id == "desktop" { crate::pages::desktop::build_page() }
            else if id == "displays" { crate::pages::displays::build_page() }
            else if id == "wifi" { crate::pages::network::build_page() }
            else if id == "audio" { crate::pages::audio::build_page() }
            else if id == "notifications" { crate::pages::notifications::build_page() }
            else if id == "battery" { crate::pages::battery::build_page() }
            else if id == "accounts" { crate::pages::accounts::build_page() }
            else if id == "keyboard" { crate::pages::keyboard::build_page() }
            else if id == "mouse" { crate::pages::mouse::build_page() }
            else if id == "general" { crate::pages::general::build_page() }
            else if id == "privacy" { crate::pages::privacy::build_page() }
            else if id == "bluetooth" { crate::pages::bluetooth::build_page() }
            else if id == "network" { crate::pages::wired::build_page() }
            else if id == "focus" { crate::pages::focus::build_page() }
            else if id == "ecosystem" { crate::pages::ecosystem::build_page() }
            else if id == "labs" { crate::pages::labs::build_page() }
            else if id == "updates" { crate::pages::updates::build_page() }
            else {
                let p = GtkBox::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(20)
                    .margin_top(40)
                    .margin_start(40)
                    .build();
                p.append(&Label::builder().label(title).css_classes(["title-1"]).halign(Align::Start).build());
                p.append(&Label::builder().label("Opzioni in costruzione...").css_classes(["subtitle"]).halign(Align::Start).build());
                p
            };
            stack.add_named(&page, Some(id));
        }

        let query_cell = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
        let qc_clone = query_cell.clone();
        list_box.set_filter_func(move |row| {
            let q = qc_clone.borrow().to_lowercase();
            if q.is_empty() { return true; }
            let widget_name = row.widget_name().to_string();
            if widget_name == "SEPARATOR" { return false; }
            if let Some(child) = row.child() {
                if let Some(hbox) = child.downcast_ref::<GtkBox>() {
                    if let Some(lbl_widget) = hbox.last_child() {
                        if let Some(lbl) = lbl_widget.downcast_ref::<Label>() {
                            let title = lbl.label().to_string();
                            return category_matches_query(&widget_name, &title, &q);
                        }
                    }
                }
            }
            false
        });

        let qc = query_cell.clone();
        let lb = list_box.clone();
        search_entry.connect_search_changed(move |entry| {
            *qc.borrow_mut() = entry.text().to_string();
            lb.invalidate_filter();
        });

        if let Some(row) = first_row {
            list_box.select_row(Some(&row));
        }

        let stack_clone = stack.clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(r) = row {
                let target_id = r.widget_name().to_string();
                if !target_id.is_empty() && target_id != "SEPARATOR" {
                    stack_clone.set_visible_child_name(&target_id);
                }
            }
        });

        let scroll = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .child(&list_box)
            .build();

        sidebar_container.append(&scroll);
        main_box.append(&sidebar_container);
        main_box.append(&stack);

        MAIN_STACK.with(|s| *s.borrow_mut() = Some(stack.clone()));
        MAIN_LIST_BOX.with(|lb| *lb.borrow_mut() = Some(list_box.clone()));

        if let Some(ref page) = model.page_to_open {
            switch_to_page(page);
        }

        relm4::ComponentParts { model, widgets }
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

    let app = relm4::RelmApp::new("os.ermete.Settings");
    app.run::<AppModel>(page_id);
}
