import re

with open('./forge/specs/ermete-shell-rs/ermete-shell-rs-1.0.0/src/control_center/panel.rs', 'r') as f:
    content = f.read()

proxy_code = """use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};

#[zbus::proxy(
    interface = "os.ermete.MeshSync",
    default_service = "os.ermete.MeshSync",
    default_path = "/os/ermete/MeshSync"
)]
trait MeshSync {
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn ip_addr(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn signal_strength(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn wifi_enabled(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn eth_active(&self) -> zbus::Result<bool>;
}

#[zbus::proxy(
    interface = "os.ermete.AiDaemon",
    default_service = "os.ermete.AiDaemon",
    default_path = "/os/ermete/AiDaemon"
)]
trait AiDaemon {
    #[zbus(property)]
    fn current_mode(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn ring0_tracepoints_active(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn latency_reduction_pct(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn context_switches_saved(&self) -> zbus::Result<u64>;
    #[zbus(property)]
    fn status_text(&self) -> zbus::Result<String>;
}
"""

content = content.replace("use crate::ui::viewmodel::{ControlCenterViewModel, ControlCenterIntent};\n", proxy_code)

init_code = """        root.add_controller(key_ctrl);

        let sender_mesh = sender.clone();
        glib::spawn_future_local(async move {
            let conn = match zbus::Connection::system().await {
                Ok(c) => c,
                Err(_) => return,
            };
            
            if let Ok(mesh_proxy) = MeshSyncProxy::new(&conn).await {
                let update_mesh = |proxy: &MeshSyncProxy<'_>| async {
                    if let (Ok(connected), Ok(ssid), Ok(ip_addr), Ok(signal), Ok(wifi), Ok(eth)) = 
                        (proxy.connected().await, proxy.ssid().await, proxy.ip_addr().await, proxy.signal_strength().await, proxy.wifi_enabled().await, proxy.eth_active().await) {
                        let net_data = NetworkModuleData {
                            connected,
                            ssid,
                            ip_addr,
                            signal_strength: signal,
                            wifi_enabled: wifi,
                            eth_active: eth,
                        };
                        sender_mesh.input(CcPanelInput::UpdateNetwork(net_data));
                    }
                };
                
                update_mesh(&mesh_proxy).await;
                
                if let Ok(mut changes) = mesh_proxy.receive_all_properties_changed().await {
                    use zbus::export::futures_util::StreamExt;
                    while let Some(_) = changes.next().await {
                        update_mesh(&mesh_proxy).await;
                    }
                }
            }
        });

        let sender_ai = sender.clone();
        glib::spawn_future_local(async move {
            let conn = match zbus::Connection::system().await {
                Ok(c) => c,
                Err(_) => return,
            };
            
            if let Ok(ai_proxy) = AiDaemonProxy::new(&conn).await {
                let update_ai = |proxy: &AiDaemonProxy<'_>| async {
                    if let (Ok(mode_str), Ok(tracepoints), Ok(latency), Ok(cs_saved), Ok(status)) = 
                        (proxy.current_mode().await, proxy.ring0_tracepoints_active().await, proxy.latency_reduction_pct().await, proxy.context_switches_saved().await, proxy.status_text().await) {
                        
                        let mode = match mode_str.as_str() {
                            "GamingLowLatency" => crate::control_center::ebpf::EbpfMode::GamingLowLatency,
                            "EcoSaver" => crate::control_center::ebpf::EbpfMode::EcoSaver,
                            "MaxThroughput" => crate::control_center::ebpf::EbpfMode::MaxThroughput,
                            _ => crate::control_center::ebpf::EbpfMode::AiInferred,
                        };
                        
                        let ebpf_data = EbpfModuleData {
                            current_mode: mode,
                            ring0_tracepoints_active: tracepoints as usize,
                            latency_reduction_pct: latency,
                            context_switches_saved: cs_saved,
                            status_text: status,
                        };
                        sender_ai.input(CcPanelInput::UpdateEbpf(ebpf_data));
                    }
                };
                
                update_ai(&ai_proxy).await;
                
                if let Ok(mut changes) = ai_proxy.receive_all_properties_changed().await {
                    use zbus::export::futures_util::StreamExt;
                    while let Some(_) = changes.next().await {
                        update_ai(&ai_proxy).await;
                    }
                }
            }
        });
"""

content = content.replace("        root.add_controller(key_ctrl);\n", init_code)

with open('./forge/specs/ermete-shell-rs/ermete-shell-rs-1.0.0/src/control_center/panel.rs', 'w') as f:
    f.write(content)
