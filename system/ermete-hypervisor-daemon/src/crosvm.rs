use anyhow::{anyhow, Context, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Shared Directory configuration for MicroVM filesystem mounting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedDirConfig {
    /// Host directory path to share
    pub host_path: PathBuf,
    /// Tag identifier inside the guest VM
    pub tag: String,
    /// Read-only enforcement flag (must be true for untrusted apps)
    pub read_only: bool,
}

impl SharedDirConfig {
    pub fn new_readonly(host_path: impl Into<PathBuf>, tag: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            tag: tag.into(),
            read_only: true,
        }
    }

    /// Formats parameter into crosvm `--shared-dir` string
    /// Syntax: `<path>:<tag>:type=fs:ro=<true|false>`
    pub fn to_crosvm_arg(&self) -> String {
        format!(
            "{}:{}:type=fs:ro={}",
            self.host_path.display(),
            self.tag,
            if self.read_only { "true" } else { "false" }
        )
    }
}

/// Configuration parameters for launching a Crosvm MicroVM instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosvmConfig {
    /// Path to the crosvm binary
    pub binary_path: PathBuf,
    /// Kernel image path (e.g. UKI / vmlinuz)
    pub kernel_path: Option<PathBuf>,
    /// Initrd image path
    pub initrd_path: Option<PathBuf>,
    /// Path to the Wayland socket for GUI pass-through
    pub wayland_sock: Option<PathBuf>,
    /// Enable GPU acceleration pass-through
    pub enable_gpu: bool,
    /// Optional GPU parameters (e.g. virgl, renderNode)
    pub gpu_params: Option<String>,
    /// Shared directory configuration (Read-Only mounting for untrusted apps)
    pub shared_dir: Option<SharedDirConfig>,
    /// Memory allocation in megabytes
    pub memory_mb: u64,
    /// VCPU count
    pub vcpu_count: u32,
    /// Additional command-line arguments for crosvm
    pub extra_args: Vec<String>,
}

impl Default for CrosvmConfig {
    fn default() -> Self {
        Self {
            binary_path: PathBuf::from("/usr/bin/crosvm"),
            kernel_path: None,
            initrd_path: None,
            wayland_sock: Some(PathBuf::from("/run/user/1000/wayland-0")),
            enable_gpu: true,
            gpu_params: None,
            shared_dir: None,
            memory_mb: 2048,
            vcpu_count: 2,
            extra_args: Vec::new(),
        }
    }
}

/// Represents an active or pending Crosvm MicroVM Instance
pub struct CrosvmInstance {
    pub id: String,
    pub config: CrosvmConfig,
    pub child: Option<Child>,
    pub pid: Option<u32>,
    pub is_running: bool,
}

impl CrosvmInstance {
    /// Constructs a new CrosvmInstance with given ID and configuration
    pub fn new(id: impl Into<String>, config: CrosvmConfig) -> Self {
        Self {
            id: id.into(),
            config,
            child: None,
            pid: None,
            is_running: false,
        }
    }

    /// Builds the `tokio::process::Command` configured with Wayland, GPU, shared dir, and stdio pipes
    pub fn build_command(&self) -> Result<Command> {
        if !self.config.binary_path.exists() {
            warn!(
                "Crosvm binary path '{}' does not exist on host system. Command builder proceeding for test/sandbox validation.",
                self.config.binary_path.display()
            );
        }

        let mut cmd = Command::new(&self.config.binary_path);
        cmd.arg("run");

        // Set memory and CPU
        cmd.arg("--mem").arg(self.config.memory_mb.to_string());
        cmd.arg("--cpus").arg(self.config.vcpu_count.to_string());

        // Kernel & Initrd pass-through
        if let Some(ref kpath) = self.config.kernel_path {
            cmd.arg("--kernel").arg(kpath);
        }
        if let Some(ref ipath) = self.config.initrd_path {
            cmd.arg("--initrd").arg(ipath);
        }

        // 1. Wayland pass-through (--wayland-sock)
        if let Some(ref wsock) = self.config.wayland_sock {
            info!("Crosvm [{}]: Enabling Wayland pass-through with socket '{}'", self.id, wsock.display());
            cmd.arg("--wayland-sock").arg(wsock);
        }

        // 2. GPU Acceleration (--gpu)
        if self.config.enable_gpu {
            info!("Crosvm [{}]: Enabling Hardware GPU acceleration", self.id);
            if let Some(ref gpu_args) = self.config.gpu_params {
                cmd.arg(format!("--gpu={}", gpu_args));
            } else {
                cmd.arg("--gpu");
            }
        }

        // 3. Shared Filesystem Read-Only Mount (--shared-dir) for untrusted app
        if let Some(ref sdir) = self.config.shared_dir {
            if !sdir.read_only {
                return Err(anyhow!(
                    "Zero-Trust Security Violation: Shared directory for untrusted app must be read-only!"
                ));
            }
            let sdir_arg = sdir.to_crosvm_arg();
            info!("Crosvm [{}]: Mounting shared read-only dir arg '{}'", self.id, sdir_arg);
            cmd.arg("--shared-dir").arg(sdir_arg);
        }

        // Append any custom extra arguments
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }

