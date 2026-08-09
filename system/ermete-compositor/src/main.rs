#![deny(unsafe_code)]

mod animation;
mod backend;
mod desktop_state;
mod ipc;
mod state;
mod tiling;


use anyhow::{Context, Result};
use backend::{DrmBackendConfig, DrmKmsBackend};
use ipc::IpcServer;
use state::CompositorState;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging subscriber
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("ermete_compositor=info"));

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(env_filter)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("Starting Ermete OS Native AI-Driven Wayland Compositor (ermete-compositor)...");

    // Initialize DRM/KMS backend
    let mut drm_backend = DrmKmsBackend::new(DrmBackendConfig::default());
    drm_backend
        .initialize()
        .context("Failed to initialize DRM/KMS backend")?;

    let is_headless = drm_backend.is_headless();
    let active_cards = drm_backend.active_cards().to_vec();

    info!(
        "Compositor DRM/KMS status: mode={}, active_cards={:?}",
        if is_headless { "Headless" } else { "KMS DRM Direct" },
        active_cards
    );

    // Initialize shared compositor state
    let state = Arc::new(Mutex::new(CompositorState::new(drm_backend)));

    // Initialize and run IPC server for AI auto-tiling instructions
    let ipc_server = IpcServer::new(Arc::clone(&state));
    let socket_path = ipc_server.socket_path().to_path_buf();

    let ipc_handle = tokio::spawn(async move {
        if let Err(err) = ipc_server.run().await {
            tracing::error!("IPC server fatal error: {}", err);
        }
    });

    // Spawn 1000 Hz Mass-Spring-Damper physics frame tick loop (Unlocked Framerate for 360Hz+ Monitors)
    let anim_state = Arc::clone(&state);
    let anim_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(1));
        let mut last_tick = tokio::time::Instant::now();
        loop {
            interval.tick().await;
            let now = tokio::time::Instant::now();
            let dt = (now - last_tick).as_secs_f64();
            last_tick = now;

            let dt = dt.min(0.05); // Cap dt for numerical safety on lag spikes
            let mut state_guard = anim_state.lock().await;
            state_guard.tick_animation(dt);
        }
    });

    info!("Ermete Compositor scaffolding ready.");
    info!("Listening for AI-driven tiling commands at {:?}", socket_path);

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal. Shutting down compositor...");
        }
        _ = ipc_handle => {
            tracing::warn!("IPC server task terminated.");
        }
        _ = anim_handle => {
            tracing::warn!("Animation frame tick task terminated.");
        }
    }

    // Cleanup socket if created
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    info!("Ermete Compositor gracefully stopped.");
    Ok(())
}
