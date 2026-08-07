
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod bedrock;
mod network;
mod bluetooth;
mod settings;
mod portal;
mod portal_screencast;

mod voiceover;
mod qos;
mod live_patch;


use std::error::Error;
use zbus::connection::Builder;
use bedrock::Bedrock;
use network::Network;
use bluetooth::Bluetooth;
use settings::SettingsService;
use portal::PortalSettingsService;
use portal_screencast::{PortalScreenCastService, PortalRemoteDesktopService};

use voiceover::VoiceOverService;

use tokio_util::sync::CancellationToken;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, warn, error};
use tracing_subscriber::EnvFilter;

fn init_telemetry() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,ermete_daemon_rs=debug"))
        )
        .with_target(true)
        .init();
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    init_telemetry();
    info!("Initializing Ermete Daemon telemetry and subsystems...");

    info!("Connecting to system D-Bus for NetworkManager & BlueZ integration...");
    let sys_conn = zbus::Connection::system().await?;

    info!("Skipping PowerManager and Gatekeeper Listener (now independent microservices)...");

    info!("Starting Spatial Audio Raytracing engine & App Nap QoS observer...");
    qos::start_qos_observer(cancel_token_qos()).await;

    info!("Starting Continuity & Handoff daemon...");

    info!("Initializing ACID Settings Engine and XDG Desktop Portal backend...");
    let cancel_token = CancellationToken::new();
    let appearance_store = settings::AppearanceStateStore::new_async().await;
    let voiceover_store = settings::VoiceOverStateStore::new_async().await;
    let settings_srv = SettingsService::new_with_token(
        appearance_store.state_tx.clone(),
        voiceover_store.state_tx.clone(),
        cancel_token.clone(),
    );
    let portal_srv = PortalSettingsService::new(appearance_store.state_rx.clone());
    let screencast_srv = PortalScreenCastService::new();
    let remotedesktop_srv = PortalRemoteDesktopService::new(screencast_srv.clone());
    let voiceover_srv = VoiceOverService::new(voiceover_store.state_rx.clone());

    info!("Starting Ermete Bedrock Session Daemon on /os/ermete/Bedrock & /org/ermete/Settings...");
    let session_conn = Builder::session()?
        .name("os.ermete.Bedrock")?
        .name("org.ermete.Settings")?
        .name("os.ermete.VoiceOver")?
        .name("org.freedesktop.impl.portal.desktop.ermete")?
        .serve_at("/os/ermete/Bedrock", Bedrock::new())?
        .serve_at("/os/ermete/Bedrock/Network", Network::new(sys_conn.clone()))?
        .serve_at("/os/ermete/Bedrock/Bluetooth", Bluetooth::new(sys_conn.clone()))?
        .serve_at("/org/ermete/Settings", settings_srv.clone())?
        .serve_at("/os/ermete/Bedrock/Settings", settings_srv)?
        .serve_at("/os/ermete/VoiceOver", voiceover_srv)?
        .serve_at("/org/freedesktop/portal/desktop", portal_srv)?
        .serve_at("/org/freedesktop/portal/desktop", screencast_srv)?
        .serve_at("/org/freedesktop/portal/desktop", remotedesktop_srv)?
        .build()
        .await?;

    info!("Ermete Bedrock & Settings Daemon started and serving natively over zbus.");

    // Signal listener task for SIGINT (Ctrl+C), SIGTERM (shutdown), and SIGHUP (reload)
    let sig_token = cancel_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).ok();
        let mut sighup = signal(SignalKind::hangup()).ok();
        let ctrl_c = tokio::signal::ctrl_c();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received SIGINT (Ctrl+C). Initiating graceful shutdown...");
            }
            _ = async {
                if let Some(ref mut sig) = sigterm {
                    sig.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                info!("Received SIGTERM. Initiating graceful shutdown...");
            }
            _ = async {
                if let Some(ref mut sig) = sighup {
                    sig.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                info!("Received SIGHUP (Reload requested). Initiating graceful reload...");
            }
        }
        sig_token.cancel();
    });

    // Wait until cancellation is requested
    cancel_token.cancelled().await;

    info!("Closing ZBus connections and cleaning up resources...");
    if let Err(e) = session_conn.close().await {
        error!(error = %e, "Error closing session D-Bus connection");
    } else {
        info!("Session D-Bus connection closed cleanly.");
    }

    if let Err(e) = sys_conn.close().await {
        error!(error = %e, "Error closing system D-Bus connection");
    } else {
        info!("System D-Bus connection closed cleanly.");
    }

    info!("Ermete daemon shutdown complete.");
    Ok(())
}

fn cancel_token_qos() -> CancellationToken {
    CancellationToken::new()
}


