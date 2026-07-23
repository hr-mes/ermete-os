use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;
}

#[allow(dead_code)]
pub async fn get_network_status_dbus() -> (String, String, String) {
    let ctrl = crate::core::system_proxies::get_global_controller();
    let _ = ctrl.refresh_network_status().await;
    ctrl.get_cached_network_status()
}
