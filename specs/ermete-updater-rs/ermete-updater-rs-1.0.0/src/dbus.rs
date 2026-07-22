use zbus::interface;
use tracing::{info, warn};
use crate::engine::{UpdateEngine, UpdateStatus};

pub struct UpdaterIface;

#[interface(name = "os.ermete.Updater")]
impl UpdaterIface {
    /// Applies updates interactively. Will require Polkit authentication.
    async fn apply_updates(&self, #[zbus(header)] hdr: zbus::MessageHeader<'_>, #[zbus(connection)] _conn: &zbus::Connection) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to apply updates.");
        
        let sender = hdr.sender().ok_or(zbus::fdo::Error::Failed("No sender".into()))?;
        let status = std::process::Command::new("pkcheck")
            .arg("--system-bus-name")
            .arg(sender.as_str())
            .arg("--action-id")
            .arg("os.ermete.updater.apply")
            .status()
            .map_err(|e| zbus::fdo::Error::Failed(format!("pkcheck failed: {}", e)))?;
            
        if !status.success() {
            return Err(zbus::fdo::Error::Failed("Polkit authorization failed".into()));
        }

        let mut engine = match UpdateEngine::new().await {
            Ok(e) => e,
            Err(err) => return Err(zbus::fdo::Error::Failed(format!("Failed to init engine: {}", err))),
        };
        
        match engine.check_and_apply().await {
            Ok(UpdateStatus::NoUpdates) => Ok("Nessun aggiornamento disponibile.".into()),
            Ok(UpdateStatus::Layer1AppliedLive) => Ok("Aggiornamento applicato LIVE (nessun riavvio richiesto).".into()),
            Ok(UpdateStatus::Layer0RebootRequired) => Ok("Aggiornamento Ring-0 applicato. Riavvio obbligatorio.".into()),
            Ok(UpdateStatus::Error(e)) => Err(zbus::fdo::Error::Failed(e)),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Errore: {}", e))),
        }
    }

    /// Checks for updates (dry run without apply)
    async fn check_updates(&self) -> std::result::Result<String, zbus::fdo::Error> {
        info!("Received D-Bus request to check updates.");
        Ok("Funzionalità check_updates da implementare...".into())
    }
}
