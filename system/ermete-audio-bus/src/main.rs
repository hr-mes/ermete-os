use anyhow::Result;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod controller;
mod dbus;
mod inspector;
mod node_tree;
mod pipewire_manager;
mod routing;
mod routing_service;
mod virtual_device;

use dbus::AudioBusInterface;
use node_tree::NodeTree;
use pipewire_manager::PipewireManager;
use routing::RoutingEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("--------------------------------------------------");
    info!("Starting Ermete OS Audio Bus Daemon (ermete-audio-bus)");
    info!("Pillar 5: Native Rust PipeWire Session Manager & Swarm Routing");
    info!("--------------------------------------------------");

    // 2. Initialize Audio Node Graph Representation
    let node_tree = NodeTree::new();

    // 3. Initialize Swarm Autonomous Routing Engine
    let routing_engine = Arc::new(RoutingEngine::new(node_tree.clone()));

    // 4. Initialize Native PipeWire Session Manager Engine
    let pw_manager = Arc::new(PipewireManager::new(node_tree.clone(), routing_engine.clone()));
    let pw_init_task = tokio::spawn({
        let pw_manager = pw_manager.clone();
        async move { pw_manager.initialize().await }
    });
    match pw_init_task.await {
        Ok(Ok(())) => info!("PipeWire manager initialized successfully."),
        Ok(Err(err)) => tracing::error!("Failed to initialize PipeWire manager: {}", err),
        Err(e) => tracing::warn!("PipeWire manager initialization status: {}", e),
    }

    // 5. Expose ZBus D-Bus Interface org.ermete.AudioBus
    let dbus_interface = AudioBusInterface::new(
        node_tree.clone(),
        routing_engine.clone(),
        pw_manager.clone(),
    );

    let _connection = Builder::session()?
        .name("org.ermete.AudioBus")?
        .serve_at("/org/ermete/AudioBus", dbus_interface)?
        .build()
        .await?;

    info!("D-Bus service 'org.ermete.AudioBus' bound at path '/org/ermete/AudioBus'");

    // 6. Spawn Background PipeWire Graph Monitoring Loop
    let pw_task_manager = pw_manager.clone();
    let monitor_task = tokio::spawn(async move {
        pw_task_manager.run_event_loop().await;
    });

    info!("Ermete OS Audio Bus Daemon is running continuously.");

    // 7. Wait for shutdown signal or task termination
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT shutdown signal, shutting down Audio Bus...");
        }
        res = monitor_task => {
            if let Err(e) = res {
                tracing::error!("PipeWire monitoring task joined with error: {}", e);
            }
        }
    }

    Ok(())
}
