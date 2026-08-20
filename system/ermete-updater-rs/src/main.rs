#![allow(clippy::needless_lifetimes)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::mut_from_ref)]
#![allow(clippy::new_without_default)]

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

        let output = tokio::process::Command::new("bootc")
            .arg("upgrade")
            .arg("--check")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let err_msg = format!("Controllo aggiornamenti bootc fallito: {}", stderr);
            {
                let mut st = self.state.write().await;
                *st = UpdateState::Failed(err_msg.clone());
            }
            anyhow::bail!(err_msg);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let has_update = stdout.contains("queued") || stdout.contains("Available") || stdout.contains("update available");

        info!("Verifica completata. Aggiornamento disponibile: {}", has_update);

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

        let mut cmd = tokio::process::Command::new("bootc");
        if image_ref.is_empty() {
            cmd.arg("upgrade");
        } else {
            cmd.arg("switch").arg(image_ref);
        }

        {
            let mut st = self.state.write().await;
            *st = UpdateState::Staging;
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let err_msg = format!("Staging immagine bootc fallito: {}", stderr);
            {
                let mut st = self.state.write().await;
                *st = UpdateState::Failed(err_msg.clone());
            }
            anyhow::bail!(err_msg);
        }

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
