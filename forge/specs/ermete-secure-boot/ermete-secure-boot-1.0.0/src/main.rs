use std::fs;
use std::path::Path;
use zbus::{connection, interface, Result};
use tokio::time::{sleep, Duration};

struct SecureBootAttestation {
    tpm_available: bool,
}

#[interface(name = "org.ermete.SecureBoot")]
impl SecureBootAttestation {
    async fn get_attestation(&self) -> zbus::fdo::Result<String> {
        let secure_boot_path = "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c";
        let pk_path = "/sys/firmware/efi/efivars/PK-8be4df61-93ca-11d2-aa0d-00e098032b8c";

        let sb_state = if let Ok(data) = fs::read(secure_boot_path) {
            if data.len() >= 5 && data[4] == 1 {
                "Enabled"
            } else {
                "Disabled"
            }
        } else {
            "Unknown/NotSupported"
        };

        let pk_enrolled = if Path::new(pk_path).exists() {
            "Enrolled"
        } else {
            "NotEnrolled"
        };

        let tpm_state = if self.tpm_available {
            let pcr_path = "/sys/class/tpm/tpm0/pcr-sha256/0";
            match fs::read_to_string(pcr_path) {
                Ok(content) => format!("PCR0={}", content.trim()),
                Err(_) => "TPM Error".to_string(),
            }
        } else {
            "No TPM".to_string()
        };

        Ok(format!("SecureBoot: {}, PK: {}, TPM: {}", sb_state, pk_enrolled, tpm_state))
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
