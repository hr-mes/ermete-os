#[derive(Debug, Clone)]
pub struct WifiNetworkItem {
    pub ssid: String,
    pub signal: i32,
    pub active: bool,
    pub saved: bool,
}

pub enum WifiIntent {
    SetWifiPowered(bool),
    ConnectNetwork { ssid: String, password: Option<String> },
    ConnectWifiWithPassword { ssid: String, password: String },
    DisconnectNetwork { ssid: String },
    ForgetNetwork { ssid: String },
    ModifyWifi { ssid: String, dhcp: bool, ip: String, gw: String, dns: String, auto: bool },
    LaunchWifiSettings,
}

pub struct WifiViewModel;

impl WifiViewModel {
    pub fn fetch_initial_state<F: Fn(bool) + 'static>(on_powered: F) {
        gtk4::glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            if let Ok(enabled) = ctrl.is_wifi_enabled().await {
                on_powered(enabled);
            }
        });
    }

    pub fn fetch_networks<F: Fn(Result<Vec<WifiNetworkItem>, String>) + 'static>(on_networks: F) {
        gtk4::glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            match ctrl.list_wifi_networks().await {
                Ok(nets) => {
                    let items = nets.into_iter().map(|n| WifiNetworkItem {
                        ssid: n.ssid,
                        signal: n.signal,
                        active: n.active,
                        saved: n.saved,
                    }).collect();
                    on_networks(Ok(items));
                }
                Err(e) => {
                    on_networks(Err(e.to_string()));
                }
            }
        });
    }

    pub fn fetch_details<F: Fn(String, String, String, String, bool) + 'static>(ssid: &str, on_details: F) {
        let ssid_clone = ssid.to_string();
        gtk4::glib::MainContext::default().spawn_local(async move {
            let ctrl = crate::core::get_network_controller();
            if let Ok((method, ip, gw, dns, auto)) = ctrl.get_wifi_details(&ssid_clone).await {
                on_details(method, ip, gw, dns, auto);
            }
        });
    }

    pub fn execute_intent(intent: WifiIntent) {
        match intent {
            WifiIntent::SetWifiPowered(state) => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let _ = ctrl.set_wifi_powered(state).await;
                });
            }
            WifiIntent::ConnectNetwork { ssid, password } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let pwd = password.unwrap_or_default();
                    let _ = ctrl.connect_wifi(&ssid, &pwd).await;
                });
            }
            WifiIntent::ConnectWifiWithPassword { ssid, password } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let _ = ctrl.connect_wifi(&ssid, &password).await;
                });
            }
            WifiIntent::DisconnectNetwork { ssid } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let _ = ctrl.disconnect_wifi(&ssid).await;
                });
            }
            WifiIntent::ForgetNetwork { ssid } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let _ = ctrl.delete_wifi(&ssid).await;
                });
            }
            WifiIntent::ModifyWifi { ssid, dhcp, ip, gw, dns, auto } => {
                gtk4::glib::MainContext::default().spawn_local(async move {
                    let ctrl = crate::core::get_network_controller();
                    let _ = ctrl.modify_wifi(&ssid, dhcp, &ip, &gw, &dns, auto).await;
                });
            }
            WifiIntent::LaunchWifiSettings => {
                let _ = gtk4::glib::spawn_command_line_async("ermete-settings-rs --page wifi");
            }
        }
    }
}
