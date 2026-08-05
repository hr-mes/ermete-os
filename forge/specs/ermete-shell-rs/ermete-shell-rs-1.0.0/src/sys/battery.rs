use zbus::proxy;

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

// Battery system proxies (reactive via dbus/ebpf)
