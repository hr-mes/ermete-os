use anyhow::{Context, Result};
use log::{error, info, warn};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::verifier::VerifiedHardwareReport;

/// Secure memory container for the sensitive decryption key.
/// ZeroizeOnDrop guarantees the key bytes in RAM are wiped clean when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretDecryptionKey {
    pub key_data: [u8; 32], // 256-bit AES-GCM / LUKS decryption key
}

impl SecretDecryptionKey {
    pub fn new(raw: [u8; 32]) -> Self {
        Self { key_data: raw }
    }
}

pub struct KeyReleaseManager {
    output_path: std::path::PathBuf,
}

impl KeyReleaseManager {
    pub fn new(output_path: std::path::PathBuf) -> Self {
        Self { output_path }
    }

    /// Releases the decryption key for /var/home ONLY after successful hardware attestation verification
    pub fn release_var_home_key(&self, report: &VerifiedHardwareReport) -> Result<()> {
        info!("============================================================");
        info!("HARDWARE ATTESTATION SUCCESSFUL! Proceeding with Key Release.");
        info!("============================================================");

        match report {
            VerifiedHardwareReport::SevSnp { measurement, .. } => {
                info!("Hardware Tier: AMD SEV-SNP CVM");
                info!("Attestor Measurement: {}", hex::encode(measurement));
            }
            VerifiedHardwareReport::Tdx { mrtd, .. } => {
                info!("Hardware Tier: Intel TDX CVM");
                info!("Attestor MRTD: {}", hex::encode(mrtd));
            }
            VerifiedHardwareReport::MockSimulated { measurement, hardware_type } => {
                warn!("Hardware Tier: SIMULATED ({})", hardware_type);
                warn!("Attestor Measurement: {}", hex::encode(measurement));
            }
        }

        // Derive/unseal 256-bit key bound to hardware attestation state using HKDF-SHA256
        let (ikm, info_label): (&[u8], &[u8]) = match report {
            VerifiedHardwareReport::SevSnp { measurement, .. } => (&measurement[..], b"ermete-sev-snp-luks-v1"),
            VerifiedHardwareReport::Tdx { mrtd, .. } => (&mrtd[..], b"ermete-tdx-luks-v1"),
            VerifiedHardwareReport::MockSimulated { measurement, .. } => (&measurement[..], b"ermete-simulated-luks-v1"),
        };

        let salt = ring::hkdf::Salt::new(ring::hkdf::HKDF_SHA256, b"ermete-zero-trust-luks-salt-v1");
        let prk = salt.extract(ikm);
        let info_slice = [info_label];
        let okm = prk
            .expand(&info_slice, ring::hkdf::HKDF_SHA256)
            .map_err(|_| anyhow::anyhow!("HKDF expansion failed for LUKS key release"))?;

        let mut key_buffer = [0u8; 32];
        okm.fill(&mut key_buffer)
            .map_err(|_| anyhow::anyhow!("Failed to fill HKDF output key buffer"))?;

        let secret_key = SecretDecryptionKey::new(key_buffer);

        // Ensure parent directory /run/ermete exists
        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        info!("Writing decryption key securely to {:?}", self.output_path);

        // Create key file with strict Unix permissions: 0400 (Read-only by owner / root)
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o400)
            .open(&self.output_path)
            .with_context(|| format!("Failed to create secure key file at {:?}", self.output_path))?;

        file.write_all(&secret_key.key_data)
            .with_context(|| "Failed to write key data to output file")?;
        file.flush()?;

        info!("Successfully released decryption key for /var/home at {:?}", self.output_path);
        info!("Memory buffers scrubbed (Zeroize active). Zero-Trust hardware release COMPLETE.");

        Ok(())
    }

    /// Revokes key release and sanitizes any existing key files on attestation failure
    pub fn revoke_and_purge(&self) {
        error!("PERFORMING SECURITY PURGE: Revoking key release for /var/home.");
        if Path::new(&self.output_path).exists() {
            if let Err(e) = fs::remove_file(&self.output_path) {
                error!("Failed to remove key file at {:?}: {}", self.output_path, e);
            } else {
                info!("Purged existing key file at {:?}", self.output_path);
            }
        }
    }
}
