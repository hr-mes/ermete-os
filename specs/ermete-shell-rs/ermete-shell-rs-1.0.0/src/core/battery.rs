use zbus::{proxy, Connection};

#[proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
pub trait UPower {
    fn enumerate_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    
    #[zbus(property)]
    fn on_battery(&self) -> zbus::Result<bool>;
}

#[proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower"
)]
pub trait UPowerDevice {
    #[zbus(property, name = "Type")]
    fn type_(&self) -> zbus::Result<u32>;
    
    #[zbus(property, name = "State")]
    fn state(&self) -> zbus::Result<u32>;
    
    #[zbus(property, name = "Percentage")]
    fn percentage(&self) -> zbus::Result<f64>;
    
    #[zbus(property, name = "IconName")]
    fn icon_name(&self) -> zbus::Result<String>;
}

#[derive(Debug, Clone)]
pub struct BatteryState {
    pub percentage: f64,
    pub state: u32,
    pub icon_name: String,
}

pub async fn get_battery_state() -> zbus::Result<Option<BatteryState>> {
    let system_bus = tokio::time::timeout(std::time::Duration::from_secs(5), Connection::system()).await.map_err(|_| zbus::Error::Failure("timeout".into()))??;
    let upower = tokio::time::timeout(std::time::Duration::from_secs(5), UPowerProxy::new(&system_bus)).await.map_err(|_| zbus::Error::Failure("timeout".into()))??;
    
    let devices = tokio::time::timeout(std::time::Duration::from_secs(5), upower.enumerate_devices()).await.map_err(|_| zbus::Error::Failure("timeout".into()))??;
    for dev_path in devices {
        let device = tokio::time::timeout(std::time::Duration::from_secs(5), UPowerDeviceProxy::builder(&system_bus).path(dev_path)?.build()).await.map_err(|_| zbus::Error::Failure("timeout".into()))??;
            
        // Type 2 is Battery
        if let Ok(dev_type) = device.type_().await {
            if dev_type == 2 {
                let percentage = device.percentage().await.unwrap_or(0.0);
                let state = device.state().await.unwrap_or(0);
                let icon_name = device.icon_name().await.unwrap_or_else(|_| "battery-missing-symbolic".to_string());
                
                return Ok(Some(BatteryState {
                    percentage,
                    state,
                    icon_name,
                }));
            }
        }
    }
    
    Ok(None)
}
