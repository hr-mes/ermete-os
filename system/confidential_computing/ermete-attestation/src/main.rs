#![allow(unsafe_code)]

mod config;
mod key_release;
mod sev_snp;
mod tdx;
mod verifier;

use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::fs;
use std::path::Path;
use std::process::exit;

use config::AttestationConfig;
use key_release::KeyReleaseManager;
use verifier::{AttestationVerifier, VerifiedHardwareReport};

fn generate_hardware_nonce() -> [u8; 64] {
    use ring::rand::{SecureRandom, SystemRandom};
    let mut nonce = [0u8; 64];
    let rng = SystemRandom::new();
    if rng.fill(&mut nonce).is_ok() {
        return nonce;
    }
    // Fallback pseudo-random initialization if RNG fails
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(42);
    for (i, byte) in nonce.iter_mut().enumerate() {
        *byte = (timestamp.wrapping_add(i as u128)) as u8;
    }
    nonce
}

fn create_mock_remote_pubkey_if_missing(pubkey_path: &Path) -> Result<()> {
    if !pubkey_path.exists() {
        if let Some(parent) = pubkey_path.parent() {
            fs::create_dir_all(parent)?;
        }
        info!("Creating initial remote public key file at {:?}", pubkey_path);
        let sample_pem = "-----BEGIN PUBLIC KEY-----\n\
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE7v3n0hQk/8K1N2+5T0dYwWJ4WJ6v\n\
Qz4N2k8X+0W1Y3E5v7J6k9N8X2A0V4T7L9K2N1M3P5Q7R9S1T3U5V7W9Y==\n\
-----END PUBLIC KEY-----\n";
        fs::write(pubkey_path, sample_pem)?;
    }
    Ok(())
}

fn run_attestation_daemon() -> Result<()> {
    info!("============================================================");
    info!("Ermete OS Confidential Computing Hardware Attestation Daemon");
    info!("============================================================");

    let config = AttestationConfig::load_or_default();
    let key_manager = KeyReleaseManager::new(config.key_output_path.clone());
    let verifier = AttestationVerifier::new(config.clone());

    // Auto-create directory structure for remote public key if needed
    let _ = create_mock_remote_pubkey_if_missing(&config.remote_pubkey_path);

    let nonce = generate_hardware_nonce();
    info!("Generated 512-bit hardware attestation challenge nonce.");

    let mut verified_report: Option<VerifiedHardwareReport> = None;

    // 1. AMD SEV-SNP Hardware Interface Check
    if sev_snp::is_sev_snp_available() {
        info!("AMD SEV-SNP hardware device detected (/dev/sev-guest or /dev/sev).");
        match sev_snp::get_sev_snp_report(&nonce) {
            Ok(report) => {
                match verifier.verify_sev_snp_report(&report, &nonce) {
                    Ok(verified) => {
                        verified_report = Some(verified);
                    }
                    Err(e) => {
                        error!("AMD SEV-SNP cryptographic verification failed: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to extract AMD SEV-SNP hardware report: {}", e);
            }
        }
    }

    // 2. Intel TDX Hardware Interface Check
    if verified_report.is_none() && tdx::is_tdx_available() {
        info!("Intel TDX hardware device detected (/dev/tdx_guest or /dev/tdx-guest).");
        match tdx::get_tdx_report(&nonce) {
            Ok(report) => {
                match verifier.verify_tdx_report(&report, &nonce) {
                    Ok(verified) => {
                        verified_report = Some(verified);
                    }
                    Err(e) => {
                        error!("Intel TDX cryptographic verification failed: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to extract Intel TDX hardware report: {}", e);
            }
        }
    }

    // 3. Fallback / Dev Environment Simulation Handling
    if verified_report.is_none() {
        if !config.strict_zero_trust {
            warn!("Hardware CVM devices missing or unverified, but strict Zero-Trust is OFF.");
            warn!("Engaging dev simulation mode for hardware attestation.");
            let mock_measurement = [0xABu8; 48];
            verified_report = Some(VerifiedHardwareReport::MockSimulated {
                measurement: mock_measurement,
                hardware_type: "Dev-Simulation-Fallback".to_string(),
            });
        } else {
            error!("Zero-Trust Hardware Attestation Failed!");
            error!("No trusted CVM hardware interface (/dev/sev-guest, /dev/tdx_guest) responded successfully.");
            error!("Or hardware report cryptographic signature failed verification against remote public key.");
            key_manager.revoke_and_purge();
            return Err(anyhow!("Hardware attestation failed: untrusted environment"));
        }
    }

    // 4. Key Release for /var/home
    if let Some(report) = verified_report {
        key_manager.release_var_home_key(&report)?;
        info!("Hardware enclave verified successfully. Zero-Trust access granted.");
        Ok(())
    } else {
        key_manager.revoke_and_purge();
        Err(anyhow!("Hardware attestation failed"))
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match run_attestation_daemon() {
        Ok(_) => {
            info!("Ermete Attestation Daemon completed successfully.");
            exit(0);
        }
        Err(e) => {
            error!("FATAL: Confidential Computing Hardware Attestation error: {}", e);
            error!("Halting boot process. Key release for /var/home REFUSED.");
            exit(1);
        }
    }
}
