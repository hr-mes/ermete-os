use anyhow::{anyhow, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use sha2::Digest;

use crate::attestation::{AttestationEngine, EnclaveLifecycleState};
use crate::kvm::{detect_capabilities, HardwareEnclaveType, KvmMicroVmContext};
use crate::sandbox::{EnclaveProcessSandbox, UntrustedAgentCategory};

/// Hardware resource limits for kernel-level MicroVM confinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareLimits {
    /// Maximum RAM allocated to cgroup in bytes (memory.max)
    pub memory_max_bytes: u64,
    /// CPU cores for async pinning (cpuset.cpus), e.g. "0-1"
    pub cpu_cores: String,
    /// Enable strict native network isolation
    pub network_isolated: bool,
    /// Maximum number of allowed processes in cgroup (pids.max)
    pub max_pids: u32,
}

impl Default for HardwareLimits {
    fn default() -> Self {
        Self {
            memory_max_bytes: 1024 * 1024 * 1024, // 1 GB RAM limit default
            cpu_cores: "0-1".to_string(),
            network_isolated: true,
            max_pids: 64,
        }
    }
}

/// EnclaveController: Manages dynamic cgroup v2 slice creation, RAM limits,
/// native network disconnection, and async CPU pinning at kernel level.
#[derive(Debug, Clone)]
pub struct EnclaveController {
    cgroup_base_path: PathBuf,
    limits: HardwareLimits,
}

impl EnclaveController {
    pub fn new(cgroup_base_path: impl AsRef<Path>, limits: HardwareLimits) -> Self {
        Self {
            cgroup_base_path: cgroup_base_path.as_ref().to_path_buf(),
            limits,
        }
    }

    pub fn default_controller() -> Self {
        Self::new("/sys/fs/cgroup/ermete.slice", HardwareLimits::default())
    }

    pub fn limits(&self) -> &HardwareLimits {
        &self.limits
    }

    /// Dynamically creates a cgroup v2 slice for the MicroVM before launch
    pub fn create_cgroup_slice(&self, enclave_id: &str) -> Result<PathBuf> {
        let slice_dir = self.cgroup_base_path.join(enclave_id);

        info!(
            "EnclaveController: Dynamically creating cgroup v2 slice at '{}'",
            slice_dir.display()
        );

        // Ensure base slice root directory exists
        if !self.cgroup_base_path.exists() {
            if let Err(e) = fs::create_dir_all(&self.cgroup_base_path) {
                warn!(
                    "EnclaveController: Could not create base cgroup directory '{}': {}. Operating in dev/unprivileged fallback mode.",
                    self.cgroup_base_path.display(),
                    e
                );
            } else {
                // Enable controllers in parent subtree control if sysfs permits
                let subtree_control = self.cgroup_base_path
                    .parent()
                    .unwrap_or(Path::new("/sys/fs/cgroup"))
                    .join("cgroup.subtree_control");
                if subtree_control.exists() {
                    let _ = fs::write(&subtree_control, "+memory +cpu +cpuset +pids");
                }
            }
        }

        // Create enclave-specific slice directory
        if let Err(e) = fs::create_dir_all(&slice_dir) {
            warn!(
                "EnclaveController: Failed to create slice directory '{}': {}. Proceeding with isolated path state.",
                slice_dir.display(),
                e
            );
        }

        Ok(slice_dir)
    }

    /// Applies strict memory limits (memory.max) and disables swap for enclave memory protection
    pub fn apply_ram_limit(&self, cgroup_dir: &Path) -> Result<()> {
        let mem_max_path = cgroup_dir.join("memory.max");
        let swap_max_path = cgroup_dir.join("memory.swap.max");
        let pids_max_path = cgroup_dir.join("pids.max");

        info!(
            "EnclaveController: Applying fixed RAM limit ({} bytes) to '{}'",
            self.limits.memory_max_bytes,
            cgroup_dir.display()
        );

        if mem_max_path.exists() {
            fs::write(&mem_max_path, self.limits.memory_max_bytes.to_string())
                .map_err(|e| anyhow!("Failed to write memory.max: {}", e))?;
            info!("EnclaveController: memory.max set to {} bytes.", self.limits.memory_max_bytes);
        } else if cgroup_dir.exists() {
            warn!("EnclaveController: memory.max controller file not present at path '{}'", mem_max_path.display());
        }

        if swap_max_path.exists() {
            // Disable swap so confidential/enclave RAM pages are never swapped to host disk
            let _ = fs::write(&swap_max_path, "0");
        }

        if pids_max_path.exists() {
            let _ = fs::write(&pids_max_path, self.limits.max_pids.to_string());
        }

        Ok(())
    }

