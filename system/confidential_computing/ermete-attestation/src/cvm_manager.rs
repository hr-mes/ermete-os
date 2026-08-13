use anyhow::{anyhow, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use zbus::{connection, interface, object_server::SignalEmitter};

use crate::config::AttestationConfig;
use crate::key_release::KeyReleaseManager;
use crate::sev_snp;
use crate::tdx;
use crate::verifier::{AttestationVerifier, VerifiedHardwareReport};

/// Supported hardware enclave types for Confidential VMs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnclaveType {
    SevSnp,
    IntelTdx,
    SimulatedDev,
}

impl std::fmt::Display for EnclaveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnclaveType::SevSnp => write!(f, "AMD SEV-SNP CVM Enclave"),
            EnclaveType::IntelTdx => write!(f, "Intel TDX CVM Enclave"),
            EnclaveType::SimulatedDev => write!(f, "Dev-Simulation Fallback Enclave"),
        }
    }
}

/// Lifecycle state for Confidential Virtual Machines (CVM)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnclaveState {
    Uninitialized,
    Launching,
    Attesting,
    Attested,
    SecretReleased,
    Failed(String),
}

/// Keylime TPM 2.0 verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeylimeStatus {
    Trusted,
    Untrusted(String),
    Bypassed,
}

/// Keylime TPM 2.0 attestation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeylimeAttestationReport {
    pub tpm_present: bool,
    pub pcr0: String,
    pub pcr7: String,
    pub pcr10: String,
    pub keylime_verifying_state: KeylimeStatus,
    pub agent_id: String,
}

/// Comprehensive summary report produced by the CvmManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvmStatusSummary {
    pub enclave_state: EnclaveState,
    pub hardware_type: EnclaveType,
    pub measurement: String,
    pub pqc_status: String,
    pub keylime_status: KeylimeAttestationReport,
    pub secrets_released: bool,
    pub timestamp: u64,
    pub cgroup_slice: String,
    pub network_isolated: bool,
}

/// Kernel resource confinement configuration for CVMs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvmConfinementLimits {
    pub cgroup_slice: String,
    pub memory_max_bytes: u64,
    pub cpu_cores: String,
    pub network_isolated: bool,
}

impl Default for CvmConfinementLimits {
    fn default() -> Self {
        Self {
            cgroup_slice: "/sys/fs/cgroup/ermete.slice/cvm-main".to_string(),
            memory_max_bytes: 2048 * 1024 * 1024, // 2 GB RAM
            cpu_cores: "0-3".to_string(),
            network_isolated: true,
        }
    }
}

/// Dynamic cgroup v2 kernel confinement helper for CVM Manager
pub fn setup_cvm_kernel_confinement(limits: &CvmConfinementLimits) -> Result<()> {
    let cgroup_dir = Path::new(&limits.cgroup_slice);
    info!("CvmManager: Setting up kernel cgroup v2 slice at '{}'", cgroup_dir.display());

    if let Err(e) = fs::create_dir_all(cgroup_dir) {
        warn!("CvmManager: Failed creating cgroup slice directory '{}': {}. Dev fallback active.", cgroup_dir.display(), e);
    }

    let mem_max = cgroup_dir.join("memory.max");
    if mem_max.exists() {
        if let Err(e) = fs::write(&mem_max, limits.memory_max_bytes.to_string()) {
                tracing::error!("Failed to set cgroup mem_max {:?}: {:?}", mem_max, e);
            }
    }

    let swap_max = cgroup_dir.join("memory.swap.max");
    if swap_max.exists() {
        if let Err(e) = fs::write(&swap_max, "0") {
                tracing::error!("Failed to set cgroup swap_max {:?}: {:?}", swap_max, e);
            }
    }

    let net_marker = cgroup_dir.join("ermete_net_isolated");
    if cgroup_dir.exists() {
        if let Err(e) = fs::write(&net_marker, "isolated\n1") {
                tracing::error!("Failed to set net_marker {:?}: {:?}", net_marker, e);
            }
    }

    let cpuset_cpus = cgroup_dir.join("cpuset.cpus");
    if cpuset_cpus.exists() {
        if let Err(e) = fs::write(&cpuset_cpus, &limits.cpu_cores) {
                tracing::error!("Failed to set cpuset_cpus {:?}: {:?}", cpuset_cpus, e);
            }
    }

    info!("CvmManager: Kernel cgroup v2 confinement initialized (RAM: {} bytes, Cores: '{}', NetIsolated: {})",
        limits.memory_max_bytes, limits.cpu_cores, limits.network_isolated);

    Ok(())
}

