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

#[interface(name = "os.ermete.VoiceOver")]
impl VoiceOverService {
    /// Parla il testo specificato (solo se VoiceOver è attivo nel sistema)
    async fn speak(&self, text: String) -> zbus::fdo::Result<()> {
        let is_enabled = self.state.lock().await.voiceover_enabled;
        if !is_enabled {
            return Ok(());
        }

        // Interrompiamo lo speech precedente e parliamo il nuovo usando speech-dispatcher (spd-say)
        let _ = tokio::process::Command::new("killall")
            .arg("spd-say")
            .output()
            .await;

        let _ = tokio::process::Command::new("spd-say")
            .arg("-l")
            .arg("it") // Lingua italiana
            .arg("-r")
            .arg("10") // Un po' più veloce
            .arg(&text)
            .spawn();

        Ok(())
    }
    
    /// Stoppa immediatamente la lettura corrente
    async fn stop(&self) -> zbus::fdo::Result<()> {
        let _ = tokio::process::Command::new("killall")
            .arg("spd-say")
            .output()
            .await;
        let _ = tokio::process::Command::new("spd-say")
            .arg("-S")
            .output()
            .await;
        Ok(())
    }
}
