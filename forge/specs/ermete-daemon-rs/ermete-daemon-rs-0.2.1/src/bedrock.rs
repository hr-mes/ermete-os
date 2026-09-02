extern crate serde;
use ermete_bus_api::polkit;
use zbus::interface;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use zbus::fdo;
use zbus::message::Header;

/// Checks a D-Bus caller's polkit authorization for `action_id`.
///
/// This daemon serves on the **session** bus, so `sender` (the unique name from the message
/// header) means nothing to polkit, which lives on the system bus. The caller is resolved to
/// a `unix-process` subject through the session bus driver, and that subject is sent to
/// polkit over a system-bus connection. Fails closed: a missing sender, a bus failure, or a
/// polkit error all yield `false`.
pub async fn check_polkit_auth(sender: Option<&str>, action_id: &str) -> bool {
    let sender = match sender {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    let Some(session) = get_session_conn().await else {
        return false;
    };
    let subject = match polkit::unix_process_subject(&session, sender).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("[Polkit Zero-Trust] cannot resolve caller {sender} on the session bus: {e}");
            return false;
        }
    };
    let system = match zbus::Connection::system().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("[Polkit Zero-Trust] failed to connect to system bus: {e}");
            return false;
        }
    };
    // AllowUserInteraction: os.ermete.network.configure and os.ermete.livepatcher.apply are
    // auth_admin* in this daemon's .policy file; without the flag polkit could never prompt.
    match polkit::check_subject(&system, &subject, action_id, true).await {
        Ok(authorized) => authorized,
        Err(e) => {
            tracing::error!("[Polkit Zero-Trust] CheckAuthorization failed for {action_id}: {e}");
            false
        }
    }
}
/// D-Bus object exposing `os.ermete.Bedrock`: a `ping` liveness check, the live-patch
/// control surface (delegates to [`crate::live_patch::LivePatchManager`]), and the
/// session audio volume property (mirrored to `os.ermete.AudioWorker`).
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
    /// Creates a new instance with volume initialized to `0.5`.
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