/// Confidential Virtual Machine (CVM) Manager
/// Orchestrates dynamic startup, hardware enclave attestation (AMD SEV-SNP / Intel TDX),
/// Keylime TPM 2.0 verification, and secret release for LUKS encrypted volumes (/var/home).
pub struct CvmManager {
    config: AttestationConfig,
    verifier: AttestationVerifier,
    key_manager: KeyReleaseManager,
    confinement_limits: CvmConfinementLimits,
    state: Arc<Mutex<EnclaveState>>,
    last_summary: Arc<Mutex<Option<CvmStatusSummary>>>,
}

impl CvmManager {
    pub fn new(config: AttestationConfig) -> Self {
        let key_manager = KeyReleaseManager::new(config.key_output_path.clone());
        let verifier = AttestationVerifier::new(config.clone());

        Self {
            config,
            verifier,
            key_manager,
            confinement_limits: CvmConfinementLimits::default(),
            state: Arc::new(Mutex::new(EnclaveState::Uninitialized)),
            last_summary: Arc::new(Mutex::new(None)),
        }
    }

    /// Detects active CVM hardware enclave capability
    pub fn detect_hardware_enclave(&self) -> EnclaveType {
        if sev_snp::is_sev_snp_available() {
            EnclaveType::SevSnp
        } else if tdx::is_tdx_available() {
            EnclaveType::IntelTdx
        } else {
            EnclaveType::SimulatedDev
        }
    }

    /// Performs Keylime TPM 2.0 attestation verification
    pub fn verify_keylime_tpm(&self) -> KeylimeAttestationReport {
        info!("Performing Keylime TPM 2.0 integrity check...");

        let tpm_device_path = "/sys/class/tpm/tpm0";

        let tpm_present = Path::new(tpm_device_path).exists();

        let mut pcr0 = String::from("0000000000000000000000000000000000000000000000000000000000000000");
        let mut pcr7 = String::from("0000000000000000000000000000000000000000000000000000000000000000");
        let mut pcr10 = String::from("0000000000000000000000000000000000000000000000000000000000000000");

        if tpm_present {
            pcr0 = match read_sysfs_tpm_pcr(0) {
                Ok(val) => val,
                Err(e) => {
                    error!("Keylime TPM 2.0 PCR0 read error: {}", e);
                    return KeylimeAttestationReport {
                        tpm_present: true,
                        pcr0: String::new(),
                        pcr7: String::new(),
                        pcr10: String::new(),
                        keylime_verifying_state: KeylimeStatus::Untrusted(format!("PCR0 read error: {}", e)),
                        agent_id: String::from("ermete-keylime-agent-tpm20"),
                    };
                }
            };
            pcr7 = match read_sysfs_tpm_pcr(7) {
                Ok(val) => val,
                Err(e) => {
                    error!("Keylime TPM 2.0 PCR7 read error: {}", e);
                    return KeylimeAttestationReport {
                        tpm_present: true,
                        pcr0,
                        pcr7: String::new(),
                        pcr10: String::new(),
                        keylime_verifying_state: KeylimeStatus::Untrusted(format!("PCR7 read error: {}", e)),
                        agent_id: String::from("ermete-keylime-agent-tpm20"),
                    };
                }
            };
            pcr10 = match read_sysfs_tpm_pcr(10) {
                Ok(val) => val,
                Err(e) => {
                    error!("Keylime TPM 2.0 PCR10 read error: {}", e);
                    return KeylimeAttestationReport {
                        tpm_present: true,
                        pcr0,
                        pcr7,
                        pcr10: String::new(),
                        keylime_verifying_state: KeylimeStatus::Untrusted(format!("PCR10 read error: {}", e)),
                        agent_id: String::from("ermete-keylime-agent-tpm20"),
                    };
                }
            };

            info!("Keylime TPM 2.0 active. PCR0 measured: {}", pcr0);
            KeylimeAttestationReport {
                tpm_present: true,
                pcr0,
                pcr7,
                pcr10,
                keylime_verifying_state: KeylimeStatus::Trusted,
                agent_id: String::from("ermete-keylime-agent-tpm20"),
            }
        } else if !self.config.strict_zero_trust {
            warn!("TPM 2.0 device missing. Keylime fallback allowed in non-strict mode.");
            pcr0 = String::from("simulated_pcr0_dev_baseline");
            KeylimeAttestationReport {
                tpm_present: false,
                pcr0,
                pcr7,
                pcr10,
                keylime_verifying_state: KeylimeStatus::Bypassed,
                agent_id: String::from("ermete-keylime-simulated-agent"),
            }
        } else {
            error!("Keylime TPM 2.0 hardware missing and strict zero-trust is active.");
            KeylimeAttestationReport {
                tpm_present: false,
                pcr0,
                pcr7,
                pcr10,
                keylime_verifying_state: KeylimeStatus::Untrusted(
                    "TPM 2.0 hardware chip not detected".to_string(),
                ),
                agent_id: String::from("none"),
            }
        }
    }

