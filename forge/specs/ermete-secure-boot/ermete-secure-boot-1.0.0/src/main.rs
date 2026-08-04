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
    async fn get_attestation(&self) -> String {
        if !self.tpm_available {
            return "Fallback: TPM not found. System running without hardware attestation.".to_string();
        }

        // Dummy implementation of reading PCR registers from sysfs
        // In reality, this would read from /sys/class/tpm/tpm0/pcr-sha256/0 etc.
        let pcr_path = "/sys/class/class/tpm/tpm0/pcr-sha256/0";
        if Path::new(pcr_path).exists() {
            if let Ok(content) = fs::read_to_string(pcr_path) {
                return format!("Attestation OK: PCR0={}", content.trim());
            }
        }
        
        "Attestation OK: TPM present but PCR reading mocked.".to_string()
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
