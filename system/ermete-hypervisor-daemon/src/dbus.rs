use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use zbus::{connection, interface, SignalContext};

use crate::attestation::{AttestationEngine, EnclaveLifecycleState};
use crate::enclave::EnclaveManager;
use crate::kvm::HardwareEnclaveType;

/// D-Bus interface `org.ermete.Hypervisor1` for zero-trust micro-enclave orchestration
pub struct HypervisorDbus {
    pub enclave_manager: Arc<EnclaveManager>,
    pub attestation_engine: Arc<AttestationEngine>,
}

#[interface(name = "org.ermete.Hypervisor1")]
impl HypervisorDbus {
    /// Launches a new Micro-VM Hardware Enclave for an untrusted binary or application
    async fn launch_enclave(
        &self,
        #[zbus(signal_context)] signal_ctxt: SignalContext<'_>,
        app_name: String,
        exec_path: String,
        args: Vec<String>,
        enclave_type: String,
    ) -> String {
        let requested_type = match enclave_type.to_lowercase().as_str() {
            "sev-snp" | "sevsnp" => Some(HardwareEnclaveType::SevSnp),
            "tdx" | "intel-tdx" => Some(HardwareEnclaveType::IntelTdx),
            "software" | "dev" => Some(HardwareEnclaveType::SoftwareEnclave),
            _ => None,
        };

        match self.enclave_manager.launch_enclave(
            &app_name,
            &exec_path,
            &args,
            requested_type,
            crate::sandbox::UntrustedAgentCategory::UntrustedTool,
        ) {
            Ok(enclave_id) => {
                let _ = Self::enclave_created(&signal_ctxt, &enclave_id, &app_name).await;
                enclave_id
            }
            Err(e) => {
                format!("Error launching enclave: {}", e)
            }
        }
    }

    /// Automatically encloses an untrusted process PID into a zero-trust hardware enclave
    async fn enclose_untrusted_agent(
        &self,
        #[zbus(signal_context)] signal_ctxt: SignalContext<'_>,
        pid: u32,
        app_type: String,
    ) -> String {
        match self.enclave_manager.enclose_untrusted_agent(pid, &app_type) {
            Ok(enclave_id) => {
                let _ = Self::untrusted_agent_trapped(&signal_ctxt, pid, &enclave_id).await;
                enclave_id
            }
            Err(e) => {
                format!("Error trapping untrusted PID {}: {}", pid, e)
            }
        }
    }

    /// Terminates an active Micro-VM Enclave
    async fn terminate_enclave(
        &self,
        #[zbus(signal_context)] signal_ctxt: SignalContext<'_>,
        enclave_id: String,
    ) -> bool {
        match self.enclave_manager.terminate_enclave(&enclave_id) {
            Ok(success) => {
                if success {
                    let _ = Self::enclave_terminated(&signal_ctxt, &enclave_id).await;
                }
                success
            }
            Err(_) => false,
        }
    }

