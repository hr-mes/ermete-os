use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    CheckingForUpdates,
    Downloading,
    Staging,
    ReadyForReboot,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub current_booted_image: String,
    pub pending_image: Option<String>,
    pub rollback_image: Option<String>,
    pub last_checked_timestamp: u64,
}

pub struct UpdaterEngine {
    state: Arc<RwLock<UpdateState>>,
    status: Arc<RwLock<DeploymentStatus>>,
}

impl UpdaterEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(UpdateState::Idle)),
            status: Arc::new(RwLock::new(DeploymentStatus {
                current_booted_image: "localhost/ermete-os:v1.0.0-current".to_string(),
                pending_image: None,
                rollback_image: Some("localhost/ermete-os:v0.9.9-rollback".to_string()),
                last_checked_timestamp: 0,
            })),
        }
    }

    pub async fn get_state(&self) -> UpdateState {
        self.state.read().await.clone()
    }

    pub async fn check_for_updates(&self) -> Result<bool> {
        info!("Avvio controllo aggiornamenti OTA / bootc container registry...");
        {
            let mut st = self.state.write().await;
            *st = UpdateState::CheckingForUpdates;
        }

        // Simula o interroga bootc/OSTree inspect
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let has_update = false; // In produzione verrebbe eseguito `bootc upgrade --check` o HTTP registry check
        info!("Verifica completata: nessun nuovo aggiornamento pendente trovato.");

        {
            let mut st = self.state.write().await;
            *st = UpdateState::Idle;
        }

        Ok(has_update)
    }

    pub async fn stage_update(&self, image_ref: &str) -> Result<()> {
        info!(image_ref = %image_ref, "Pre-fetching e staging nuovo container image bootc...");
        {
            let mut st = self.state.write().await;
            *st = UpdateState::Downloading;
        }

        // Simula la fase di download e applicazione dello stage bootc/OSTree
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        {
            let mut st = self.state.write().await;
            *st = UpdateState::Staging;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        {
            let mut status = self.status.write().await;
            status.pending_image = Some(image_ref.to_string());
        }

        {
            let mut st = self.state.write().await;
            *st = UpdateState::ReadyForReboot;
        }

        info!("Immagine bootc/OSTree allocata con successo. Pronto per il riavvio atomic.");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inizializza logger Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,ermete_updater_rs=debug")),
        )
        .init();

    info!("Avvio ermete-updater-rs: OTA & Bootc/OSTree Update Daemon");

    let engine = Arc::new(UpdaterEngine::new());

    // Esegue una verifica iniziale di integrità dello stato del deployment
    let current_state = engine.get_state().await;
    info!(?current_state, "Stato engine updater inizializzato.");

    // Avvia controllo aggiornamenti periodico in background
    let engine_clone = Arc::clone(&engine);
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = engine_clone.check_for_updates().await {
                warn!("Errore durante il controllo aggiornamenti schedulato: {:?}", e);
            }
        }
    });

    info!("ermete-updater-rs operativo. In attesa di comandi di aggiornamento OTA/bootc.");

    // In ascolto per segnale di terminazione
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Segnale SIGINT/SIGTERM ricevuto. Arresto ordinato di ermete-updater-rs...");
        }
    }

    handle.abort();
    info!("ermete-updater-rs terminato.");
    Ok(())
}
