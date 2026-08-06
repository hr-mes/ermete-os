#![allow(deprecated)]
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, ListBox, Orientation, Switch};
use crate::components::action_row::ActionRow;

#[allow(deprecated)]
#[zbus::dbus_proxy(
    interface = "os.ermete.Bedrock.Bluetooth",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock/Bluetooth"
)]
trait Bluetooth {
    #[dbus_proxy(property)]
    fn power(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_power(&self, value: bool) -> zbus::Result<()>;

    fn get_devices(&self) -> zbus::Result<Vec<(String, String)>>;
}

#[allow(deprecated)]
#[zbus::dbus_proxy(
    interface = "org.bluez.Device1",
    default_service = "org.bluez"
)]
trait Device1 {
    fn connect(&self) -> zbus::Result<()>;
    fn pair(&self) -> zbus::Result<()>;
}

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("Bluetooth")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();

    container.append(&title);

    let settings_card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .css_classes(["card"])
        .build();

    // Global Switch
    let power_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    // Set initial state
    let power_switch_clone = power_switch.clone();
    relm4::spawn_local(async move {
        match crate::get_system_connection().await {
            Ok(conn) => {
                match BluetoothProxy::new(&conn).await {
                    Ok(proxy) => {
                        match proxy.power().await {
                            Ok(power) => power_switch_clone.set_active(power),
                            Err(e) => eprintln!("Error getting Bluetooth power state: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Error creating DBus proxy for Bluetooth: {:?}", e),
                }
            }
            Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
        }
    });

    power_switch.connect_state_set(|_switch, state| {
        relm4::spawn_local(async move {
            match crate::get_system_connection().await {
                Ok(conn) => {
                    match BluetoothProxy::new(&conn).await {
                        Ok(proxy) => {
                            if let Err(e) = proxy.set_power(state).await {
                                eprintln!("Error setting Bluetooth power state: {:?}", e);
                            }
                        }
                        Err(e) => eprintln!("Error creating DBus proxy for Bluetooth: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
            }
        });
        gtk4::glib::Propagation::Proceed
    });

    let power_row = ActionRow::builder("Attiva Bluetooth")
        .subtitle("Abilita o disabilita il modulo Bluetooth del sistema")
        .suffix(&power_switch)
        .build();

    settings_card.append(&power_row);
    container.append(&settings_card);

    // Search button
    let search_button = Button::builder()
        .label("Cerca Dispositivi")
        .halign(Align::Start)
        .build();

    // Devices list box
    let list_box = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    let list_box_clone = list_box.clone();

    search_button.connect_clicked(move |_| {
        let list_box = list_box_clone.clone();
        
        // Show loading state
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        let loading_label = Label::new(Some("Caricamento..."));
        loading_label.set_margin_top(12);
        loading_label.set_margin_bottom(12);
        list_box.append(&loading_label);
        
        relm4::spawn_local(async move {
            match crate::get_system_connection().await {
                Ok(conn) => {
                    match BluetoothProxy::new(&conn).await {
                        Ok(proxy) => {
                            match proxy.get_devices().await {
                                Ok(devices) => {
                                    while let Some(child) = list_box.first_child() {
                                        list_box.remove(&child);
                                    }
                                    for (device_name, device_path) in devices {
                                        let connect_btn = Button::builder()
                                            .label("Connetti")
                                            .valign(Align::Center)
                                            .build();
                                            
                                        let connect_btn_clone = connect_btn.clone();
                                        let device_path_for_closure = device_path.clone();
                                        connect_btn.connect_clicked(move |_| {
                                            connect_btn_clone.set_label("Connessione...");
                                            connect_btn_clone.set_sensitive(false);
                                            let path = device_path_for_closure.clone();
                                            let connect_btn_async = connect_btn_clone.clone();
                                            relm4::spawn_local(async move {
                                                let mut success = true;
                                                match crate::get_system_connection().await {
                                                    Ok(conn) => {
                                                        let Ok(builder) = Device1Proxy::builder(&conn).path(path.as_str()) else {
                                                            eprintln!("Invalid DBus object path for device: {}", path);
                                                            connect_btn_async.set_label("Errore");
                                                            connect_btn_async.set_sensitive(true);
                                                            return;
                                                        };
                                                        if let Ok(proxy) = builder.build().await {
                                                            if let Err(e) = proxy.pair().await {
                                                                eprintln!("Error pairing with {}: {:?}", path, e);
                                                            } else {
                                                                println!("Successfully paired with {}", path);
                                                            }
                                                            if let Err(e) = proxy.connect().await {
                                                                eprintln!("Error connecting to {}: {:?}", path, e);
                                                                success = false;
                                                            } else {
                                                                println!("Successfully connected to {}", path);
                                                            }
                                                        } else {
                                                            eprintln!("Error building proxy for Device1");
                                                            success = false;
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Error connecting to DBus: {:?}", e);
                                                        success = false;
                                                    }
                                                }
                                                
                                                if success {
                                                    connect_btn_async.set_label("Connesso");
                                                } else {
                                                    connect_btn_async.set_label("Errore");
                                                    connect_btn_async.set_sensitive(true);
                                                }
                                            });
                                        });
                                            
                                        let row = ActionRow::builder(&device_name)
                                            .subtitle(&device_path)
                                            .suffix(&connect_btn)
                                            .build();
                                        
                                        list_box.append(&row);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error getting Bluetooth devices: {:?}", e);
                                    while let Some(child) = list_box.first_child() {
                                        list_box.remove(&child);
                                    }
                                    let error_label = Label::new(Some("Errore durante la ricerca"));
                                    error_label.set_margin_top(12);
                                    error_label.set_margin_bottom(12);
                                    list_box.append(&error_label);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error creating DBus proxy for Bluetooth: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
            }
        });
    });

    container.append(&search_button);
    container.append(&list_box);

    container
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluetooth_proxies_exist() {
        let _ = BluetoothProxy::builder;
        let _ = Device1Proxy::builder;
    }
}
