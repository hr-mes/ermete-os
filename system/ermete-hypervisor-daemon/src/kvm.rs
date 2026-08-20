#![allow(dead_code)]

use anyhow::{anyhow, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use vmm_sys_util::eventfd::EventFd;

/// Hardware enclave technology supported by the Micro-Hypervisor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareEnclaveType {
    /// AMD Secure Encrypted Virtualization with Secure Nested Paging
    SevSnp,
    /// Intel Trust Domain Extensions
    IntelTdx,
    /// Fallback isolated software micro-VM (for development or non-CVM hardware)
    SoftwareEnclave,
}

impl std::fmt::Display for HardwareEnclaveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareEnclaveType::SevSnp => write!(f, "AMD SEV-SNP Hardware Enclave"),
            HardwareEnclaveType::IntelTdx => write!(f, "Intel TDX Trust Domain Enclave"),
            HardwareEnclaveType::SoftwareEnclave => write!(f, "Zero-Trust Software Micro-VM Enclave"),
        }
    }
}

/// KVM Hypervisor capabilities detected on host system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypervisorCapabilities {
    pub kvm_available: bool,
    pub sev_snp_supported: bool,
    pub tdx_supported: bool,
    pub max_vcpus_per_vm: u32,
    pub default_enclave_type: HardwareEnclaveType,
}

/// Represents an active KVM Micro-VM context managed by vmm-sys-util
pub struct KvmMicroVmContext {
    pub vm_fd: RawFd,
    pub _kvm_file: Option<File>,
    pub enclave_type: HardwareEnclaveType,
    pub event_fd: EventFd,
    pub memory_size_mb: u64,
    pub vcpu_count: u32,
}

const KVM_GET_API_VERSION: libc::c_ulong = 0xAE00;
const KVM_CREATE_VM: libc::c_ulong = 0xAE01;

impl KvmMicroVmContext {
    #[allow(unsafe_code)]
    pub fn new(enclave_type: HardwareEnclaveType, memory_size_mb: u64, vcpu_count: u32) -> Result<Self> {
        let event_fd = EventFd::new(0).map_err(|e| anyhow!("EventFd creation failed: {}", e))?;
        
        if !Path::new("/dev/kvm").exists() {
            anyhow::bail!("KVM hardware device (/dev/kvm) does not exist on host system");
        }

        let file = File::open("/dev/kvm")
            .map_err(|e| anyhow!("Failed to open /dev/kvm: {}", e))?;
        let kvm_fd = file.as_raw_fd();
        
        let api_version = unsafe { libc::ioctl(kvm_fd, KVM_GET_API_VERSION, 0) };
        if api_version < 0 {
            anyhow::bail!(
                "KVM_GET_API_VERSION ioctl failed: {}",
                std::io::Error::last_os_error()
            );
        }
        
        info!("KVM API version: {}", api_version);
        let created_vm_fd = unsafe { libc::ioctl(kvm_fd, KVM_CREATE_VM, 0) };
        if created_vm_fd < 0 {
            anyhow::bail!(
                "KVM_CREATE_VM ioctl failed: {}",
                std::io::Error::last_os_error()
            );
        }
        
        info!("KVM_CREATE_VM ioctl succeeded: created VM file descriptor {}", created_vm_fd);

        Ok(Self {
            vm_fd: created_vm_fd,
            _kvm_file: Some(file),
            enclave_type,
            event_fd,
            memory_size_mb,
            vcpu_count,
        })
    }

    #[allow(unsafe_code)]
    pub fn shutdown(&self) -> Result<()> {
        if self.vm_fd >= 0 {
            unsafe {
                libc::close(self.vm_fd);
            }
        }
        info!("KVM Micro-VM context (Type: {}) shut down cleanly.", self.enclave_type);
        Ok(())
    }
}

/// Helper utility for detecting KVM and confidential computing capabilities
pub fn detect_capabilities() -> HypervisorCapabilities {
    let kvm_available = Path::new("/dev/kvm").exists();

    // Check AMD SEV-SNP support via sysfs / dev node
    let sev_snp_supported = Path::new("/dev/sev-guest").exists()
        || Path::new("/sys/module/kvm_amd/parameters/sev_snp").exists()
        || (Path::new("/sys/module/kvm_amd/parameters/sev").exists()
            && fs::read_to_string("/sys/module/kvm_amd/parameters/sev")
                .map(|s| s.trim() == "1" || s.trim() == "Y")
                .unwrap_or(false));

    // Check Intel TDX support via sysfs / dev node
    let tdx_supported = Path::new("/dev/tdx_guest").exists()
        || Path::new("/dev/tdx-attest").exists()
        || (Path::new("/sys/module/kvm_intel/parameters/tdx").exists()
            && fs::read_to_string("/sys/module/kvm_intel/parameters/tdx")
                .map(|s| s.trim() == "1" || s.trim() == "Y")
                .unwrap_or(false));

    let default_enclave_type = if sev_snp_supported {
        HardwareEnclaveType::SevSnp
    } else if tdx_supported {
        HardwareEnclaveType::IntelTdx
    } else {
        HardwareEnclaveType::SoftwareEnclave
    };

    HypervisorCapabilities {
        kvm_available,
        sev_snp_supported,
        tdx_supported,
        max_vcpus_per_vm: 16,
        default_enclave_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_capabilities() {
        let caps = detect_capabilities();
        println!("Detected Hypervisor Capabilities: {:?}", caps);
        assert!(caps.max_vcpus_per_vm > 0);
    }

    #[test]
    fn test_kvm_micro_vm_context_creation() {
        if Path::new("/dev/kvm").exists() {
            let ctx = KvmMicroVmContext::new(HardwareEnclaveType::SoftwareEnclave, 512, 2);
            if let Ok(vm) = ctx {
                assert_eq!(vm.memory_size_mb, 512);
                assert_eq!(vm.vcpu_count, 2);
                assert!(vm.shutdown().is_ok());
            }
        } else {
            let ctx = KvmMicroVmContext::new(HardwareEnclaveType::SoftwareEnclave, 512, 2);
            assert!(ctx.is_err(), "KvmMicroVmContext creation must fail when /dev/kvm device is missing");
        }
    }
}