        // 4. Stdio management to prevent panics and unhandled pipe hangs
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        Ok(cmd)
    }

    /// Spawns the Crosvm MicroVM process asynchronously
    pub async fn spawn(&mut self) -> Result<u32> {
        if self.is_running {
            return Err(anyhow!("CrosvmInstance [{}] is already running.", self.id));
        }

        let mut command = self.build_command()?;
        info!("Spawning Crosvm MicroVM instance [{}]...", self.id);

        let child = command
            .spawn()
            .with_context(|| format!("Failed to spawn crosvm process for instance [{}]", self.id))?;

        let pid = child
            .id()
            .ok_or_else(|| anyhow!("Failed to retrieve PID for spawned crosvm instance [{}]", self.id))?;

        info!("Crosvm MicroVM instance [{}] successfully spawned with PID {}", self.id, pid);

        self.pid = Some(pid);
        self.child = Some(child);
        self.is_running = true;

        Ok(pid)
    }

    /// Safely terminates the Crosvm MicroVM instance
    pub async fn terminate(&mut self) -> Result<()> {
        if !self.is_running {
            info!("CrosvmInstance [{}] is not running, termination ignored.", self.id);
            return Ok(());
        }

        info!("Terminating Crosvm MicroVM instance [{}] (PID {:?})...", self.id, self.pid);

        if let Some(mut child) = self.child.take() {
            // Attempt standard SIGTERM or graceful child kill
            if let Err(e) = child.kill().await {
                warn!("Failed to kill child process for instance [{}]: {}", self.id, e);
            }
            let _ = child.wait().await;
        }

        self.pid = None;
        self.is_running = false;
        info!("CrosvmInstance [{}] terminated cleanly.", self.id);
        Ok(())
    }

    /// Checks whether the child process is still running asynchronously
    pub async fn check_alive(&mut self) -> Result<bool> {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    info!("CrosvmInstance [{}] exited with status: {}", self.id, status);
                    self.is_running = false;
                    self.pid = None;
                    Ok(false)
                }
                Ok(None) => Ok(true),
                Err(e) => {
                    error!("Error checking status of CrosvmInstance [{}]: {}", self.id, e);
                    Err(anyhow!(e))
                }
            }
        } else {
            self.is_running = false;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_dir_formatting() {
        let sdir = SharedDirConfig::new_readonly("/var/app/untrusted", "appfs");
        assert_eq!(sdir.to_crosvm_arg(), "/var/app/untrusted:appfs:type=fs:ro=true");
    }

    #[test]
    fn test_command_builder() {
        let mut config = CrosvmConfig::default();
        config.wayland_sock = Some(PathBuf::from("/tmp/wayland-test.sock"));
        config.enable_gpu = true;
        config.shared_dir = Some(SharedDirConfig::new_readonly("/tmp/untrusted_app", "app_mount"));

        let instance = CrosvmInstance::new("test-vm-1", config);
        let cmd_res = instance.build_command();
        assert!(cmd_res.is_ok());
    }
}
