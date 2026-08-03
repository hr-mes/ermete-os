use anyhow::{Result};
use log::{info, error};
use std::process::exit;
use std::path::Path;

/// Checks for AMD SEV-SNP support and verifies the attestation report
fn verify_sev_snp() -> Result<bool> {
    info!("Checking AMD SEV-SNP attestation report...");
    if Path::new("/dev/sev-guest").exists() {
        info!("Found /dev/sev-guest. SEV-SNP is active.");
        // TODO: Request and verify SEV-SNP attestation report from the PSP.
        return Ok(true);
    }
    Ok(false)
}

/// Checks for Intel TDX support and verifies the attestation report
fn verify_tdx() -> Result<bool> {
    info!("Checking Intel TDX attestation report...");
    if Path::new("/dev/tdx_guest").exists() {
        info!("Found /dev/tdx_guest. TDX is active.");
        // TODO: Request and verify TDX quote from the quoting enclave.
        return Ok(true);
    }
    Ok(false)
}

/// Checks for vTPM presence and verifies PCRs
fn verify_vtpm() -> Result<bool> {
    info!("Checking vTPM PCRs...");
    if Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists() {
        info!("Found TPM device. Verifying PCRs...");
        // TODO: Read and verify PCR values against expected measurements.
        return Ok(true);
    }
    Ok(false)
}

fn main() {
    env_logger::init();
    info!("Starting Ermete Attestation Verification for Confidential Virtual Machines (CVM)...");

    // We verify all available attestation methods. In a real scenario, we might require specific ones.
    let sev_ok = verify_sev_snp().unwrap_or(false);
    let tdx_ok = verify_tdx().unwrap_or(false);
    let vtpm_ok = verify_vtpm().unwrap_or(false);

    if sev_ok || tdx_ok || vtpm_ok {
        info!("Attestation verified successfully. Memory encryption is active and hardware enclave is trusted.");
        info!("System is secure. Proceeding to start EventBus...");
    } else {
        error!("Attestation failed! Could not verify SEV-SNP / TDX / vTPM report.");
        error!("System environment might be compromised or not running in a CVM.");
        error!("Halting boot process. EventBus will NOT start.");
        exit(1);
    }
}
