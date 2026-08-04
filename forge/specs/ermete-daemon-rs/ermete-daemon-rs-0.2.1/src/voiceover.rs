use zbus::interface;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::settings::SettingsState;

pub struct VoiceOverService {
    state: Arc<Mutex<SettingsState>>,
}

impl VoiceOverService {
    pub fn new(state: Arc<Mutex<SettingsState>>) -> Self {
        Self { state }
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

#[interface(name = "os.ermete.VoiceOver")]
impl VoiceOverService {
    /// Parla il testo specificato (solo se VoiceOver è attivo nel sistema)
    async fn speak(&self, text: String) -> zbus::fdo::Result<()> {
        let is_enabled = self.state.lock().await.voiceover_enabled;
        if !is_enabled {
            return Ok(());
        }

        if let Ok(conn) = zbus::Connection::session().await {
            if let Ok(worker) = VoiceOverWorkerProxy::new(&conn).await {
                let _ = worker.speak(&text).await;
            }
        }

        Ok(())
    }
    
    /// Stoppa immediatamente la lettura corrente
    async fn stop(&self) -> zbus::fdo::Result<()> {
        if let Ok(conn) = zbus::Connection::session().await {
            if let Ok(worker) = VoiceOverWorkerProxy::new(&conn).await {
                let _ = worker.stop().await;
            }
        }
        Ok(())
    }
}
