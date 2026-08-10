use std::fs;
use std::path::Path;
use zbus::{connection, interface, Result};
use tokio::time::{sleep, Duration};

struct SecureBootAttestation {
    tpm_available: bool,
}

#[interface(name = "org.ermete.SecureBoot")]
impl SecureBootAttestation {
    /// Measure PCRs and return attestation status.
    async fn get_attestation(&self) -> zbus::fdo::Result<String> {
        if !self.tpm_available {
            return Ok("Fallback: TPM not found. System running without hardware attestation.".to_string());
        }

        let pcr_path = "/sys/class/tpm/tpm0/pcr-sha256/0";
        let content = fs::read_to_string(pcr_path).map_err(|e| {
            zbus::fdo::Error::Failed(format!("Impossibile leggere il registro PCR0 da {}: {}", pcr_path, e))
        })?;

        Ok(format!("Attestation OK: PCR0={}", content.trim()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check if TPM chip is present
    let tpm_available = Path::new("/sys/class/tpm/tpm0").exists();
    
    let attestation = SecureBootAttestation { tpm_available };

    let _conn = connection::Builder::system()?
        .name("org.ermete.SecureBoot")?
        .serve_at("/org/ermete/SecureBoot", attestation)?
        .build()
        .await?;

    println!("Ermete Secure Boot Daemon running. TPM available: {}", tpm_available);
    
    // Prevent the daemon from exiting
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
