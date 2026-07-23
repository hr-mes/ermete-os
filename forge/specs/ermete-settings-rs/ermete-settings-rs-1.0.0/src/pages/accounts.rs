use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Image, Label, Orientation, Switch};

#[zbus::dbus_proxy(
    interface = "org.freedesktop.Accounts.User",
    default_service = "org.freedesktop.Accounts"
)]
trait AccountsUser {
    fn set_password(&self, password: &str, hint: &str) -> zbus::Result<()>;
}

#[zbus::dbus_proxy(
    interface = "org.ermete.Bedrock",
    default_service = "org.ermete.Bedrock",
    default_path = "/org/ermete/Bedrock"
)]
trait Bedrock {
    fn enroll_keyring_secret(&self, secret: &str) -> zbus::Result<()>;
}

pub fn build_page() -> gtk4::Box {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(32)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    // Title
    let title = Label::builder()
        .label("Account Utente")
        .halign(Align::Start)
        .css_classes(["title-1", "large-title"])
        .build();

    // User Profile Section
    let profile_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .halign(Align::Center)
        .build();

    let avatar = Image::builder()
        .icon_name("avatar-default-symbolic")
        .pixel_size(128)
        .css_classes(["circular"])
        .build();

    let username = get_username();

    let name_label = Label::builder()
        .label(&username)
        .halign(Align::Center)
        .css_classes(["title-2"])
        .build();

    let role_label = Label::builder()
        .label("Amministratore")
        .halign(Align::Center)
        .css_classes(["dim-label"])
        .build();

    let name_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .halign(Align::Center)
        .build();
    
    name_box.append(&name_label);
    name_box.append(&role_label);

    profile_box.append(&avatar);
    profile_box.append(&name_box);

    // Settings Section (Card-like)
    let settings_list = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["card"])
        .build();

    // Change Password Row
    let password_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let password_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();
    let password_title = Label::builder()
        .label("Password")
        .halign(Align::Start)
        .build();
    let password_desc = Label::builder()
        .label("Modifica la password di accesso")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();
    password_label_box.append(&password_title);
    password_label_box.append(&password_desc);

    let password_btn = Button::builder()
        .label("Cambia Password...")
        .valign(Align::Center)
        .build();
    
    password_btn.connect_clicked(move |btn| {
        if let Some(window) = btn.root().and_downcast::<gtk4::Window>() {
            let dialog = gtk4::Dialog::builder()
                .title("Cambia Password")
                .transient_for(&window)
                .modal(true)
                .build();

            let content_area = dialog.content_area();
            let entry = gtk4::PasswordEntry::builder()
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .show_peek_icon(true)
                .build();

            content_area.append(&entry);
            dialog.add_button("Annulla", gtk4::ResponseType::Cancel);
            dialog.add_button("Cambia", gtk4::ResponseType::Ok);

            entry.grab_focus();
            let entry_clone = entry.clone();
            
            dialog.connect_response(move |dlg, response| {
                if response == gtk4::ResponseType::Ok {
                    let new_password = entry_clone.text().to_string();
                    let dlg_clone = dlg.clone();
                    let entry_for_error = entry_clone.clone();
                    let ctx = gtk4::glib::MainContext::default();
                    ctx.spawn_local(async move {
                        let mut success = false;
                        match zbus::Connection::system().await {
                            Ok(conn) => {
                                let uid = unsafe { libc::getuid() };
                                let path = format!("/org/freedesktop/Accounts/User{}", uid);
                                let Ok(builder) = AccountsUserProxy::builder(&conn).path(path.as_str()) else {
                                    eprintln!("Invalid DBus object path for user: {}", path);
                                    entry_for_error.add_css_class("error");
                                    return;
                                };
                                if let Ok(proxy) = builder.build().await {
                                    if let Err(e) = proxy.set_password(&new_password, "hint").await {
                                        eprintln!("Error setting password on AccountService: {:?}", e);
                                    } else {
                                        success = true;
                                    }
                                } else {
                                    eprintln!("Error building proxy for AccountService");
                                }
                                if success {
                                    if let Ok(bedrock) = BedrockProxy::new(&conn).await {
                                        if let Err(e) = bedrock.enroll_keyring_secret(&new_password).await {
                                            eprintln!("Error enrolling secret: {:?}", e);
                                            success = false;
                                        } else {
                                            println!("Successfully changed password and enrolled secret.");
                                        }
                                    } else {
                                        eprintln!("Error building proxy for Bedrock");
                                        success = false;
                                    }
                                }
                            }
                            Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
                        }
                        
                        if success {
                            dlg_clone.close();
                        } else {
                            entry_for_error.add_css_class("error");
                        }
                    });
                } else {
                    dlg.close();
                }
            });

            dialog.present();
        }
    });

    password_row.append(&password_label_box);
    password_row.append(&password_btn);

    // Auto Login Row
    let autologin_row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let autologin_label_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .build();
    let autologin_title = Label::builder()
        .label("Login Automatico")
        .halign(Align::Start)
        .build();
    let autologin_desc = Label::builder()
        .label("Accedi senza inserire la password all'avvio")
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();
    autologin_label_box.append(&autologin_title);
    autologin_label_box.append(&autologin_desc);

    let autologin_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    autologin_row.append(&autologin_label_box);
    autologin_row.append(&autologin_switch);

    settings_list.append(&password_row);
    // Add a separator here if you prefer using GtkSeparator
    let separator = gtk4::Separator::builder()
        .orientation(Orientation::Horizontal)
        .build();
    settings_list.append(&separator);
    settings_list.append(&autologin_row);

    container.append(&title);
    container.append(&profile_box);
    container.append(&settings_list);

    container
}

fn get_username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "ermete".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accounts_proxies_exist() {
        // Assert proxy interfaces exist without live D-Bus connection
        // (This validates that zbus macros successfully compiled the proxy traits)
        let _ = AccountsUserProxy::builder;
        let _ = BedrockProxy::builder;
        
        // Some zbus versions don't expose INTERFACE as string constant, so just verifying the builder
        // exists and typechecks is sufficient struct verification.
    }
}
