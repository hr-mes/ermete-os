use zbus::{interface, Connection, fdo, names::InterfaceName};
use zbus::zvariant::Value;

pub struct Bluetooth {
    pub sys_conn: Connection,
}

impl Bluetooth {
    pub fn new(sys_conn: Connection) -> Self {
        Self { sys_conn }
    }
}

#[interface(name = "os.ermete.Bedrock.Bluetooth")]
impl Bluetooth {
    #[zbus(property)]
    async fn power(&self) -> bool {
        let proxy_builder = fdo::PropertiesProxy::builder(&self.sys_conn)
            .destination("org.bluez");
        let proxy_builder = match proxy_builder {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error setting destination: {}", e);
                return false;
            }
        };
        let proxy_builder = match proxy_builder.path("/org/bluez/hci0") {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error setting path: {}", e);
                return false;
            }
        };
        if let Ok(proxy) = proxy_builder.build().await
        {
            if let Ok(iface) = InterfaceName::try_from("org.bluez.Adapter1") {
                if let Ok(val) = proxy.get(iface, "Powered").await {
                    if let Ok(b) = bool::try_from(&*val) {
                        return b;
                    }
                }
            }
        }
        false
    }

    #[zbus(property)]
    async fn set_power(&self, val: bool) -> fdo::Result<()> {
        let proxy = fdo::PropertiesProxy::builder(&self.sys_conn)
            .destination("org.bluez")
            .map_err(|e| fdo::Error::Failed(format!("Invalid destination: {}", e)))?
            .path("/org/bluez/hci0")
            .map_err(|e| fdo::Error::Failed(format!("Invalid path: {}", e)))?
            .build().await
            .map_err(|e| fdo::Error::Failed(format!("Failed to connect to BlueZ DBus: {}", e)))?;

        let iface = InterfaceName::try_from("org.bluez.Adapter1")
            .map_err(|e| fdo::Error::Failed(format!("Invalid interface name: {}", e)))?;

        let value = Value::from(val);
        proxy.set(iface, "Powered", value).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to set power: {}", e)))?;
        Ok(())
    }

    async fn get_devices(&self) -> fdo::Result<Vec<(String, String)>> {
        let obj_mgr = fdo::ObjectManagerProxy::builder(&self.sys_conn)
            .destination("org.bluez")
            .map_err(|e| fdo::Error::Failed(format!("Invalid destination: {}", e)))?
            .path("/")
            .map_err(|e| fdo::Error::Failed(format!("Invalid path: {}", e)))?
            .build().await
            .map_err(|e| fdo::Error::Failed(format!("Failed to connect to BlueZ ObjectManager: {}", e)))?;

        let objects = obj_mgr.get_managed_objects().await
            .map_err(|e| fdo::Error::Failed(format!("Failed to get managed objects: {}", e)))?;

        let mut devices = Vec::new();
        for (path, interfaces) in objects {
            if let Some(dev_props) = interfaces.get("org.bluez.Device1") {
                let name = dev_props.get("Alias")
                    .or_else(|| dev_props.get("Name"))
                    .and_then(|v| String::try_from(&**v).ok())
                    .unwrap_or_else(|| {
                        dev_props.get("Address")
                            .and_then(|v| String::try_from(&**v).ok())
                            .unwrap_or_else(|| path.to_string())
                    });

                let connected = dev_props.get("Connected")
                    .and_then(|v| bool::try_from(&**v).ok())
                    .unwrap_or(false);

                let status_icon = if connected { " (Connesso)" } else { "" };
                devices.push((format!("{} {}", status_icon, name), path.to_string()));
            }
        }
        devices.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(devices)
    }
}