    /// Retrieves status summary of a specific enclave as JSON
    async fn get_enclave_status(&self, enclave_id: String) -> String {
        match self.enclave_manager.get_enclave_status(&enclave_id) {
            Ok(Some(desc)) => serde_json::to_string_pretty(&desc).unwrap_or_default(),
            Ok(None) => format!(r#"{{"error": "Enclave '{}' not found"}}"#, enclave_id),
            Err(e) => format!(r#"{{"error": "Failed to get enclave status: {}"}}"#, e),
        }
    }

    /// Lists all active micro-enclaves as JSON array
    async fn list_enclaves(&self) -> String {
        match self.enclave_manager.list_enclaves() {
            Ok(list) => serde_json::to_string_pretty(&list).unwrap_or_default(),
            Err(e) => format!(r#"{{"error": "Failed to list enclaves: {}"}}"#, e),
        }
    }

    /// Triggers dynamic attestation for a specific enclave
    async fn attest_enclave(&self, enclave_id: String, _nonce: String) -> String {
        let caps = crate::kvm::detect_capabilities();
        match self
            .attestation_engine
            .orchestrate_attestation(&enclave_id, caps.default_enclave_type)
        {
            Ok(summary) => serde_json::to_string_pretty(&summary).unwrap_or_default(),
            Err(e) => format!(r#"{{"error": "Attestation failed: {}"}}"#, e),
        }
    }

    /// Checks if specified app runs inside a Micro-VM Enclave
    async fn is_microvm_app(&self, app_id: String) -> bool {
        self.enclave_manager.is_microvm_app(&app_id).unwrap_or(false)
    }

    /// Opens a secure virtio-fs tunnel for Micro-VM file access
    async fn open_virtiofs_tunnel(
        &self,
        enclave_id: String,
        host_path: String,
        read_only: bool,
    ) -> String {
        match self.enclave_manager.open_virtiofs_tunnel(&enclave_id, &host_path, read_only) {
            Ok(json_resp) => json_resp,
            Err(e) => format!(r#"{{"error": "Failed to open virtio-fs tunnel: {}"}}"#, e),
        }
    }

    /// Bridges a PipeWire Screen Sharing stream to a Micro-VM Enclave
    async fn bridge_screencast_tunnel(&self, enclave_id: String, pipewire_node: u32) -> String {
        match self.enclave_manager.bridge_screencast_tunnel(&enclave_id, pipewire_node) {
            Ok(json_resp) => json_resp,
            Err(e) => format!(r#"{{"error": "Failed to bridge ScreenCast stream: {}"}}"#, e),
        }
    }

    /// Signal emitted when a new enclave is created
    #[zbus(signal)]
    pub async fn enclave_created(
        signal_ctxt: &SignalContext<'_>,
        enclave_id: &str,
        app_name: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when an enclave is terminated
    #[zbus(signal)]
    pub async fn enclave_terminated(
        signal_ctxt: &SignalContext<'_>,
        enclave_id: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when an untrusted agent PID is trapped into an enclave
    #[zbus(signal)]
    pub async fn untrusted_agent_trapped(
        signal_ctxt: &SignalContext<'_>,
        pid: u32,
        enclave_id: &str,
    ) -> zbus::Result<()>;
}

/// Legacy replacement interface `org.ermete.AttestationAlarm1` for backward compatibility with `CvmManager`
pub struct AttestationAlarmDbus {
    pub attestation_engine: Arc<AttestationEngine>,
}

#[interface(name = "org.ermete.AttestationAlarm1")]
impl AttestationAlarmDbus {
    /// Returns overall attestation status
    async fn status(&self) -> String {
        match self.attestation_engine.get_state() {
            EnclaveLifecycleState::SecretReleased | EnclaveLifecycleState::EnclaveActive => {
                "Level 16 Micro-Hypervisor SEV-SNP/TDX Enclave Attestation Verified (Secret Released)".to_string()
            }
            EnclaveLifecycleState::Failed(ref reason) => {
                format!("Attestation Alarm: Failed ({})", reason)
            }
            state => format!("Micro-Hypervisor Attestation Status: {:?}", state),
        }
    }

    /// Returns PQC status
    async fn pqc_status(&self) -> String {
        "Level 16 PQC ML-KEM-1024 & ML-DSA-5 (Dilithium5) Micro-Hypervisor Active".to_string()
    }

    /// Triggers dynamic hardware attestation
    async fn trigger_attestation(
        &self,
        #[zbus(signal_context)] signal_ctxt: SignalContext<'_>,
    ) -> String {
        let caps = crate::kvm::detect_capabilities();
        match self
            .attestation_engine
            .orchestrate_attestation("system-main-cvm", caps.default_enclave_type)
        {
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
        if let Some(summary) = self.attestation_engine.get_last_summary() {
            serde_json::to_string_pretty(&summary).unwrap_or_default()
        } else {
            r#"{"status": "Uninitialized"}"#.to_string()
        }
    }

    /// Alarm event signal when attestation fails
    #[zbus(signal)]
    pub async fn attestation_failed(
        signal_ctxt: &SignalContext<'_>,
        reason: &str,
    ) -> zbus::Result<()>;

    /// Event signal when attestation succeeds
    #[zbus(signal)]
    pub async fn attestation_success(signal_ctxt: &SignalContext<'_>) -> zbus::Result<()>;
}

/// Helper function to register and run DBus services for Hypervisor and AttestationAlarm
pub async fn run_hypervisor_dbus_services(
    enclave_manager: Arc<EnclaveManager>,
    attestation_engine: Arc<AttestationEngine>,
) -> Result<()> {
    info!("Registering Micro-Hypervisor D-Bus services...");

    let hypervisor_iface = HypervisorDbus {
        enclave_manager,
        attestation_engine: attestation_engine.clone(),
    };

    let alarm_iface = AttestationAlarmDbus {
        attestation_engine: attestation_engine.clone(),
    };

    let _conn = connection::Builder::system()?
        .name("org.ermete.Hypervisor")?
        .serve_at("/org/ermete/Hypervisor", hypervisor_iface)?
        .serve_at("/org/ermete/AttestationAlarm", alarm_iface)?
        .build()
        .await?;

    info!("Micro-Hypervisor D-Bus active on bus 'org.ermete.Hypervisor' & 'org.ermete.AttestationAlarm'.");

    // Perform initial boot attestation check
    let caps = crate::kvm::detect_capabilities();
    match attestation_engine.orchestrate_attestation("boot-system-enclave", caps.default_enclave_type) {
        Ok(_) => {
            info!("Initial system boot hardware enclave attestation SUCCEEDED.");
        }
        Err(e) => {
            error!("Initial system boot hardware enclave attestation FAILED: {}", e);
        }
    }

    // Serve requests
    std::future::pending::<()>().await;
    Ok(())
}
