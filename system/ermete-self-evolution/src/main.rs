#![allow(unsafe_code)]

pub mod autofdo;
pub mod codegraph_bridge;
pub mod engine;
pub mod hot_swap;
pub mod recompiler;
pub mod telemetry_listener;

use engine::SelfEvolutionEngine;
use std::env;
use std::time::Duration;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("==========================================================================");
    info!("🧬 Level 17 Autonomous Self-Evolving Code Graph & Runtime Engine Starting");
    info!("   Transforming Ermete OS into a Living Organism with AutoFDO & Recompilation");
    info!("==========================================================================");

    let workspace_root = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/var/home/ermete/GEMINI/ermete-os"));
    info!("Workspace Root: {:?}", workspace_root);

    let mut engine = SelfEvolutionEngine::new(workspace_root);
    let mut interval = tokio::time::interval(Duration::from_secs(3));

    info!("Level 17 Autonomous Control Loop Active. Monitoring for bottlenecks...");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = engine.run_evolution_cycle().await {
                    tracing::error!("Error during self-evolution cycle execution: {}", e);
                }
            }
            _ = signal::ctrl_c() => {
                info!("Received Ctrl-C signal. Shutting down Level 17 Self-Evolution Engine cleanly.");
                break;
            }
        }
    }

    let final_metrics = engine.get_metrics();
    info!(
        "🏁 Final Self-Evolution Summary: Total Cycles: {}, Neutralized Bottlenecks: {}, Rebuilds: {}, Hot-Swaps: {}",
        final_metrics.total_cycles,
        final_metrics.bottlenecks_neutralized,
        final_metrics.recompilations_completed,
        final_metrics.hot_swaps_applied
    );

    Ok(())
}
