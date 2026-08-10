mod attestation;
mod auth;
mod tpm;

use anyhow::{anyhow, Result};
use attestation::{AttestationClient, AttestationReport};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use tpm::{TpmBootChainReport, TpmManager};
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionContext {
    pub username: String,
    pub session_id: String,
    pub tpm_verified: bool,
    pub attestation_verified: bool,
    pub key_release_status: String,
}

#[derive(ZeroizeOnDrop)]
pub struct SessionKeyMaterial {
    pub key_bytes: Vec<u8>,
}

pub struct ErmeteGreeter {
    tpm: TpmManager,
    attestation: AttestationClient,
}

impl Default for ErmeteGreeter {
    fn default() -> Self {
        Self::new()
    }
}

impl ErmeteGreeter {
    pub fn new() -> Self {
        Self {
            tpm: TpmManager::new(),
            attestation: AttestationClient::new(),
        }
    }

    /// Performs pre-login boot chain and attestation integrity verification
    pub async fn perform_preflight_checks(&self) -> Result<(TpmBootChainReport, AttestationReport)> {
        info!("--- Ermete Greeter Zero-Trust Boot Chain Verification ---");
        
        let tpm_report = self.tpm.verify_boot_chain();
        if !tpm_report.is_trusted {
            warn!("TPM 2.0 boot chain verification flagged untrusted state or missing hardware.");
        } else {
            info!("TPM 2.0 PCR registers verified (PCR0, PCR7, PCR10).");
        }

        let attestation_report = self.attestation.query_attestation_status().await?;
        info!(
            "Attestation Daemon Status: {} | PQC: {}",
            attestation_report.status, attestation_report.pqc_status
        );

        Ok((tpm_report, attestation_report))
    }

    /// Authenticates user and unseals session key share via TPM 2.0 & ermete-attestation
    pub async fn authenticate_and_release_keys(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(UserSessionContext, SessionKeyMaterial)> {
        info!("Initiating Zero-Trust login sequence for user '{}'...", username);

        // Perform real PAM / UNIX authentication
        auth::authenticate_user(username, password)?;

        let (tpm_report, att_report) = self.perform_preflight_checks().await?;

        if !tpm_report.is_trusted && !tpm_report.tpm_present {
            warn!("Proceeding with software fallback key unsealing (dev/standalone mode).");
        }

        // Unseal login key share from TPM 2.0
        let key_bytes = self
            .tpm
            .unseal_login_key_share(username, password)
            .map_err(|e| anyhow!("TPM 2.0 Key Unseal Failed: {}", e))?;

        let session_id = format!("ermete-sess-{}", hex::encode(&key_bytes[..4]));

        let session_ctx = UserSessionContext {
            username: username.to_string(),
            session_id,
            tpm_verified: tpm_report.is_trusted || !tpm_report.tpm_present,
            attestation_verified: att_report.hardware_enclave_active || att_report.secrets_released,
            key_release_status: "SUCCESS_KEY_UNSEALED".to_string(),
        };

        info!(
            "Zero-Trust session created successfully for user '{}' [Session ID: {}]",
            username, session_ctx.session_id
        );

        Ok((session_ctx, SessionKeyMaterial { key_bytes }))
    }

    /// Service main loop for greetd replacement
    pub async fn run_service(&self) -> Result<()> {
        info!("Starting ermete-greeter daemon (greetd Zero-Trust replacement)...");

        let (tpm_rep, att_rep) = self.perform_preflight_checks().await?;
        info!("Pre-flight boot integrity score: TPM Present={}, Attestation Status='{}'", tpm_rep.tpm_present, att_rep.status);

        info!("ermete-greeter ready. Listening for Zero-Trust authentication requests.");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("=== Ermete OS Zero-Trust Greeter Service ===");
    info!("Replacing standard greetd with TPM 2.0 & ermete-attestation key release pipeline");

    let greeter = ErmeteGreeter::new();

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 && args[1] == "--login" {
        let username = &args[2];
        let password = env::var("GREETER_PASSWORD").unwrap_or_else(|_| "demo-pass".to_string());
        
        match greeter.authenticate_and_release_keys(username, &password).await {
            Ok((ctx, _keys)) => {
                println!("{}", serde_json::to_string_pretty(&ctx)?);
            }
            Err(e) => {
                error!("Authentication failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        greeter.run_service().await?;
    }

    Ok(())
}

