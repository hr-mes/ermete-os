use zbus::interface;
use tokio::sync::watch;
use crate::settings::SettingsState;

pub struct VoiceOverService {
    rx: watch::Receiver<SettingsState>,
}

impl VoiceOverService {
    pub fn new(rx: watch::Receiver<SettingsState>) -> Self {
        Self { rx }
    }
}

#[zbus::proxy(
    interface = "os.ermete.VoiceOverWorker",
    default_service = "os.ermete.VoiceOverWorker",
    default_path = "/os/ermete/VoiceOverWorker"
)]
trait VoiceOverWorker {
    fn speak(&self, text: &str) -> zbus::Result<()>;
    fn stop(&self) -> zbus::Result<()>;
}

static SESSION_CONN: tokio::sync::OnceCell<Option<zbus::Connection>> = tokio::sync::OnceCell::const_new();

async fn get_session_conn() -> Option<zbus::Connection> {
    let conn_opt = SESSION_CONN.get_or_init(|| async {
        zbus::Connection::session().await.ok()
    }).await;
    conn_opt.clone()
}

#[interface(name = "os.ermete.VoiceOver")]
impl VoiceOverService {
    /// Parla il testo specificato (solo se VoiceOver è attivo nel sistema)
    async fn speak(&self, text: String) -> zbus::fdo::Result<()> {
        let is_enabled = self.rx.borrow().voiceover_enabled;
        if !is_enabled {
            return Ok(());
        }

        if let Some(conn) = get_session_conn().await {
            if let Ok(worker) = VoiceOverWorkerProxy::new(&conn).await {
                let _ = worker.speak(&text).await;
            }
        }

        Ok(())
    }
    
    /// Stoppa immediatamente la lettura corrente
    async fn stop(&self) -> zbus::fdo::Result<()> {
        if let Some(conn) = get_session_conn().await {
            if let Ok(worker) = VoiceOverWorkerProxy::new(&conn).await {
                let _ = worker.stop().await;
            }
        }
        Ok(())
    }
}
