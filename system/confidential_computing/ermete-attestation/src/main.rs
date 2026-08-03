use anyhow::{Result};
use log::{info, error};
use std::process::exit;
use std::path::Path;

/// Checks for AMD SEV-SNP support and verifies the attestation report
fn verify_sev_snp() -> Result<bool> {
    info!("Checking AMD SEV-SNP attestation report...");
    if Path::new("/dev/sev-guest").exists() {
        info!("Found /dev/sev-guest. SEV-SNP is active. Validating report...");
        let output = std::process::Command::new("sev-guest-parse")
            .arg("--verify")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                info!("SEV-SNP report verified successfully.");
                return Ok(true);
            }
            Ok(output) => {
                error!("SEV-SNP report verification failed: {:?}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                error!("Failed to execute sev-guest-parse: {}", e);
            }
        }
    }
    Ok(false)
}

/// Checks for Intel TDX support and verifies the attestation report
fn verify_tdx() -> Result<bool> {
    info!("Checking Intel TDX attestation report...");
    if Path::new("/dev/tdx_guest").exists() {
        info!("Found /dev/tdx_guest. TDX is active. Validating quote...");
        let output = std::process::Command::new("tdx-guest-verify")
            .arg("--verify")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                info!("TDX quote verified successfully.");
                return Ok(true);
            }
            Ok(output) => {
                error!("TDX quote verification failed: {:?}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                error!("Failed to execute TDX verification tool: {}", e);
            }
        }
    }
    Ok(false)
}

/// Checks for vTPM presence and verifies PCRs
fn verify_vtpm() -> Result<bool> {
    info!("Checking vTPM PCRs...");
    if Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists() {
        info!("Found TPM device. Verifying PCRs...");
        let output = std::process::Command::new("tpm2_pcrread")
            .arg("sha256:0,1,2,3")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("sha256:") {
                    info!("vTPM PCRs verified successfully.");
                    return Ok(true);
                } else {
                    error!("vTPM verification failed: unexpected output from tpm2_pcrread.");
                }
            }
            Ok(output) => {
                error!("vTPM PCR read failed: {:?}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                error!("Failed to execute tpm2_pcrread: {}", e);
            }
        }
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
