pub mod builder;
pub mod hardware;

use std::error::Error;
use tokio::signal;
use tracing::info;
use zbus::interface;

pub struct KernelForgeDaemon;

#[interface(name = "org.ermete.KernelForge")]
impl KernelForgeDaemon {
    /// D-Bus Method: ForgeHardwareTailoredKernel
    /// Extracts local kernel sources, detects CPU/hardware flags (-march=native),
    /// executes Gentoo-style LTO/AutoFDO kernel build with driver pruning,
    /// and forges a super-optimized Unified Kernel Image (UKI).
    async fn forge_hardware_tailored_kernel(&self) -> zbus::fdo::Result<String> {
        info!("Received D-Bus call: ForgeHardwareTailoredKernel");
        match builder::run_kernel_forge().await {
            Ok(res) => Ok(res.message),
            Err(e) => Err(zbus::fdo::Error::Failed(format!("Kernel Forge Failed: {}", e))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    info!("==========================================================================");
    info!("🧬 Ermete OS Gentoo-Style Hardware-Tailored Kernel Forge Daemon Starting");
    info!("   D-Bus Service: org.ermete.KernelForge");
    info!("   Object Path:   /org/ermete/KernelForge");
    info!("==========================================================================");

    let daemon = KernelForgeDaemon;
    
    // Try registering on session bus first, or fallback gracefully
    let conn_builder = match zbus::connection::Builder::session() {
        Ok(b) => b,
        Err(_) => zbus::connection::Builder::system()?,
    };

    let _conn = conn_builder
        .name("org.ermete.KernelForge")?
        .serve_at("/org/ermete/KernelForge", daemon)?
        .build()
        .await?;

    info!("🚀 D-Bus Service org.ermete.KernelForge successfully exported and listening!");

    signal::ctrl_c().await?;
    info!("Clean shutdown of Ermete Kernel Forge daemon.");
    Ok(())
}
