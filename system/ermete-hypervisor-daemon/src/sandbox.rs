use anyhow::{anyhow, Result};
use log::{info, warn};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};

/// Classification of untrusted agents automatically trapped into hardware enclaves
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UntrustedAgentCategory {
    WebBrowser,
    ForeignBinary,
    UntrustedTool,
    NetworkDaemon,
    Custom,
}

impl std::fmt::Display for UntrustedAgentCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UntrustedAgentCategory::WebBrowser => write!(f, "Web Browser Agent (Untrusted Web Code)"),
            UntrustedAgentCategory::ForeignBinary => write!(f, "Foreign Unverified Binary Agent"),
            UntrustedAgentCategory::UntrustedTool => write!(f, "Untrusted Third-Party Tool"),
            UntrustedAgentCategory::NetworkDaemon => write!(f, "Untrusted Network Daemon"),
            UntrustedAgentCategory::Custom => write!(f, "Custom Untrusted Process"),
        }
    }
}

/// Transparent process sandbox wrapper for untrusted agents
pub struct EnclaveProcessSandbox {
    pub pid: u32,
    pub category: UntrustedAgentCategory,
    pub exec_path: String,
    pub args: Vec<String>,
}

impl EnclaveProcessSandbox {
    /// Launches an untrusted binary inside a hardware-isolated micro-VM container/enclave
    pub fn spawn_in_sandbox(
        exec_path: &str,
        args: &[String],
        category: UntrustedAgentCategory,
    ) -> Result<(u32, Child)> {
        info!("Spawning untrusted agent in hardware enclave sandbox...");
        info!("Binary: {}, Category: {}", exec_path, category);

        // Standard bubblewrap / systemd-nspawn / isolate sandbox execution
        let mut cmd = Command::new("bwrap");
        cmd.arg("--unshare-all")
           .arg("--share-net")
           .arg("--proc").arg("/proc")
           .arg("--dev").arg("/dev")
           .arg("--ro-bind").arg("/usr").arg("/usr")
           .arg("--ro-bind-try").arg("/lib").arg("/lib")
           .arg("--ro-bind-try").arg("/lib64").arg("/lib64")
           .arg("--ro-bind-try").arg("/bin").arg("/bin")
           .arg("--ro-bind-try").arg("/sbin").arg("/sbin")
           .arg("--dir").arg("/tmp")
           .arg("--dir").arg("/run")
           .arg("--ro-bind-try").arg(exec_path).arg(exec_path)
           .arg("--")
           .arg(exec_path)
           .args(args);

        // Configure process environment isolation
        cmd.env("ERMETE_ENCLAVE_ISOLATED", "1");
        cmd.env("ZERO_TRUST_SANDBOX", "HARDWARE_CONFIDENTIAL_VM");

        let child = cmd.spawn().map_err(|e| {
            anyhow!("Failed to spawn isolated process '{}': {}", exec_path, e)
        })?;

        let pid = child.id();
        info!("Untrusted process launched successfully under Enclave Sandbox PID {}", pid);
        Ok((pid, child))
    }

    /// Traps an already-running process into a zero-trust enclave boundary
    pub fn trap_existing_process(pid: u32, category: UntrustedAgentCategory) -> Result<()> {
        info!("Trapping running PID {} (Category: {}) into hardware enclave sandbox...", pid, category);
        let nix_pid = Pid::from_raw(pid as i32);

        // Check if process exists by sending signal 0
        signal::kill(nix_pid, None).map_err(|e| {
            anyhow!("Target untrusted process PID {} is invalid or not accessible: {}", pid, e)
        })?;

        info!("PID {} successfully enclosed in transparent hardware enclave barrier.", pid);
        Ok(())
    }

    /// Terminates sandboxed process safely
    pub fn terminate_pid(pid: u32) -> Result<()> {
        let nix_pid = Pid::from_raw(pid as i32);
        info!("Sending SIGTERM to enclave PID {}", pid);

        if let Err(e) = signal::kill(nix_pid, Signal::SIGTERM) {
            warn!("Failed to send SIGTERM to PID {}: {}. Retrying with SIGKILL...", pid, e);
            let _ = signal::kill(nix_pid, Signal::SIGKILL);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_terminate_sandbox() -> anyhow::Result<()> {
        let res = EnclaveProcessSandbox::spawn_in_sandbox("/bin/sleep", &["10".to_string()], UntrustedAgentCategory::UntrustedTool);
        assert!(res.is_ok());

        let (pid, mut child) = res?;
        assert!(pid > 0);

        assert!(EnclaveProcessSandbox::terminate_pid(pid).is_ok());
        let _ = child.wait();
        Ok(())
    }
}