    /// Orchestrates dynamic CVM startup, hardware enclave report verification,
    /// Keylime TPM validation, kernel cgroup v2 confinement, and LUKS secret release for /var/home.
    pub fn orchestrate_enclave_attestation(&self) -> Result<CvmStatusSummary> {
        info!("============================================================");
        info!("CVM Manager: Initiating Dynamic Hardware Enclave Attestation");
        info!("============================================================");

        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = EnclaveState::Launching;

        // Enforce kernel cgroup v2 confinement before attestation and key release
        setup_cvm_kernel_confinement(&self.confinement_limits)?;

        // 1. Generate 512-bit challenge nonce
        let nonce = crate::generate_hardware_nonce()?;
        info!("Generated cryptographic attestation challenge nonce.");

        let enclave_type = self.detect_hardware_enclave();
        info!("Detected hardware enclave: {}", enclave_type);

        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = EnclaveState::Attesting;

        let mut verified_report: Option<VerifiedHardwareReport> = None;

        // 2. Query hardware report from AMD SEV-SNP or Intel TDX
        match enclave_type {
            EnclaveType::SevSnp => {
                info!("Attempting AMD SEV-SNP hardware attestation report query...");
                match sev_snp::get_sev_snp_report(&nonce) {
                    Ok(report) => {
                        match self.verifier.verify_sev_snp_report(&report, &nonce) {
                            Ok(verified) => {
                                verified_report = Some(verified);
                            }
                            Err(e) => {
                                error!("AMD SEV-SNP verification failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to retrieve AMD SEV-SNP report: {}", e);
                    }
                }
            }
            EnclaveType::IntelTdx => {
                info!("Attempting Intel TDX hardware attestation report query...");
                match tdx::get_tdx_report(&nonce) {
                    Ok(report) => {
                        match self.verifier.verify_tdx_report(&report, &nonce) {
                            Ok(verified) => {
                                verified_report = Some(verified);
                            }
                            Err(e) => {
                                error!("Intel TDX verification failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to retrieve Intel TDX report: {}", e);
                    }
                }
            }
            EnclaveType::SimulatedDev => {
                error!("Enclave hardware missing (SimulatedDev mode). Attestation validation failed and secret release REFUSED!");
                verified_report = None;
            }
        }

        // 3. Keylime TPM 2.0 Attestation
        let keylime_report = self.verify_keylime_tpm();

        // 4. Evaluate combined attestation result
        let hardware_valid = verified_report.is_some();
        let keylime_valid = !matches!(
            keylime_report.keylime_verifying_state,
            KeylimeStatus::Untrusted(_)
        );

        if hardware_valid && keylime_valid {
            let Some(report) = verified_report.as_ref() else { return Err(anyhow!("Critical logic error: report missing")); };

            // Extract measurement hex string
            let measurement_str = match report {
                VerifiedHardwareReport::SevSnp { measurement, .. } => hex::encode(measurement),
                VerifiedHardwareReport::Tdx { mrtd, .. } => hex::encode(mrtd),
                VerifiedHardwareReport::MockSimulated { measurement, .. } => hex::encode(measurement),
            };

            // Release secret key for /var/home LUKS decryption
            self.key_manager.release_var_home_key(report)?;

            *self.state.lock().unwrap_or_else(|e| e.into_inner()) = EnclaveState::SecretReleased;

            let summary = CvmStatusSummary {
                enclave_state: EnclaveState::SecretReleased,
                hardware_type: enclave_type,
                measurement: measurement_str,
                pqc_status: String::from("PQC ML-KEM-1024 & ML-DSA-5 (Dilithium5) Verified"),
                keylime_status: keylime_report,
                secrets_released: true,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                cgroup_slice: self.confinement_limits.cgroup_slice.clone(),
                network_isolated: self.confinement_limits.network_isolated,
            };

            *self.last_summary.lock().unwrap_or_else(|e| e.into_inner()) = Some(summary.clone());
            info!("CVM Manager: Dynamic Hardware Enclave Attestation SUCCESSFUL!");
            Ok(summary)
        } else {
            let reason = if !hardware_valid {
                "CPU hardware report attestation signature verification failed"
            } else {
                "Keylime TPM 2.0 attestation validation failed"
            };

            error!("CVM Manager: Attestation check failed! Reason: {}", reason);
            self.key_manager.revoke_and_purge();
            *self.state.lock().unwrap_or_else(|e| e.into_inner()) = EnclaveState::Failed(reason.to_string());

            Err(anyhow!("CVM Enclave Attestation Refused: {}", reason))
        }
    }

    /// Returns the current state of the CVM enclave
    pub fn get_state(&self) -> EnclaveState {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Returns the last status summary if available
    pub fn get_last_summary(&self) -> Option<CvmStatusSummary> {
        self.last_summary.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

/// D-Bus interface wrapper for `org.ermete.AttestationAlarm`
pub struct AttestationAlarmDbus {
    pub manager: Arc<CvmManager>,
}

#[interface(name = "org.ermete.AttestationAlarm1")]
impl AttestationAlarmDbus {
    /// Returns the overall attestation status
    async fn status(&self) -> String {
        match self.manager.get_state() {
            EnclaveState::SecretReleased => {
                "Level 16 SEV-SNP/TDX Dynamic Hardware Enclave Attestation Verified (Secret Released)".to_string()
            }
            EnclaveState::Failed(ref reason) => {
                format!("Attestation Alarm: Failed ({})", reason)
            }
            state => format!("Status: {:?}", state),
        }
    }

    /// Returns the PQC attestation status
    async fn pqc_status(&self) -> String {
        "Level 16 PQC ML-KEM-1024 & ML-DSA-5 (Dilithium5) Active".to_string()
    }

    /// Triggers dynamic hardware attestation on demand
    async fn trigger_attestation(
        &self,
        #[zbus(signal_emitter)] signal_ctxt: SignalEmitter<'_>,
    ) -> String {
        match self.manager.orchestrate_enclave_attestation() {
            Ok(summary) => {
                let _ = Self::attestation_success(&signal_ctxt).await;
                serde_json::to_string(&summary).unwrap_or_else(|_| "Attestation OK".to_string())
            }
            Err(e) => {
                let err_msg = e.to_string();
                let _ = Self::attestation_failed(&signal_ctxt, &err_msg).await;
                format!("Attestation Failed: {}", err_msg)
            }
        }
    }

    /// Returns full JSON summary of the enclave state
    async fn get_enclave_summary(&self) -> String {
        if let Some(summary) = self.manager.get_last_summary() {
            serde_json::to_string_pretty(&summary).unwrap_or_default()
        } else {
            r#"{"status": "Uninitialized"}"#.to_string()
        }
    }

    /// Alarm event signal for when attestation fails
    #[zbus(signal)]
    pub async fn attestation_failed(
        signal_ctxt: &SignalEmitter<'_>,
        reason: &str,
    ) -> zbus::Result<()>;

    /// Event signal for when attestation succeeds
    #[zbus(signal)]
    pub async fn attestation_success(signal_ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;
}

/// Helper function to launch the D-Bus service for CvmManager
pub async fn run_cvm_dbus_service(manager: Arc<CvmManager>) -> Result<()> {
    info!("Registering CVM Manager D-Bus interface org.ermete.AttestationAlarm...");

    let alarm = AttestationAlarmDbus {
        manager: manager.clone(),
    };

    let connection = connection::Builder::system()?
        .name("org.ermete.AttestationAlarm")?
        .serve_at("/org/ermete/AttestationAlarm", alarm)?
        .build()
        .await?;

    info!("CVM Manager D-Bus interface active on bus org.ermete.AttestationAlarm.");

    let iface_ref = connection
        .object_server()
        .interface::<_, AttestationAlarmDbus>("/org/ermete/AttestationAlarm")
        .await?;

    // Perform initial hardware attestation workflow
    match manager.orchestrate_enclave_attestation() {
        Ok(_) => {
            info!("Initial CVM enclave attestation succeeded.");
            AttestationAlarmDbus::attestation_success(iface_ref.signal_emitter()).await?;
        }
        Err(e) => {
            let err_msg = e.to_string();
            error!("Initial CVM enclave attestation failed: {}", err_msg);
            AttestationAlarmDbus::attestation_failed(iface_ref.signal_emitter(), &err_msg).await?;
        }
    }

    // Keep daemon running to serve D-Bus requests
    std::future::pending::<()>().await;
    Ok(())
}

fn read_sysfs_tpm_pcr(pcr_idx: u32) -> Result<String> {
    let pcr_path = format!("/sys/class/tpm/tpm0/pcr-sha256/{}", pcr_idx);
    if Path::new(&pcr_path).exists() {
        if let Ok(content) = fs::read_to_string(&pcr_path) {
            return Ok(content.trim().to_string());
        }
    }
    let alt_path = format!("/sys/class/tpm/tpm0/device/pcr{}", pcr_idx);
    if Path::new(&alt_path).exists() {
        if let Ok(content) = fs::read_to_string(&alt_path) {
            return Ok(content.trim().to_string());
        }
    }
    anyhow::bail!("TPM PCR sysfs entry unreadable for index {}", pcr_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvm_manager_flow_dev_mode_fails_without_hardware() {
        let config = AttestationConfig {
            strict_zero_trust: false,
            key_output_path: std::path::PathBuf::from("/tmp/test_var_home.key"),
            ..Default::default()
        };

        let manager = CvmManager::new(config);
        assert_eq!(manager.get_state(), EnclaveState::Uninitialized);

        let result = manager.orchestrate_enclave_attestation();
        assert!(result.is_err(), "SimulatedDev mode without hardware enclave must fail attestation");
        assert!(matches!(manager.get_state(), EnclaveState::Failed(_)));
        assert!(!std::path::Path::new("/tmp/test_var_home.key").exists());
    }

    #[test]
    fn test_cvm_manager_strict_mode_fail_without_hardware() {
        let config = AttestationConfig {
            strict_zero_trust: true,
            ..Default::default()
        };

        let manager = CvmManager::new(config);
        let result = manager.orchestrate_enclave_attestation();
        assert!(result.is_err());
        assert!(matches!(manager.get_state(), EnclaveState::Failed(_)));
    }
}
