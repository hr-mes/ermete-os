use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::connection::Builder;

mod aggregator;
mod ai_engine;
mod collector;
mod dbus;
mod oracle_bridge;
mod security;

use aggregator::BatchAggregator;
use ai_engine::AiPredictiveEngine;
use collector::JournalCollector;
use dbus::{TelemetryDbusInterface, TelemetryMetrics};
use oracle_bridge::OracleBridge;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Logging / Tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // Apply Capability Dropping Security Hardening
    security::apply_telemetry_hardening();

    info!("==================================================");
    info!("Starting Ermete OS AI Predictive Telemetry Daemon");

    info!("Fronte 3: Log-Aggregator Predittivo AI & Self-Healing");
    info!("==================================================");

    // 2. Setup Async MPSC Communication Channels
    let (record_tx, record_rx) = mpsc::channel(1000);
    let (batch_tx, batch_rx) = mpsc::channel(100);
    let (report_tx, report_rx) = mpsc::channel(50);

    let metrics = Arc::new(TelemetryMetrics::new());

    // 3. Instantiate Subsystems
    let collector = JournalCollector::new(record_tx);
    let aggregator = BatchAggregator::new(
        record_rx,
        batch_tx,
        20,                          // max batch size
        Duration::from_millis(2000), // batch max flush latency
    );
    let ai_engine = Arc::new(AiPredictiveEngine::new(report_tx.clone()).await);
    let oracle_bridge = OracleBridge::new(report_rx).await;

    // 4. Register DBus Service: org.ermete.Telemetry
    let dbus_interface = TelemetryDbusInterface::new(metrics.clone(), report_tx.clone());
    let connection_builder = Builder::system();

    let dbus_conn = match connection_builder {
        Ok(b) => match b.name("org.ermete.Telemetry") {
            Ok(b2) => match b2.serve_at("/org/ermete/Telemetry", dbus_interface) {
                Ok(b3) => match b3.build().await {
                    Ok(conn) => {
                        info!("DBus service 'org.ermete.Telemetry' registered on System Bus.");
                        Some(conn)
                    }
                    Err(_) => None,
                },
                Err(_) => None,
            },
            Err(_) => None,
        },
        Err(_) => None,
    };

    let _active_dbus = if dbus_conn.is_none() {
        if let Ok(b_session) = Builder::session() {
            let dbus_interface2 = TelemetryDbusInterface::new(metrics.clone(), report_tx.clone());
            if let Ok(b2) = b_session.name("org.ermete.Telemetry") {
                if let Ok(b3) = b2.serve_at("/org/ermete/Telemetry", dbus_interface2) {
                    if let Ok(conn) = b3.build().await {
                        info!("DBus service 'org.ermete.Telemetry' registered on Session Bus (fallback).");
                        Some(conn)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        dbus_conn
    };

    // 5. Spawn Async Micro-service Actor Tasks
    let collector_task = tokio::spawn(async move {
        if let Err(e) = collector.run_loop().await {
            error!("Journal collector error: {}", e);
        }
    });

    let aggregator_task = tokio::spawn(async move {
        aggregator.run_loop().await;
    });

    let ai_engine_task = tokio::spawn(async move {
        ai_engine.run_loop(batch_rx).await;
    });

    let oracle_bridge_task = tokio::spawn(async move {
        oracle_bridge.run_loop().await;
    });

    info!("🚀 ermete-telemetry daemon fully operational in async event-driven mode.");

    // 6. Graceful Shutdown Signal Handler
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT signal. Initiating graceful shutdown of ermete-telemetry...");
        }
        res = collector_task => {
            if let Err(e) = res {
                error!("Collector task terminated unexpectedly: {}", e);
            }
        }
        res = aggregator_task => {
            if let Err(e) = res {
                error!("Aggregator task terminated unexpectedly: {}", e);
            }
        }
        res = ai_engine_task => {
            if let Err(e) = res {
                error!("AI engine task terminated unexpectedly: {}", e);
            }
        }
        res = oracle_bridge_task => {
            if let Err(e) = res {
                error!("Oracle bridge task terminated unexpectedly: {}", e);
            }
        }
    }

    info!("👋 ermete-telemetry daemon stopped.");
    Ok(())
}
