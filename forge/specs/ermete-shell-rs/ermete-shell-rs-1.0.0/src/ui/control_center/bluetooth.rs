use crate::ui::popup_manager::setup_popup_autoclose;
use crate::ui::viewmodel::{BluetoothViewModel, BluetoothIntent};
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, Switch};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_bluetooth_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Bluetooth")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "bluetooth");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .css_classes(["liquid-surface"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let header_lbl = Label::builder().label("Bluetooth").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let bt_sw = Switch::builder().active(true).valign(Align::Center).build();
    let bt_sw_clone = bt_sw.clone();
    
    BluetoothViewModel::fetch_initial_state(move |enabled| {
        bt_sw_clone.set_active(enabled);
    });

    bt_sw.connect_state_set(move |_, state| {
        BluetoothViewModel::execute_intent(BluetoothIntent::TogglePowered(state));
        glib::Propagation::Proceed
    });
    header_card.append(&header_icon);
    header_card.append(&header_lbl);
    header_card.append(&bt_sw);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let list_box_clone = list_box.clone();
    BluetoothViewModel::fetch_devices(move |devices| {
        for dev in devices.iter().take(8) {
            let item_row = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(10)
                .css_classes(["pro-applet-card"])
                .build();

            let icon_lbl = Label::builder().label("").build();
            let texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
            let name_lbl = Label::builder()
                .label(&dev.name)
                .css_classes(["cc-label-main"])
                .halign(Align::Start)
                .build();
            let sub_lbl = Label::builder()
                .label(if dev.connected { "Connesso" } else { "Dispositivo Rilevato" })
                .css_classes(["cc-label-sub"])
                .halign(Align::Start)
                .build();
            texts.append(&name_lbl);
            texts.append(&sub_lbl);

            item_row.append(&icon_lbl);
            item_row.append(&texts);
            list_box_clone.append(&item_row);
        }
        if devices.is_empty() {
            let no_bt = Label::builder()
                .label("Nessun dispositivo accoppiato")
                .css_classes(["cc-label-sub"])
                .build();
            list_box_clone.append(&no_bt);
        }
    });

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    let footer_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .build();
    let settings_bt_btn = Button::builder()
        .label("⚙ Impostazioni Bluetooth")
        .css_classes(["cc-quick-btn"])
        .hexpand(true)
        .build();
    let pop_bt_s = pop.clone();
    settings_bt_btn.connect_clicked(move |_| {
        pop_bt_s.close();
        BluetoothViewModel::execute_intent(BluetoothIntent::LaunchBluetoothSettings);
    });
    footer_box.append(&settings_bt_btn);
    footer_box.append(&close_btn);

    card.append(&header_card);
    card.append(&list_box);
    card.append(&footer_box);

    pop.set_child(Some(&card));
    pop.present();
}
