extern crate serde;
use zbus::interface;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use zbus::fdo;
use zbus::message::Header;
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

pub async fn check_polkit_auth(sender: Option<&str>, action_id: &str) -> bool {
    let sender_str = match sender {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };

    let conn = match zbus::Connection::system().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Polkit Zero-Trust] Failed to connect to system bus: {}", e);
            return false;
        }
    };

    let proxy = match PolicyKitAuthorityProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[Polkit Zero-Trust] Failed to create PolicyKit authority proxy: {}", e);
            return false;
        }
    };

    let subject = PolkitSubject::system_bus_name(sender_str);
    let details = HashMap::<&str, &str>::new();

    match proxy.check_authorization(&subject, action_id, &details, 0, "").await {
        Ok(result) => result.is_authorized,
        Err(e) => {
            eprintln!("[Polkit Zero-Trust] CheckAuthorization D-Bus call failed for action {}: {}", action_id, e);
            false
        }
    }
}

#[derive(Clone)]
pub struct Bedrock {
    volume: Arc<AtomicU64>,
}

impl Default for Bedrock {
    fn default() -> Self {
        Self::new()
    }
}

impl Bedrock {
    pub fn new() -> Self {
        Self {
            volume: Arc::new(AtomicU64::new(0.5f64.to_bits())),
        }
    }
}

#[zbus::proxy(
    interface = "os.ermete.AudioWorker",
    default_service = "os.ermete.AudioWorker",
    default_path = "/os/ermete/AudioWorker"
)]
trait AudioWorker {
    fn set_volume(&self, volume: f64) -> zbus::Result<()>;
}

static SESSION_CONN: tokio::sync::OnceCell<Option<zbus::Connection>> = tokio::sync::OnceCell::const_new();

async fn get_session_conn() -> Option<zbus::Connection> {
    let conn_opt = SESSION_CONN.get_or_init(|| async {
        match zbus::Connection::session().await {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::error!("Failed to connect to zbus session bus: {:?}", e);
                None
            }
        }
    }).await;
    conn_opt.clone()
}

#[interface(name = "os.ermete.Bedrock")]
impl Bedrock {
    async fn ping(&self) -> String {
        if let Some(patched) = crate::live_patch::LivePatchManager::global().dispatch("ping", "") {
            return patched;
        }
        "pong".to_string()
    }

    /// ZBus API to load a dynamic shared library (.so) for zero-downtime hot-patching of method logic.
    async fn apply_live_patch(
        &self,
        so_path: String,
        #[zbus(header)] hdr: Header<'_>,
    ) -> fdo::Result<String> {
        let sender = hdr.sender().map(|s| s.as_str());

        if !check_polkit_auth(sender, "os.ermete.livepatcher.apply").await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed for live patching".into()));
        }

        crate::live_patch::LivePatchManager::global()
            .load_patch_so(&so_path)
            .map_err(fdo::Error::Failed)
    }

    /// Retrieve live patching status metadata as JSON
    async fn get_live_patch_status(&self) -> String {
        let status = crate::live_patch::LivePatchManager::global().get_status();
        serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string())
    }

    #[zbus(property, name = "Volume")]
    async fn audio_volume(&self) -> f64 {
        f64::from_bits(self.volume.load(Ordering::Relaxed))
    }

    #[zbus(property, name = "Volume")]
    async fn set_audio_volume(
        &self,
        val: f64,
        #[zbus(header)] hdr: Option<Header<'_>>,
    ) -> fdo::Result<()> {
        let sender = hdr.as_ref().and_then(|h| h.sender()).map(|s| s.as_str());
        if !check_polkit_auth(sender, "os.ermete.bedrock.setvolume").await {
            return Err(fdo::Error::AccessDenied("Polkit authorization failed".into()));
        }
        self.volume.store(val.to_bits(), Ordering::Relaxed);
        
        if let Some(conn) = get_session_conn().await {
            if let Ok(worker) = AudioWorkerProxy::new(&conn).await {
                let _ = worker.set_volume(val).await;
            }
        }
        Ok(())
    }
}

