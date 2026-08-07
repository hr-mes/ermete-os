mod attestation;
mod dbus;
mod enclave;
mod kvm;
mod sandbox;

use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use std::env;

use attestation::{AttestationConfig, AttestationEngine};
use dbus::run_hypervisor_dbus_services;
use enclave::EnclaveManager;
use kvm::detect_capabilities;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("============================================================");
    info!("Ermete OS Level 16 Zero-Trust Hardware Micro-Hypervisor Daemon");
    info!("Confidential Micro-VM Enclave Orchestrator (AMD SEV-SNP / Intel TDX)");
    info!("============================================================");

    let caps = detect_capabilities();
    info!("Host Hypervisor Capabilities: KVM={}, SEV-SNP={}, TDX={}, Default Enclave={}",
        caps.kvm_available, caps.sev_snp_supported, caps.tdx_supported, caps.default_enclave_type);

    let mut config = AttestationConfig::default();

    // Parse command-line flags
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--strict") {
        info!("Strict zero-trust hardware attestation mode ENABLED.");
        config.strict_zero_trust = true;
    }

    let attestation_engine = Arc::new(AttestationEngine::new(config));
    let enclave_manager = Arc::new(EnclaveManager::new(attestation_engine.clone()));

    // If --test-enclave flag is passed, perform test launch and exit
    if args.iter().any(|arg| arg == "--test-enclave") {
        info!("Executing test enclave launch...");
        let enclave_id = enclave_manager.launch_enclave(
            "test-app",
            "/bin/sleep",
            &["2".to_string()],
            None,
            sandbox::UntrustedAgentCategory::UntrustedTool,
        )?;
        info!("Test enclave created: {}", enclave_id);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        enclave_manager.terminate_enclave(&enclave_id)?;
        info!("Test enclave terminated successfully.");
        return Ok(());
    }

    // Connect and run D-Bus services on org.ermete.Hypervisor and org.ermete.AttestationAlarm
    if let Err(e) = run_hypervisor_dbus_services(enclave_manager.clone(), attestation_engine.clone()).await {
        error!("FATAL: Micro-Hypervisor D-Bus service failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
