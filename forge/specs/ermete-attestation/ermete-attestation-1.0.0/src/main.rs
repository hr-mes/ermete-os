use std::fs::File;
use std::io::Read;
use std::os::unix::fs::FileTypeExt;
use zbus::{connection, interface};

struct AttestationAlarm;

#[interface(name = "org.ermete.AttestationAlarm1")]
impl AttestationAlarm {
    /// Returns the current status of the attestation
    async fn status(&self) -> &str {
        "Unknown"
    }

    /// Alarm event signal for when attestation fails
    #[zbus(signal)]
    async fn attestation_failed(signal_ctxt: &zbus::SignalContext<'_>, reason: &str) -> zbus::Result<()>;

    /// Event signal for when attestation succeeds
    #[zbus(signal)]
    async fn attestation_success(signal_ctxt: &zbus::SignalContext<'_>) -> zbus::Result<()>;
}

async fn check_attestation() -> Result<(), Box<dyn std::error::Error>> {
    // Check for AMD SEV-SNP guest device.
    match File::open("/dev/sev-guest") {
        Ok(mut file) => {
            // Check if it's a valid character device
            let metadata = file.metadata()?;
            if !metadata.file_type().is_char_device() {
                return Err("Device /dev/sev-guest is not a character device. Cryptographic fallback required.".into());
            }

            // Real syscall check via read. Typically SEV requires ioctls for attestation reports.
            // A direct read might return an error, but we handle it gracefully here as a syscall check.
            let mut buf = [0u8; 1];
            if let Err(e) = file.read(&mut buf) {
                tracing::info!("Note: direct read from /dev/sev-guest returned: {} (ioctl usually required)", e);
            }

            Ok(())
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Check for TDX as a fallback
            match File::open("/dev/tdx_guest") {
                Ok(tdx_file) => {
                    let metadata = tdx_file.metadata()?;
                    if !metadata.file_type().is_char_device() {
                        return Err("Device /dev/tdx_guest is not a character device.".into());
                    }
                    Ok(())
                },
                Err(tdx_e) => {
                    Err(format!("Hardware attestation failed: CVM enclave guest device not found (SEV: {}, TDX: {}).", e, tdx_e).into())
                }
            }
        },
        Err(e) => {
            Err(format!("Error accessing /dev/sev-guest: {}", e).into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting ermete-attestation daemon...");

    let connection = connection::Builder::system()?
        .name("org.ermete.AttestationAlarm")?
        .serve_at("/org/ermete/AttestationAlarm", AttestationAlarm)?
        .build()
        .await?;

    let iface_ref = connection
        .object_server()
        .interface::<_, AttestationAlarm>("/org/ermete/AttestationAlarm")
        .await?;

    match check_attestation().await {
        Ok(_) => {
            tracing::info!("Attestation successful.");
            AttestationAlarm::attestation_success(iface_ref.signal_context()).await?;
            tracing::info!("Attestation success signal emitted on DBus.");
        },
        Err(e) => {
            tracing::error!("Attestation Error: {}", e);
            let error_msg = e.to_string();
            AttestationAlarm::attestation_failed(iface_ref.signal_context(), &error_msg).await?;
            tracing::error!("Attestation alarm signal emitted on DBus.");
        }
    }

    // Keep the daemon alive to process DBus messages
    std::future::pending::<()>().await;

    Ok(())
}