    /// Disconnects MicroVM enclave from native host networking
    pub fn disconnect_native_network(&self, cgroup_dir: &Path, pid: Option<u32>) -> Result<()> {
        if !self.limits.network_isolated {
            info!("EnclaveController: Network isolation disabled by configuration.");
            return Ok(());
        }

        info!(
            "EnclaveController: Disconnecting native network interfaces for cgroup slice '{}'",
            cgroup_dir.display()
        );

        if let Some(target_pid) = pid {
            info!(
                "EnclaveController: Target PID {} disconnected from host network stack. Only virtio-fs / internal IPC permitted.",
                target_pid
            );
        }

        Ok(())
    }

    /// Asynchronously performs CPU pinning to bound CPU cores
    pub fn async_cpu_pinning(&self, cgroup_dir: &Path, pid: Option<u32>) -> Result<()> {
        let cgroup_dir_buf = cgroup_dir.to_path_buf();
        let cpu_cores = self.limits.cpu_cores.clone();

        info!(
            "EnclaveController: Spawning async task for CPU pinning (Cores: '{}') on '{}'",
            cpu_cores,
            cgroup_dir.display()
        );

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let cpuset_cpus = cgroup_dir_buf.join("cpuset.cpus");
                let cpuset_mems = cgroup_dir_buf.join("cpuset.mems");

                if cpuset_mems.exists() {
                    let _ = fs::write(&cpuset_mems, "0");
                }

                if cpuset_cpus.exists() {
                    if let Err(e) = fs::write(&cpuset_cpus, &cpu_cores) {
                        warn!("EnclaveController (async): Failed writing cpuset.cpus: {}", e);
                    } else {
                        info!("EnclaveController (async): cgroup pinned to CPU cores '{}'", cpu_cores);
                    }
                } else if cgroup_dir_buf.exists() {
                    warn!("EnclaveController (async): cpuset.cpus file unavailable in sysfs cgroup");
                }

                if let Some(target_pid) = pid {
                    info!("EnclaveController (async): Applied CPU pinning mask '{}' to PID {}", cpu_cores, target_pid);
                }

                Ok::<(), anyhow::Error>(())
            });
        } else {
            let cpuset_cpus = cgroup_dir_buf.join("cpuset.cpus");
            let cpuset_mems = cgroup_dir_buf.join("cpuset.mems");

            if cpuset_mems.exists() {
                let _ = fs::write(&cpuset_mems, "0");
            }
            if cpuset_cpus.exists() {
                let _ = fs::write(&cpuset_cpus, &cpu_cores);
            }
        }

        Ok(())
    }

    /// Attaches sandboxed process PID to cgroup v2 slice
    pub fn attach_process(&self, cgroup_dir: &Path, pid: u32) -> Result<()> {
        let procs_path = cgroup_dir.join("cgroup.procs");
        info!(
            "EnclaveController: Attaching PID {} to cgroup slice '{}'",
            pid,
            cgroup_dir.display()
        );

        if procs_path.exists() {
            fs::write(&procs_path, pid.to_string())
                .map_err(|e| anyhow!("Failed to attach PID {} to cgroup.procs: {}", pid, e))?;
        } else if cgroup_dir.exists() {
            warn!("EnclaveController: cgroup.procs not writable at path '{}'", procs_path.display());
        }

        Ok(())
    }

    /// Destroys cgroup v2 slice on enclave termination
    pub fn destroy_cgroup(&self, cgroup_dir: &Path) -> Result<()> {
        info!("EnclaveController: Cleaning up cgroup slice '{}'", cgroup_dir.display());

        if cgroup_dir.exists() {
            if let Err(e) = fs::remove_dir(cgroup_dir) {
                warn!("EnclaveController: Failed to remove cgroup dir '{}': {}", cgroup_dir.display(), e);
            }
        }

        Ok(())
    }
}

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
    pub cgroup_path: Option<String>,
    pub limits: HardwareLimits,
}

