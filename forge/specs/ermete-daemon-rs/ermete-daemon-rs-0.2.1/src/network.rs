use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use zbus::{interface, proxy, Connection, fdo};
use zbus::zvariant::OwnedObjectPath;
use std::collections::{HashMap, HashSet};

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
    fn check_connectivity(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NmDevice {
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NmWireless {
    fn get_access_points(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
    fn request_scan(&self, options: HashMap<&str, zbus::zvariant::Value<'_>>) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NmAccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
    #[zbus(property)]
    fn wpa_flags(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn rsn_flags(&self) -> zbus::Result<u32>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait NmSettings {
    fn add_connection(&self, connection: HashMap<&'static str, HashMap<&'static str, zbus::zvariant::Value<'_>>>) -> zbus::Result<OwnedObjectPath>;
}

pub struct Network {
    pub sys_conn: Connection,
}

impl Network {
    pub fn new(sys_conn: Connection) -> Self {
        Self { sys_conn }
    }
}

use futures_util::future::join_all;

#[interface(name = "os.ermete.Bedrock.Network")]
impl Network {
    #[tracing::instrument(skip(self))]
    async fn scan_networks(&self) -> fdo::Result<Vec<String>> {
        let nm_proxy = NetworkManagerProxy::new(&self.sys_conn).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to connect to NetworkManager DBus: {}", e)))?;

        let devices = nm_proxy.get_devices().await
            .map_err(|e| fdo::Error::Failed(format!("Failed to get devices: {}", e)))?;

        // Concurrently query device types and fetch access points for WiFi devices
        let dev_futures = devices.into_iter().map(|dev_path| async move {
            let builder = NmDeviceProxy::builder(&self.sys_conn).path(dev_path.clone()).ok()?;
            let dev_proxy = builder.build().await.ok()?;
            let dev_type = dev_proxy.device_type().await.ok()?;
            if dev_type == 2 {
                let wifi_builder = NmWirelessProxy::builder(&self.sys_conn).path(dev_path).ok()?;
                let wifi_proxy = wifi_builder.build().await.ok()?;
                let _ = wifi_proxy.request_scan(HashMap::new()).await;
                wifi_proxy.get_access_points().await.ok()
            } else {
                None
            }
        });

        let dev_results = join_all(dev_futures).await;
        let ap_paths: Vec<OwnedObjectPath> = dev_results.into_iter().flatten().flatten().collect();

        // Concurrently query all access point properties
        let ap_futures = ap_paths.into_iter().map(|ap_path| async move {
            let builder = NmAccessPointProxy::builder(&self.sys_conn).path(ap_path).ok()?;
            let ap_proxy = builder.build().await.ok()?;

            let (ssid_res, strength_res, wpa_res, rsn_res) = tokio::join!(
                ap_proxy.ssid(),
                ap_proxy.strength(),
                ap_proxy.wpa_flags(),
                ap_proxy.rsn_flags()
            );

            let ssid_bytes = ssid_res.ok()?;
            let ssid = String::from_utf8_lossy(&ssid_bytes).trim().to_string();
            if ssid.is_empty() {
                return None;
            }

            let strength = strength_res.unwrap_or(0);
            let wpa = wpa_res.unwrap_or(0);
            let rsn = rsn_res.unwrap_or(0);
            let is_protected = wpa > 0 || rsn > 0;

            Some((ssid, strength, is_protected))
        });

        let ap_results = join_all(ap_futures).await;
        let mut results: Vec<(String, u8, bool)> = ap_results.into_iter().flatten().collect();

        results.sort_by_key(|b| std::cmp::Reverse(b.1));
        let mut seen = HashSet::new();
        let mut formatted = Vec::new();
        for (ssid, strength, is_protected) in results {
            if seen.insert(ssid.clone()) {
                let sec_icon = if is_protected { "🔒" } else { "🔓" };
                formatted.push(format!("{} [{}% {}]", ssid, strength, sec_icon));
            }
        }

        Ok(formatted)
    }

    /// Check system connectivity status (`PORTAL`, `FULL`, `LIMITED`, `NONE`, `UNKNOWN`)
    #[tracing::instrument(skip(self))]
    async fn check_connectivity(&self) -> fdo::Result<String> {
        let nm_proxy = NetworkManagerProxy::new(&self.sys_conn).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to connect to NetworkManager DBus: {}", e)))?;

        let status = match nm_proxy.check_connectivity().await {
            Ok(s) => s,
            Err(_) => nm_proxy.connectivity().await.unwrap_or(1),
        };

        // NM_CONNECTIVITY values: 1=UNKNOWN, 2=NONE, 3=PORTAL, 4=LIMITED, 5=FULL
        let status_str = match status {
            2 => "NONE",
            3 => "PORTAL",
            4 => "LIMITED",
            5 => "FULL",
            _ => "UNKNOWN",
        };

        tracing::info!(status, status_str, "CheckConnectivity status returned");
        Ok(status_str.to_string())
    }

    /// Configure and activate 802.1x EAP enterprise Wi-Fi connection
    #[tracing::instrument(skip(self, hdr, identity))]
    async fn connect_enterprise_wifi(
        &self,
        ssid: String,
        identity: String,
        eap_method: String,
        ca_cert_path: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
    ) -> fdo::Result<String> {
        let sender = hdr.sender().map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.network.configure").await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }

        tracing::info!(%ssid, %eap_method, identity = "[REDACTED]", "Enterprise Wi-Fi connection requested");
        let nm_settings = NmSettingsProxy::new(&self.sys_conn).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to connect to NetworkManager Settings DBus: {}", e)))?;
        let dict = Self::build_enterprise_wifi_dict(&ssid, &identity, &eap_method, &ca_cert_path);
        let path = nm_settings.add_connection(dict).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to add connection: {}", e)))?;
        Ok(path.to_string())
    }

    /// Add WireGuard or OpenVPN tunnel from config file
    #[tracing::instrument(skip(self, hdr))]
    async fn add_vpn_tunnel(
        &self,
        name: String,
        vpn_type: String,
        config_path: String,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
    ) -> fdo::Result<String> {
        let sender = hdr.sender().map(|s| s.as_str());
        if !crate::bedrock::check_polkit_auth(sender, "os.ermete.network.configure").await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }

        if config_path.contains("..") || (!config_path.starts_with("/etc/") && !config_path.starts_with("/var/home/")) {
            return Err(fdo::Error::InvalidArgs("Invalid config path for VPN".to_string()));
        }
        
        // Sanitize vpn_type to avoid shell injections or invalid types
        if !vpn_type.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err(fdo::Error::InvalidArgs("Invalid VPN type".to_string()));
        }

        tracing::info!(%name, %vpn_type, %config_path, "Staging VPN Tunnel");

        let content = tokio::fs::read_to_string(&config_path).await.unwrap_or_default();
        let nm_settings = NmSettingsProxy::new(&self.sys_conn).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to connect to NetworkManager Settings DBus: {}", e)))?;
        let dict = Self::build_vpn_tunnel_dict(&name, &vpn_type, &config_path, &content);
        let path = nm_settings.add_connection(dict).await
            .map_err(|e| fdo::Error::Failed(format!("Failed to add connection: {}", e)))?;
        Ok(path.to_string())
    }
}

impl Network {
    pub fn build_enterprise_wifi_dict(
        ssid: &str,
        identity: &str,
        eap_method: &str,
        ca_cert_path: &str,
    ) -> HashMap<&'static str, HashMap<&'static str, zbus::zvariant::Value<'static>>> {
        let mut dict = HashMap::new();

        let mut conn = HashMap::new();
        conn.insert("id", zbus::zvariant::Value::from(ssid.to_string()));
        conn.insert("type", zbus::zvariant::Value::from("802-11-wireless"));
        dict.insert("connection", conn);

        let mut wifi = HashMap::new();
        wifi.insert("ssid", zbus::zvariant::Value::from(ssid.as_bytes().to_vec()));
        wifi.insert("security", zbus::zvariant::Value::from("802-11-wireless-security"));
        dict.insert("802-11-wireless", wifi);

        let mut sec = HashMap::new();
        sec.insert("key-mgmt", zbus::zvariant::Value::from("wpa-eap"));
        dict.insert("802-11-wireless-security", sec);

        let mut eap = HashMap::new();
        eap.insert("eap", zbus::zvariant::Value::from(vec![eap_method.to_string()]));
        eap.insert("identity", zbus::zvariant::Value::from(identity.to_string()));
        // 1 = NM_SETTING_SECRET_FLAG_AGENT_OWNED
        eap.insert("password-flags", zbus::zvariant::Value::from(1u32));
        eap.insert("ca-cert", zbus::zvariant::Value::from(ca_cert_path.as_bytes().to_vec()));
        dict.insert("802-1x", eap);

        let mut ipv4 = HashMap::new();
        ipv4.insert("method", zbus::zvariant::Value::from("auto"));
        dict.insert("ipv4", ipv4);

        let mut ipv6 = HashMap::new();
        ipv6.insert("method", zbus::zvariant::Value::from("auto"));
        dict.insert("ipv6", ipv6);

        dict
    }

    pub fn build_vpn_tunnel_dict(
        name: &str,
        vpn_type: &str,
        config_path: &str,
        content: &str,
    ) -> HashMap<&'static str, HashMap<&'static str, zbus::zvariant::Value<'static>>> {
        let mut dict = HashMap::new();

        let mut conn = HashMap::new();
        conn.insert("id", zbus::zvariant::Value::from(name.to_string()));
        conn.insert("type", zbus::zvariant::Value::from(vpn_type.to_string()));
        dict.insert("connection", conn);

        let mut vpn = HashMap::new();
        let service_type = if vpn_type.contains('.') {
            vpn_type.to_string()
        } else {
            format!("org.freedesktop.NetworkManager.{}", vpn_type)
        };
        vpn.insert("service-type", zbus::zvariant::Value::from(service_type));

        let mut data_map: HashMap<String, String> = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with('[') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                data_map.insert(k.trim().to_string(), v.trim().to_string());
            } else if let Some((k, v)) = trimmed.split_once(' ') {
                data_map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        if data_map.is_empty() {
            data_map.insert("config".to_string(), config_path.to_string());
        }
        vpn.insert("data", zbus::zvariant::Value::from(data_map));
        dict.insert("vpn", vpn);

        dict
    }

    /// Safe extractor for top-level DBus setting dictionary
    pub fn extract_setting<'a>(
        dict: &'a HashMap<&'static str, HashMap<&'static str, zbus::zvariant::Value<'static>>>,
        setting_name: &str,
    ) -> Result<&'a HashMap<&'static str, zbus::zvariant::Value<'static>>, anyhow::Error> {
        dict.get(setting_name)
            .ok_or_else(|| anyhow::anyhow!("Missing DBus setting: '{}'", setting_name))
    }

    /// Safe extractor for DBus setting value
    pub fn extract_setting_value<'a>(
        setting: &'a HashMap<&'static str, zbus::zvariant::Value<'static>>,
        key: &str,
    ) -> Result<&'a zbus::zvariant::Value<'static>, anyhow::Error> {
        setting.get(key)
            .ok_or_else(|| anyhow::anyhow!("Missing key '{}' in setting dictionary", key))
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::Value;

    #[test]
    fn test_build_enterprise_wifi_dict() {
        let dict = Network::build_enterprise_wifi_dict(
            "Corporate-SSID",
            "user@ermete.os",
            "peap",
            "/etc/ssl/certs/ca.pem",
        );

        let conn = dict.get("connection").expect("missing 'connection' setting");
        assert_eq!(conn.get("id").unwrap(), &Value::from("Corporate-SSID"));
        assert_eq!(conn.get("type").unwrap(), &Value::from("802-11-wireless"));

        let wifi = dict.get("802-11-wireless").expect("missing '802-11-wireless' setting");
        assert_eq!(wifi.get("ssid").unwrap(), &Value::from("Corporate-SSID".as_bytes().to_vec()));
        assert_eq!(wifi.get("security").unwrap(), &Value::from("802-11-wireless-security"));

        let sec = dict.get("802-11-wireless-security").expect("missing '802-11-wireless-security' setting");
        assert_eq!(sec.get("key-mgmt").unwrap(), &Value::from("wpa-eap"));

        let eap = dict.get("802-1x").expect("missing '802-1x' setting");
        assert_eq!(eap.get("eap").unwrap(), &Value::from(vec!["peap".to_string()]));
        assert_eq!(eap.get("identity").unwrap(), &Value::from("user@ermete.os"));
        assert_eq!(eap.get("password-flags").unwrap(), &Value::from(1u32));
        assert_eq!(eap.get("ca-cert").unwrap(), &Value::from("/etc/ssl/certs/ca.pem".as_bytes().to_vec()));

        let ipv4 = dict.get("ipv4").expect("missing 'ipv4' setting");
        assert_eq!(ipv4.get("method").unwrap(), &Value::from("auto"));

        let ipv6 = dict.get("ipv6").expect("missing 'ipv6' setting");
        assert_eq!(ipv6.get("method").unwrap(), &Value::from("auto"));
    }

    #[test]
    fn test_build_vpn_tunnel_dict_from_file() {
        let tmp_path = "/tmp/test_ermete_vpn.conf";
        let content = "remote = vpn.ermete.os\nport = 1194\n";
        if let Err(e) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(tmp_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, content.as_bytes()))
        {
            tracing::error!("Failed to write temporary network file {:?}: {:?}", tmp_path, e);
        }
        let dict = Network::build_vpn_tunnel_dict("Ermete-VPN", "openvpn", tmp_path, content);

        let conn = dict.get("connection").expect("missing 'connection' setting");
        assert_eq!(conn.get("id").unwrap(), &Value::from("Ermete-VPN"));
        assert_eq!(conn.get("type").unwrap(), &Value::from("openvpn"));

        let vpn = dict.get("vpn").expect("missing 'vpn' setting");
        assert_eq!(vpn.get("service-type").unwrap(), &Value::from("org.freedesktop.NetworkManager.openvpn"));

        let data = vpn.get("data").expect("missing 'data' dictionary");
        if let Value::Dict(d) = data {
            let remote_key = Value::from("remote");
            let remote: Value = d.get::<_, Value<'_>>(&remote_key).unwrap().unwrap().clone();
            assert_eq!(remote, Value::from("vpn.ermete.os"));
            let port_key = Value::from("port");
            let port: Value = d.get::<_, Value<'_>>(&port_key).unwrap().unwrap().clone();
            assert_eq!(port, Value::from("1194"));
        } else {
            panic!("expected Dict for vpn.data");
        }
        if let Err(e) = std::fs::remove_file(tmp_path) {
            tracing::error!("Failed to remove temporary network file {:?}: {:?}", tmp_path, e);
        }
    }
}

