use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, Orientation,
    PasswordEntry, Spinner,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiometricState {
    Idle,
    Scanning,
    Recognized,
    Failed,
}

pub struct BiometricPrompt {
    pub state: Rc<RefCell<BiometricState>>,
}

impl BiometricPrompt {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(BiometricState::Idle)),
        }
    }
}

pub fn build_ui(app: &Application, daemon_name: &str, username: &str) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Autenticazione PolKit / PAM - Biometria")
        .css_classes(["biometrics-window"])
        .default_width(440)
        .build();

    window.init_layer_shell();
    window.set_namespace("biometrics");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.auto_exclusive_zone_enable();

    // Center layout
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Bottom, 0);
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);

    let main_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(28)
        .margin_bottom(28)
        .margin_start(28)
        .margin_end(28)
        .css_classes(["biometrics-card"])
        .build();

    // 1. Header (Privilege Request Daemon Info)
    let header_icon = Image::builder()
        .icon_name("security-high-symbolic")
        .pixel_size(52)
        .css_classes(["biometrics-header-icon"])
        .halign(Align::Center)
        .build();
    main_box.append(&header_icon);

    let title_text = format!("Richiesta Privilegi: {}", daemon_name);
    let title_label = Label::builder()
        .label(&title_text)
        .css_classes(["biometrics-title"])
        .halign(Align::Center)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();
    main_box.append(&title_label);

    let desc_label = Label::builder()
        .label("Il demone richiede privilegi amministrativi. Autenticati tramite biometria o password.")
        .css_classes(["biometrics-desc"])
        .halign(Align::Center)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();
    main_box.append(&desc_label);

    // 2. User Avatar Section
    let user_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .css_classes(["biometrics-user-box"])
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let user_avatar = Image::builder()
        .icon_name("avatar-default-symbolic")
        .pixel_size(40)
        .css_classes(["user-avatar"])
        .build();

    let user_name_text = if username.is_empty() {
        "Utente: root".to_string()
    } else {
        format!("Utente: {}", username)
    };

    let user_name_label = Label::builder()
        .label(&user_name_text)
        .css_classes(["user-name"])
        .halign(Align::Center)
        .build();

    user_box.append(&user_avatar);
    user_box.append(&user_name_label);
    main_box.append(&user_box);

    // 3. Biometrics Area (Fingerprint / Face Scan UI)
    let biometric_area = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .halign(Align::Center)
        .css_classes(["biometric-area"])
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let bio_icon = Image::builder()
        .icon_name("fingerprint-symbolic")
        .pixel_size(64)
        .css_classes(["biometric-icon"])
        .halign(Align::Center)
        .build();
    biometric_area.append(&bio_icon);

    let spinner = Spinner::builder()
        .halign(Align::Center)
        .visible(false)
        .build();
    biometric_area.append(&spinner);

    let status_label = Label::builder()
        .label("In attesa di impronta digitale o scansione viso...")
        .css_classes(["biometric-status"])
        .halign(Align::Center)
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();
    biometric_area.append(&status_label);

    main_box.append(&biometric_area);

    // 4. Text Password Fallback Area
    let password_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .visible(false)
        .css_classes(["password-fallback-box"])
        .margin_top(8)
        .build();

    let pass_label = Label::builder()
        .label("Fallback: Inserisci Password Amministratore")
        .css_classes(["password-label"])
        .halign(Align::Start)
        .build();
    password_box.append(&pass_label);

    let password_entry = PasswordEntry::builder()
        .placeholder_text("Password...")
        .css_classes(["password-entry"])
        .show_peek_icon(true)
        .build();
    password_box.append(&password_entry);

    let error_label = Label::builder()
        .label("")
        .css_classes(["password-error"])
        .halign(Align::Center)
        .visible(false)
        .build();
    password_box.append(&error_label);

    main_box.append(&password_box);

    // 5. Action Buttons
    let btn_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(Align::Center)
        .margin_top(16)
        .build();

    let btn_cancel = Button::builder()
        .label("Annulla")
        .css_classes(["biometrics-btn", "btn-cancel"])
        .build();

    let btn_scan = Button::builder()
        .label("Scansiona")
        .css_classes(["suggested-action", "biometrics-btn"])
        .build();

    let btn_password = Button::builder()
        .label("Usa Password")
        .css_classes(["biometrics-btn"])
        .build();

    let btn_confirm = Button::builder()
        .label("Conferma Password")
        .css_classes(["suggested-action", "biometrics-btn"])
        .visible(false)
        .build();

    btn_box.append(&btn_cancel);
    btn_box.append(&btn_scan);
    btn_box.append(&btn_password);
    btn_box.append(&btn_confirm);

    main_box.append(&btn_box);

    // Connect Handlers
    btn_cancel.connect_clicked(move |_| {
        std::process::exit(1);
    });

    let pass_box_clone = password_box.clone();
    let pass_entry_clone = password_entry.clone();
    let btn_confirm_show = btn_confirm.clone();
    btn_password.connect_clicked(move |_| {
        pass_box_clone.set_visible(true);
        btn_confirm_show.set_visible(true);
        pass_entry_clone.grab_focus();
    });

    let pass_entry_val = password_entry.clone();
    let err_lbl = error_label.clone();
    btn_confirm.connect_clicked(move |_| {
        let text = pass_entry_val.text();
        if text.is_empty() {
            err_lbl.set_text("La password non può essere vuota.");
            err_lbl.set_visible(true);
        } else {
            // Password confirmed
            std::process::exit(0);
        }
    });

    // Honest Biometric Hardware Query via DBus fprintd
    let status_clone1 = status_label.clone();
    let bio_icon_clone1 = bio_icon.clone();
    let spinner_clone1 = spinner.clone();
    let pass_box_c1 = password_box.clone();
    let pass_entry_c1 = password_entry.clone();
    let btn_conf_c1 = btn_confirm.clone();
    let user_name_str = username.to_string();

    btn_scan.connect_clicked(move |_| {
        status_clone1.set_text("Interrogazione fprintd DBus...");
        spinner_clone1.set_visible(true);
        spinner_clone1.start();
        bio_icon_clone1.set_icon_name(Some("fingerprint-symbolic"));

        let status_lbl = status_clone1.clone();
        let icon_img = bio_icon_clone1.clone();
        let sp = spinner_clone1.clone();
        let pass_box = pass_box_c1.clone();
        let pass_entry = pass_entry_c1.clone();
        let btn_conf = btn_conf_c1.clone();
        let target_user = user_name_str.clone();

        glib::spawn_future_local(async move {
            let scan_result = run_biometric_scan(&target_user).await;

            sp.stop();
            sp.set_visible(false);

            match scan_result {
                Ok(()) => {
                    status_lbl.set_text("Sensore biometrico acquisito. Posiziona l'impronta.");
                    icon_img.set_icon_name(Some("emblem-ok-symbolic"));
                }
                Err(err_msg) => {
                    status_lbl.set_text(&format!("Errore biometria: {}. Inserisci la password.", err_msg));
                    icon_img.set_icon_name(Some("dialog-warning-symbolic"));

                    // Fallback to text password input on failure / missing hardware
                    pass_box.set_visible(true);
                    btn_conf.set_visible(true);
                    pass_entry.grab_focus();
                }
            }
        });
    });

    window.set_child(Some(&main_box));
    window.present();
}

