use crate::sched_ext::{SchedClass, SchedExtController, TaskSchedPolicy};
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use zbus::interface;
use zbus::zvariant::{OwnedValue, Type, Value};

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
    if let Ok(creds) = conn.peer_credentials().await {
        if creds.uid() == Some(0) {
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

pub struct SchedExtDbusInterface {
    controller: Arc<SchedExtController>,
}

impl SchedExtDbusInterface {
    pub fn new(controller: Arc<SchedExtController>) -> Self {
        Self { controller }
    }
}

#[interface(name = "os.ermete.SchedExt")]
impl SchedExtDbusInterface {
    /// Remote interface allowing external daemons to update `AI_SCHED_MAP`
    async fn update_sched_map(
        &self,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
        pid: u32,
        cpu_weight: u32,
        slice_us: u64,
        sched_class: u32,
        latency_target_us: u64,
    ) -> zbus::fdo::Result<String> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.ebpfsched.update", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for update_sched_map".into()));
        }

        let policy = TaskSchedPolicy {
            pid,
            class: SchedClass::from(sched_class),
            cpu_weight,
            slice_us,
            latency_target_us,
        };

        match self.controller.apply_task_policy(&policy).await {
            Ok(_) => Ok(serde_json::json!({
                "success": true,
                "pid": pid,
                "bpf_active": self.controller.sched_map().is_bpf_active().await,
                "message": format!("Successfully updated AI_SCHED_MAP for PID {}", pid)
            })
            .to_string()),
            Err(err) => Err(zbus::fdo::Error::Failed(format!("Failed to apply task policy for PID {}: {}", pid, err))),
        }
    }

    /// Query policy for a specific PID from `AI_SCHED_MAP`
    async fn get_sched_map(&self, pid: u32) -> String {
        match self.controller.sched_map().get_policy(pid).await {
            Some(val) => serde_json::json!({
                "found": true,
                "pid": val.pid,
                "cpu_weight": val.cpu_weight,
                "slice_us": val.slice_us,
                "sched_class": val.sched_class,
                "latency_target_us": val.latency_target_us,
                "flags": val.flags
            })
            .to_string(),
            None => serde_json::json!({
                "found": false,
                "pid": pid
            })
            .to_string(),
        }
    }

    /// List all policies registered in `AI_SCHED_MAP`
    async fn list_sched_map(&self) -> String {
        let policies = self.controller.sched_map().list_policies().await;
        let serialized: Vec<_> = policies
            .into_iter()
            .map(|(pid, val)| {
                serde_json::json!({
                    "pid": pid,
                    "cpu_weight": val.cpu_weight,
                    "slice_us": val.slice_us,
                    "sched_class": val.sched_class,
                    "latency_target_us": val.latency_target_us
                })
            })
            .collect();

        serde_json::json!({
            "count": serialized.len(),
            "policies": serialized,
            "bpf_active": self.controller.sched_map().is_bpf_active().await
        })
        .to_string()
    }

    /// Status endpoint
    async fn status(&self) -> String {
        serde_json::json!({
            "daemon": "ermete-ebpf-sched",
            "version": "0.1.0",
            "sched_ext_enabled": self.controller.is_sched_ext_enabled(),
            "bpf_map_active": self.controller.sched_map().is_bpf_active().await,
            "status": "ONLINE"
        })
        .to_string()
    }
}