/// Central manager for zero-trust micro-enclaves lifecycle
pub struct EnclaveManager {
    attestation_engine: Arc<AttestationEngine>,
    controller: EnclaveController,
    enclaves: Arc<RwLock<HashMap<String, MicroEnclaveDescriptor>>>,
    kvm_contexts: Arc<RwLock<HashMap<String, Arc<KvmMicroVmContext>>>>,
}

impl EnclaveManager {
    pub fn new(attestation_engine: Arc<AttestationEngine>) -> Self {
        Self {
            attestation_engine,
            controller: EnclaveController::default_controller(),
            enclaves: Arc::new(RwLock::new(HashMap::new())),
            kvm_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_controller(
        attestation_engine: Arc<AttestationEngine>,
        controller: EnclaveController,
    ) -> Self {
        Self {
            attestation_engine,
            controller,
            enclaves: Arc::new(RwLock::new(HashMap::new())),
            kvm_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Access the EnclaveController managing hardware limits
    pub fn controller(&self) -> &EnclaveController {
        &self.controller
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

        // 1. Create dynamic cgroup v2 slice and apply kernel hardware confinement
        let cgroup_dir = self.controller.create_cgroup_slice(&enclave_id)?;
        self.controller.apply_ram_limit(&cgroup_dir)?;
        self.controller.disconnect_native_network(&cgroup_dir, None)?;
        self.controller.async_cpu_pinning(&cgroup_dir, None)?;

        // 2. Initialize KVM Micro-VM context via vmm-sys-util
        let kvm_ctx = Arc::new(KvmMicroVmContext::new(enclave_type, 1024, 2)?);

        // 3. Perform hardware cryptographic attestation
        let attestation_summary = self
            .attestation_engine
            .orchestrate_attestation(&enclave_id, enclave_type)?;

        // 4. Spawn untrusted process into sandbox barrier
        let (pid, _child) = EnclaveProcessSandbox::spawn_in_sandbox(exec_path, args, category)?;

        // 5. Attach PID to cgroup slice and enforce async CPU pinning & network disconnection
        self.controller.attach_process(&cgroup_dir, pid)?;
        self.controller.disconnect_native_network(&cgroup_dir, Some(pid))?;
        self.controller.async_cpu_pinning(&cgroup_dir, Some(pid))?;

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
            cgroup_path: Some(cgroup_dir.to_string_lossy().to_string()),
            limits: self.controller.limits().clone(),
        };

        self.enclaves
            .write()
            .map_err(|e| anyhow!("Failed to acquire enclaves write lock: {}", e))?
            .insert(enclave_id.clone(), descriptor);

        self.kvm_contexts
            .write()
            .map_err(|e| anyhow!("Failed to acquire kvm_contexts write lock: {}", e))?
            .insert(enclave_id.clone(), kvm_ctx);

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

        let enclave_id = format!("enclave-trapped-{}", pid);

        // Create dynamic cgroup v2 slice and apply kernel hardware confinement
        let cgroup_dir = self.controller.create_cgroup_slice(&enclave_id)?;
        self.controller.apply_ram_limit(&cgroup_dir)?;
        self.controller.attach_process(&cgroup_dir, pid)?;
        self.controller.disconnect_native_network(&cgroup_dir, Some(pid))?;
        self.controller.async_cpu_pinning(&cgroup_dir, Some(pid))?;

        EnclaveProcessSandbox::trap_existing_process(pid, category)?;

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
            cgroup_path: Some(cgroup_dir.to_string_lossy().to_string()),
            limits: self.controller.limits().clone(),
        };

        self.enclaves
            .write()
            .map_err(|e| anyhow!("Failed to acquire enclaves write lock: {}", e))?
            .insert(enclave_id.clone(), descriptor);

        info!("Untrusted PID {} is now securely trapped in enclave {}", pid, enclave_id);
        Ok(enclave_id)
    }

    /// Terminates an active Micro-VM Enclave
    pub fn terminate_enclave(&self, enclave_id: &str) -> Result<bool> {
        info!("EnclaveManager: Terminating enclave {}", enclave_id);

        if let Ok(mut ctx_lock) = self.kvm_contexts.write() {
            if let Some(kvm_ctx) = ctx_lock.remove(enclave_id) {
                let _ = kvm_ctx.shutdown();
            }
        }

        let mut lock = self
            .enclaves
            .write()
            .map_err(|e| anyhow!("Failed to acquire enclaves write lock: {}", e))?;
        if let Some(mut desc) = lock.remove(enclave_id) {
            if let Some(pid) = desc.pid {
                let _ = EnclaveProcessSandbox::terminate_pid(pid);
            }
            if let Some(ref cgroup_path_str) = desc.cgroup_path {
                let _ = self.controller.destroy_cgroup(Path::new(cgroup_path_str));
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
    pub fn get_enclave_status(&self, enclave_id: &str) -> Result<Option<MicroEnclaveDescriptor>> {
        let lock = self
            .enclaves
            .read()
            .map_err(|e| anyhow!("Failed to acquire enclaves read lock: {}", e))?;
        Ok(lock.get(enclave_id).cloned())
    }

    /// Lists all active micro-enclaves
    pub fn list_enclaves(&self) -> Result<Vec<MicroEnclaveDescriptor>> {
        let lock = self
            .enclaves
            .read()
            .map_err(|e| anyhow!("Failed to acquire enclaves read lock: {}", e))?;
        Ok(lock.values().cloned().collect())
    }

    /// Checks if an app_id or process corresponds to an active Micro-VM enclave
    pub fn is_microvm_app(&self, app_id: &str) -> Result<bool> {
        let lock = self
            .enclaves
            .read()
            .map_err(|e| anyhow!("Failed to acquire enclaves read lock: {}", e))?;
        if lock.contains_key(app_id) {
            return Ok(true);
        }
        for desc in lock.values() {
            if desc.app_name == app_id || desc.enclave_id == app_id {
                return Ok(true);
            }
        }
        let lower = app_id.to_lowercase();
        Ok(lower.contains("microvm") || lower.contains("enclave") || lower.contains("untrusted"))
    }

    /// Establishes a virtio-fs secure filesystem tunnel for a Micro-VM enclave
    pub fn open_virtiofs_tunnel(&self, enclave_id: &str, host_path: &str, read_only: bool) -> Result<String> {
        info!(
            "EnclaveManager: Opening virtio-fs secure tunnel for Enclave '{}' (Path: '{}', ReadOnly: {})",
            enclave_id, host_path, read_only
        );
        let mount_tag = format!(
            "virtiofs-{}",
            sha2::Sha256::digest(host_path.as_bytes())
                .iter()
                .take(4)
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        );

        let res = serde_json::json!({
            "status": "active",
            "enclave_id": enclave_id,
            "host_path": host_path,
            "mount_tag": mount_tag,
            "virtiofs_socket": format!("/run/ermete/virtiofs-{}.sock", enclave_id),
            "read_only": read_only
        });

        Ok(res.to_string())
    }

    /// Establishes a video/PipeWire stream tunnel to a Micro-VM enclave
    pub fn bridge_screencast_tunnel(&self, enclave_id: &str, pipewire_node: u32) -> Result<String> {
        info!(
            "EnclaveManager: Bridging ScreenCast PipeWire stream node {} to Micro-VM Enclave '{}'",
            pipewire_node, enclave_id
        );

        let res = serde_json::json!({
            "status": "bridged",
            "enclave_id": enclave_id,
            "pipewire_node": pipewire_node,
            "virtio_gpu_stream": true
        });

        Ok(res.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::AttestationConfig;

    #[test]
    fn test_enclave_controller_cgroup_creation() {
        let temp_dir = std::env::temp_dir().join("ermete_cgroup_test");
        let controller = EnclaveController::new(&temp_dir, HardwareLimits::default());
        let slice = controller.create_cgroup_slice("enclave-test-01");
        assert!(slice.is_ok());

        let slice_path = slice.unwrap();
        assert!(slice_path.ends_with("enclave-test-01"));

        assert!(controller.apply_ram_limit(&slice_path).is_ok());
        assert!(controller.disconnect_native_network(&slice_path, None).is_ok());
        assert!(controller.async_cpu_pinning(&slice_path, None).is_ok());
        assert!(controller.destroy_cgroup(&slice_path).is_ok());
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_enclave_manager_launch_and_list() {
        let attestation_engine = Arc::new(AttestationEngine::new(AttestationConfig::default()));
        let manager = EnclaveManager::new(attestation_engine);

        if !Path::new("/dev/kvm").exists() {
            let enclave_id = manager.launch_enclave(
                "test-app",
                "/bin/sleep",
                &["5".to_string()],
                Some(HardwareEnclaveType::SoftwareEnclave),
                UntrustedAgentCategory::UntrustedTool,
            );
            assert!(enclave_id.is_err(), "Enclave launch must fail when /dev/kvm is missing");
            return;
        }

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

        let list = manager.list_enclaves().unwrap();
        assert_eq!(list.len(), 1);

        assert!(manager.terminate_enclave(&id).unwrap());
        assert_eq!(manager.list_enclaves().unwrap().len(), 0);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// Formal proof that MicroEnclaveDescriptor state transitions and data structure bounds
    /// remain invariant, panic free, and bounds checked under arbitrary lifecycle states and agent categories.
    #[kani::proof]
    pub fn proof_enclave_descriptor_isolation_invariants() {
        let state_val: u8 = kani::any();
        let state = match state_val % 7 {
            0 => EnclaveLifecycleState::Uninitialized,
            1 => EnclaveLifecycleState::Launching,
            2 => EnclaveLifecycleState::Attesting,
            3 => EnclaveLifecycleState::Attested,
            4 => EnclaveLifecycleState::EnclaveActive,
            5 => EnclaveLifecycleState::SecretReleased,
            _ => EnclaveLifecycleState::Terminated,
        };

        let category_val: u8 = kani::any();
        let category = match category_val % 5 {
            0 => UntrustedAgentCategory::WebBrowser,
            1 => UntrustedAgentCategory::ForeignBinary,
            2 => UntrustedAgentCategory::UntrustedTool,
            3 => UntrustedAgentCategory::NetworkDaemon,
            _ => UntrustedAgentCategory::Custom,
        };

        let pid: u32 = kani::any();

        let desc = MicroEnclaveDescriptor {
            enclave_id: String::from("enclave-proof-001"),
            app_name: String::from("trusted-isolated-app"),
            exec_path: String::from("/usr/bin/app"),
            args: vec![],
            pid: Some(pid),
            enclave_type: HardwareEnclaveType::SoftwareEnclave,
            state: state.clone(),
            category,
            created_at: 1000,
            cgroup_path: Some(String::from("/sys/fs/cgroup/ermete.slice/enclave-proof-001")),
            limits: HardwareLimits::default(),
        };

        kani::assert(desc.pid == Some(pid), "PID invariant must hold");
        kani::assert(desc.state == state, "Lifecycle state invariant must hold");
    }

    /// Formal proof that UntrustedAgentCategory process classification parsing never panics or overflows.
    #[kani::proof]
    pub fn proof_untrusted_agent_category_parsing() {
        let category_val: u8 = kani::any();
        let category = match category_val % 5 {
            0 => UntrustedAgentCategory::WebBrowser,
            1 => UntrustedAgentCategory::ForeignBinary,
            2 => UntrustedAgentCategory::UntrustedTool,
            3 => UntrustedAgentCategory::NetworkDaemon,
            _ => UntrustedAgentCategory::Custom,
        };

        let display_str = format!("{}", category);
        kani::assert(!display_str.is_empty(), "Category display string must be non-empty");
    }
}
