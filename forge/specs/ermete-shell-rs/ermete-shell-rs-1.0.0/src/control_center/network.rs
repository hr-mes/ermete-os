use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Switch};

#[derive(Debug, Clone)]
pub struct NetworkModuleData {
    pub connected: bool,
    pub ssid: String,
    pub ip_addr: String,
    pub signal_strength: u8,
    pub wifi_enabled: bool,
    pub eth_active: bool,
}

impl Default for NetworkModuleData {
    fn default() -> Self {
        let (_net_icon, _title, net_sub) = crate::core::get_network_controller().get_cached_network_status();
        let connected = net_sub != "Disattivato" && net_sub != "Non connesso" && net_sub != "Off" && net_sub != "Disconnected";
        Self {
            connected,
            ssid: if connected { net_sub } else { "Disconnect".to_string() },
            ip_addr: "192.168.1.142".to_string(),
            signal_strength: 85,
            wifi_enabled: connected,
            eth_active: false,
        }
    }
}

pub fn build_network_widget(data: &NetworkModuleData) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(4)
        .margin_bottom(4)
        .build();

    // Row 1: Wi-Fi status and toggle switch
    let row1 = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .valign(Align::Center)
        .build();

    let wifi_icon = Label::builder()
        .label(if data.wifi_enabled { "󰤨" } else { "󰤯" })
        .css_classes(["cc-circle-blue"])
        .valign(Align::Center)
        .build();

    let info_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let ssid_text = if data.connected { format!("SSID: {}", data.ssid) } else { "Wi-Fi Disconnected".to_string() };
    let ssid_label = Label::builder()
        .label(&ssid_text)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();

    let status_text = if data.connected {
        format!("IP: {} • Signal: {}%", data.ip_addr, data.signal_strength)
    } else {
        "Tap switch to enable Wi-Fi".to_string()
    };
    let status_label = Label::builder()
        .label(&status_text)
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    info_box.append(&ssid_label);
    info_box.append(&status_label);

    let toggle_sw = Switch::builder()
        .active(data.wifi_enabled)
        .valign(Align::Center)
        .build();

    toggle_sw.connect_state_set(move |_, state| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::ToggleWifi(state));
        glib::Propagation::Proceed
    });

    row1.append(&wifi_icon);
    row1.append(&info_box);
    row1.append(&toggle_sw);

    // Row 2: Action buttons (Wi-Fi Settings & Ethernet Proxy)
    let row2 = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .homogeneous(true)
        .build();

    let wifi_btn = Button::builder()
        .label("⚙ Network Settings")
        .css_classes(["cc-quick-btn"])
        .build();
    wifi_btn.connect_clicked(move |_| {
        ControlCenterViewModel::execute_intent(ControlCenterIntent::LaunchSettings("network".to_string()));
    });

    let eth_btn = Button::builder()
        .label(if data.eth_active { "🔌 Eth: Active" } else { "🔌 Eth: Standby" })
        .css_classes(["cc-quick-btn"])
        .build();

    row2.append(&wifi_btn);
    row2.append(&eth_btn);

    container.append(&row1);
    container.append(&row2);
    container
}
