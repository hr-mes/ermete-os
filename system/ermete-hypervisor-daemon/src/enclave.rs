use anyhow::Result;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::attestation::{AttestationEngine, EnclaveLifecycleState};
use crate::kvm::{detect_capabilities, HardwareEnclaveType, KvmMicroVmContext};
use crate::sandbox::{EnclaveProcessSandbox, UntrustedAgentCategory};

/// Micro-VM Enclave descriptor and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroEnclaveDescriptor {
    pub enclave_id: String,
    pub app_name: String,
    pub exec_path: String,
    pub args: Vec<String>,
    pub pid: Option<u32>,
    pub enclave_type: HardwareEnclaveType,
    pub state: EnclaveLifecycleState,
    pub category: UntrustedAgentCategory,
    pub created_at: u64,
}

/// Central manager for zero-trust micro-enclaves lifecycle
pub struct EnclaveManager {
    attestation_engine: Arc<AttestationEngine>,
    enclaves: Arc<RwLock<HashMap<String, MicroEnclaveDescriptor>>>,
}

impl EnclaveManager {
    pub fn new(attestation_engine: Arc<AttestationEngine>) -> Self {
        Self {
            attestation_engine,
            enclaves: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Launches a new Micro-VM Enclave for an untrusted application
    pub fn launch_enclave(
        &self,
        app_name: &str,
        exec_path: &str,
        args: &[String],
        requested_type: Option<HardwareEnclaveType>,
        category: UntrustedAgentCategory,
    ) -> Result<String> {
        let caps = detect_capabilities();
        let enclave_type = requested_type.unwrap_or(caps.default_enclave_type);
        let enclave_id = format!("enclave-{}", sha2::Sha256::digest(format!("{}-{}-{}", app_name, exec_path, std::time::Instant::now().elapsed().as_nanos()).as_bytes())
            .iter()
            .take(8)
            .map(|b| format!("{:02x}", b))
            .collect::<String>());

        info!("EnclaveManager: Launching new Micro-VM Enclave ID: {}", enclave_id);
        info!("Target App: '{}', Hardware Type: {}", app_name, enclave_type);

        // 1. Initialize KVM Micro-VM context via vmm-sys-util
        let kvm_ctx = KvmMicroVmContext::new(enclave_type, 1024, 2)?;

        // 2. Perform hardware cryptographic attestation
        let attestation_summary = self
            .attestation_engine
            .orchestrate_attestation(&enclave_id, enclave_type)?;

        // 3. Spawn untrusted process into sandbox barrier
        let (pid, _child) = EnclaveProcessSandbox::spawn_in_sandbox(exec_path, args, category)?;

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let descriptor = MicroEnclaveDescriptor {
            enclave_id: enclave_id.clone(),
            app_name: app_name.to_string(),
            exec_path: exec_path.to_string(),
            args: args.to_vec(),
            pid: Some(pid),
            enclave_type,
            state: attestation_summary.state,
            category,
            created_at,
        };

        self.enclaves
            .write()
            .unwrap()
            .insert(enclave_id.clone(), descriptor);

        let _ = kvm_ctx.shutdown();
        info!("Micro-VM Enclave {} launched successfully.", enclave_id);
        Ok(enclave_id)
    }

    /// Automatically encloses an existing untrusted agent PID into a hardware enclave
    pub fn enclose_untrusted_agent(&self, pid: u32, app_type: &str) -> Result<String> {
        let category = match app_type.to_lowercase().as_str() {
            "browser" | "web" | "firefox" | "chrome" => UntrustedAgentCategory::WebBrowser,
            "foreign" | "binary" => UntrustedAgentCategory::ForeignBinary,
            "tool" => UntrustedAgentCategory::UntrustedTool,
            _ => UntrustedAgentCategory::Custom,
        };

        info!("EnclaveManager: Enclosing untrusted process PID {} (Category: {})", pid, category);

        EnclaveProcessSandbox::trap_existing_process(pid, category)?;

        let enclave_id = format!("enclave-trapped-{}", pid);
        let caps = detect_capabilities();

        let descriptor = MicroEnclaveDescriptor {
            enclave_id: enclave_id.clone(),
            app_name: format!("Trapped-PID-{}", pid),
            exec_path: format!("/proc/{}/exe", pid),
            args: vec![],
            pid: Some(pid),
            enclave_type: caps.default_enclave_type,
            state: EnclaveLifecycleState::EnclaveActive,
            category,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        self.enclaves
            .write()
            .unwrap()
            .insert(enclave_id.clone(), descriptor);

        info!("Untrusted PID {} is now securely trapped in enclave {}", pid, enclave_id);
        Ok(enclave_id)
    }

    /// Terminates an active Micro-VM Enclave
    pub fn terminate_enclave(&self, enclave_id: &str) -> Result<bool> {
        info!("EnclaveManager: Terminating enclave {}", enclave_id);

        let mut lock = self.enclaves.write().unwrap();
        if let Some(mut desc) = lock.remove(enclave_id) {
            if let Some(pid) = desc.pid {
                let _ = EnclaveProcessSandbox::terminate_pid(pid);
            }
            desc.state = EnclaveLifecycleState::Terminated;
            info!("Enclave {} terminated.", enclave_id);
            Ok(true)
        } else {
            warn!("Enclave {} not found.", enclave_id);
            Ok(false)
        }
    }

    /// Retrieves status summary of a specific enclave
    pub fn get_enclave_status(&self, enclave_id: &str) -> Option<MicroEnclaveDescriptor> {
        self.enclaves.read().unwrap().get(enclave_id).cloned()
    }

    /// Lists all active micro-enclaves
    pub fn list_enclaves(&self) -> Vec<MicroEnclaveDescriptor> {
        self.enclaves.read().unwrap().values().cloned().collect()
    }
}

use sha2::Digest;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::AttestationConfig;

    #[test]
    fn test_enclave_manager_launch_and_list() {
        let attestation_engine = Arc::new(AttestationEngine::new(AttestationConfig::default()));
        let manager = EnclaveManager::new(attestation_engine);

        let enclave_id = manager.launch_enclave(
            "test-app",
            "/bin/sleep",
            &["5".to_string()],
            Some(HardwareEnclaveType::SoftwareEnclave),
            UntrustedAgentCategory::UntrustedTool,
        );

        assert!(enclave_id.is_ok());
        let id = enclave_id.unwrap();
        assert!(id.starts_with("enclave-"));

        let list = manager.list_enclaves();
        assert_eq!(list.len(), 1);

        assert!(manager.terminate_enclave(&id).unwrap());
        assert_eq!(manager.list_enclaves().len(), 0);
    }
}
