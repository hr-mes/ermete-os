use zbus::interface;
use tracing::info;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedValue, Type, Value};
use crate::wipe::WipeEngine;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitSubject {
    pub kind: String,
    pub details: HashMap<String, OwnedValue>,
}

impl PolkitSubject {
    pub fn system_bus_name(name: impl Into<String>) -> Self {
        let mut details = HashMap::new();
        let val: Value = Value::from(name.into());
        if let Ok(owned) = val.try_into() {
            details.insert("name".to_string(), owned);
        }
        Self {
            kind: "system-bus-name".to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitAuthorizationResult {
    pub is_authorized: bool,
    pub is_challenge: bool,
    pub details: HashMap<String, String>,
}

#[zbus::proxy(
    interface = "org.freedesktop.PolicyKit1.Authority",
    default_service = "org.freedesktop.PolicyKit1",
    default_path = "/org/freedesktop/PolicyKit1/Authority"
)]
pub trait PolicyKitAuthority {
    fn check_authorization(
        &self,
        subject: &PolkitSubject,
        action_id: &str,
        details: &HashMap<&str, &str>,
        flags: u32,
        cancellation_id: &str,
    ) -> zbus::Result<PolkitAuthorizationResult>;
}

pub async fn check_polkit_auth_zbus(
    conn: &zbus::Connection,
    sender: &str,
    action_id: &str,
    allow_user_interaction: bool,
) -> Result<bool, zbus::Error> {
    if let Ok(creds) = conn.peer_creds().await {
        if creds.unix_user_id() == Some(0) {
            return Ok(true);
        }
    }

    let proxy = PolicyKitAuthorityProxy::new(conn).await?;
    let subject = PolkitSubject::system_bus_name(sender);
    let details = HashMap::<&str, &str>::new();
    let flags = if allow_user_interaction { 1u32 } else { 0u32 };

    let result = proxy
        .check_authorization(&subject, action_id, &details, flags, "")
        .await?;

    Ok(result.is_authorized)
}

pub struct MdmIface;

#[interface(name = "os.ermete.Mdm")]
impl MdmIface {
    /// Manually trigger a local device wipe (e.g. from the UI before giving PC away)
    async fn trigger_local_wipe(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to trigger LOCAL WIPE.");

        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.mdm.wipe", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit authorization check failed: {}", e)))?;
            
        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        
        let engine = WipeEngine::new();
        
        // This is extremely dangerous, requires Polkit auth
        match engine.execute_cryptsetup_erase(None).await {
            Ok(_) => Ok("Wipe initiated. System halting.".into()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }
}
