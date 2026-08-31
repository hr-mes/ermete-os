#![deny(unsafe_code)]
pub mod ecs;

mod animation;
mod backend;
mod dbus_listener;
mod desktop_state;
mod input_routing;
mod ipc;
mod screencopy;
mod state;
mod tiling;

use anyhow::{Context, Result};
use backend::{DrmBackendConfig, DrmKmsBackend};
use ecs::SharedEcsWorld;
use ipc::IpcServer;
use state::CompositorState;
use std::sync::atomic::Ordering;
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

    // Step 1: Initialize SharedEcsWorld at compositor startup
    let ecs_world = SharedEcsWorld::new();

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

    // Initialize shared compositor state with connected SharedEcsWorld
    let state = Arc::new(Mutex::new(CompositorState::new(drm_backend)));
    state.lock().await.desktop_state.ecs_world = ecs_world.clone();

    // Initialize and run IPC server for AI auto-tiling instructions
    let ipc_server = IpcServer::new(Arc::clone(&state));
    let socket_path = ipc_server.socket_path().to_path_buf();

    let ipc_handle = tokio::spawn(async move {
        if let Err(err) = ipc_server.run().await {
            tracing::error!("IPC server fatal error: {}", err);
        }
    });

    // Step 2 & 3: Unified VBlank-anchored native frame loop
    // Replaces the legacy 1000Hz CPU-hogging polling timers with a single event-driven loop
    let frame_ecs_world = ecs_world.clone();
    let frame_state = Arc::clone(&state);
    let native_frame_handle = tokio::spawn(async move {
        let mut render_state = ecs::systems::render::CompositorState::new(144.0);
        let mut last_tick = tokio::time::Instant::now();

        loop {
            // Anchor to native VBlank / Wayland frame callback.
            // (e.g. state.lock().await.drm_backend.wait_for_vblank().await)
            // Simulated hardware VBlank event wait here:
            tokio::time::sleep(std::time::Duration::from_nanos(6_944_444)).await;

            let now = tokio::time::Instant::now();
            let dt = (now - last_tick).as_secs_f64();
            last_tick = now;
            let capped_dt = dt.min(0.05); // Cap dt for numerical safety

            // 1. Tick Physics
            if let Ok(mut world) = frame_ecs_world.write() {
                ecs::systems::physics::spring_physics_system_batch(&mut world, capped_dt);
            }

            // 2. Tick Animation & Desktop State
            {
                let mut state_guard = frame_state.lock().await;
                if state_guard
                    .appearance_dirty
                    .swap(false, Ordering::Acquire)
                {
                    state_guard.apply_pending_appearance();
                }
                state_guard.tick_animation(capped_dt);
            }

            // 3. Render System
            ecs::systems::render::render_system(&frame_ecs_world, &mut render_state);
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
        _ = native_frame_handle => {
            tracing::warn!("Native frame loop task terminated.");
        }
    }

    // Cleanup socket if created
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            tracing::warn!("No previous socket to remove at {:?}: {:?}", socket_path, e);
        }
    }

    info!("Ermete Compositor gracefully stopped.");
    Ok(())
}