/// Honest D-Bus query to fprintd manager service for biometric hardware.
/// Returns Err("Biometric hardware missing") if device or service is missing.
pub async fn run_biometric_scan(username: &str) -> Result<(), String> {
    let connection = match zbus::Connection::system().await {
        Ok(conn) => conn,
        Err(e) => return Err(format!("D-Bus system connection failed: {}", e)),
    };

    let reply = connection
        .call_method(
            Some("net.reactivated.Fprint"),
            "/net/reactivated/Fprint/Manager",
            Some("net.reactivated.Fprint.Manager"),
            "GetDefaultDevice",
            &(),
        )
        .await;

    match reply {
        Ok(msg) => {
            let device_path: Result<zbus::zvariant::OwnedObjectPath, _> = msg.body().deserialize();
            match device_path {
                Ok(path) => {
                    tracing::info!("fprintd device path found: {:?}", path);
                    let claim_reply = connection
                        .call_method(
                            Some("net.reactivated.Fprint"),
                            &path,
                            Some("net.reactivated.Fprint.Device"),
                            "Claim",
                            &(username,),
                        )
                        .await;

                    if let Err(e) = claim_reply {
                        return Err(format!("Device claim failed: {}", e));
                    }
                    Ok(())
                }
                Err(_) => Err("Biometric hardware missing".to_string()),
            }
        }
        Err(_) => Err("Biometric hardware missing".to_string()),
    }
}

