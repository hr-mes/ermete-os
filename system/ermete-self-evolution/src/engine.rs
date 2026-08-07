use crate::autofdo::AutoFdoManager;
use crate::codegraph_bridge::CodeGraphBridge;
use crate::hot_swap::HotSwapper;
use crate::recompiler::RecompilerEngine;
use crate::telemetry_listener::TelemetryListener;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvolutionMetrics {
    pub total_cycles: u64,
    pub bottlenecks_neutralized: u64,
    pub recompilations_completed: u64,
    pub hot_swaps_applied: u64,
    pub codegraph_syncs_triggered: u64,
    pub average_speedup_pct: f64,
}

pub struct SelfEvolutionEngine {
    telemetry: TelemetryListener,
    autofdo: AutoFdoManager,
    codegraph: CodeGraphBridge,
    recompiler: RecompilerEngine,
    hot_swapper: HotSwapper,
    metrics: EvolutionMetrics,
}

impl SelfEvolutionEngine {
    pub fn new(root_dir: PathBuf) -> Self {
        let profile_dir = root_dir.join("target/autofdo_profiles");
        let staging_dir = root_dir.join("target/atomic_updates");
        let bin_install_dir = root_dir.join("target/release");

        Self {
            telemetry: TelemetryListener::new(20000), // Syscall threshold Hz
            autofdo: AutoFdoManager::new(profile_dir),
            codegraph: CodeGraphBridge::new(&root_dir),
            recompiler: RecompilerEngine::new(&root_dir),
            hot_swapper: HotSwapper::new(staging_dir, bin_install_dir),
            metrics: EvolutionMetrics {
                total_cycles: 0,
                bottlenecks_neutralized: 0,
                recompilations_completed: 0,
                hot_swaps_applied: 0,
                codegraph_syncs_triggered: 0,
                average_speedup_pct: 0.0,
            },
        }
    }

    /// Run a single iteration of the Autonomous Self-Evolution cycle
    pub async fn run_evolution_cycle(&mut self) -> Result<(), anyhow::Error> {
        self.metrics.total_cycles += 1;
        info!(
            "🧬 [Self-Evolution Engine] Beginning Living Organism Cycle #{}...",
            self.metrics.total_cycles
        );

        // 1. Poll Ring-0 Telemetry Listener for bottlenecks
        if let Some(alert) = self.telemetry.poll_bottleneck().await {
            info!("🎯 [Self-Evolution Engine] Bottleneck alert received: {:?}", alert);

            // 2. Query CodeGraph topology for impact analysis
            let impact_nodes = self.codegraph.query_bottleneck_impact(&alert.hotspot_symbol);
            info!("🕸️ [Self-Evolution Engine] CodeGraph topology returned {} affected nodes:", impact_nodes.len());
            for node in &impact_nodes {
                info!("   - Symbol: {} | Crate: {} | Call Depth: {}", node.symbol, node.crate_name, node.call_depth);
            }

            let target_crate = &alert.target_crate;

            // 3. Ingest AutoFDO profile
            let profile = self.autofdo.collect_runtime_profile(target_crate).await?;
            let rustflags = self.autofdo.get_autofdo_rustflags(&profile);

            // 4. Trigger background recompilation
            let comp_result = self
                .recompiler
                .trigger_recompilation(target_crate, &rustflags)
                .await?;
            self.metrics.recompilations_completed += 1;

            // 5. Perform atomic hot-swap or staging of optimized binary
            let hot_swap_report = self
                .hot_swapper
                .hot_swap_service(&comp_result.binary_path, target_crate)
                .await?;
            self.metrics.hot_swaps_applied += 1;

            // 6. Dynamic CodeGraph synchronization
            self.codegraph.sync_topology().await?;
            self.metrics.codegraph_syncs_triggered += 1;

            self.metrics.bottlenecks_neutralized += 1;
            self.metrics.average_speedup_pct =
                (self.metrics.average_speedup_pct * 0.7) + (profile.estimated_speedup_pct * 0.3);

            info!(
                "✨ [Self-Evolution Engine] Level 17 Singularity Cycle Complete! Target: '{}', Hot-Swap Status: '{}', Speedup: +{:.1}%",
                target_crate, hot_swap_report.status, profile.estimated_speedup_pct
            );
        } else {
            info!("🌱 [Self-Evolution Engine] System state optimal. No recompilation triggered.");
        }

        info!(
            "📊 [Evolution Scoreboard] Cycles: {} | Neutralized: {} | Rebuilds: {} | HotSwaps: {} | Speedup Gain: +{:.1}%",
            self.metrics.total_cycles,
            self.metrics.bottlenecks_neutralized,
            self.metrics.recompilations_completed,
            self.metrics.hot_swaps_applied,
            self.metrics.average_speedup_pct
        );

        Ok(())
    }

    pub fn get_metrics(&self) -> &EvolutionMetrics {
        &self.metrics
    }
}
