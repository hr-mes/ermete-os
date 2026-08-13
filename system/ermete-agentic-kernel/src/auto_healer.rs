use std::os::unix::fs::OpenOptionsExt;
use anyhow::{bail, Result};
use tracing::{info, warn};

pub struct AutoHealer;

/// Whitelist of kernel sysctl parameters that the AI engine is permitted to adjust.
/// All other sysctl paths (arbitrary memory params, kernel.panic, etc.) are strictly forbidden.
const ALLOWED_SYSCTLS: &[&str] = &[
    "net.ipv4.tcp_max_syn_backlog",
    "net.core.somaxconn",
    "vm.swappiness",
    "vm.dirty_ratio",
    "vm.dirty_background_ratio",
    "net.ipv4.ip_local_port_range",
    "net.ipv4.tcp_rmem",
    "net.ipv4.tcp_wmem",
];

impl Default for AutoHealer {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoHealer {
    pub fn new() -> Self {
        Self
    }

    /// Injects sysctl parameters dynamically into /proc/sys to heal kernel sub-optimal state or mitigate attacks.
    /// Enforces strict whitelist safety boundaries to prevent AI Ring-0 overreach or arbitrary kernel memory corruption.
    pub fn inject_sysctl(&self, param: &str, value: &str) -> Result<()> {
        // 1. Path traversal & shell injection guard
        if param.contains("..") || param.contains('/') {
            let msg = format!("⛔ [AI Confinement Violation] Rejected sysctl injection with invalid characters/path traversal: {}", param);
            warn!("{}", msg);
            bail!(msg);
        }

        // 2. Safety Whitelist Validation
        if !ALLOWED_SYSCTLS.contains(&param) {
            let msg = format!(
                "⛔ [AI Confinement Violation] Rejected non-whitelisted sysctl parameter '{}'. AI agent write access restricted to performance & networking metrics only.",
                param
            );
            warn!("{}", msg);
            bail!(msg);
        }

        // 3. Value sanitization: ensure numeric/space payload
        if !value.chars().all(|c| c.is_ascii_digit() || c == ' ') {
            let msg = format!("⛔ [AI Confinement Violation] Rejected non-numeric sysctl value '{}' for parameter '{}'", value, param);
            warn!("{}", msg);
            bail!(msg);
        }

        info!("Injecting Sysctl Parameter (Auto-Healing): {} = {}", param, value);
        
        let path = format!("/proc/sys/{}", param.replace('.', "/"));
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, value.as_bytes()))?;
        info!("Successfully updated kernel parameter {} to {} via sysfs/procfs", param, value);
        Ok(())
    }

    /// Reallocates system resources dynamically based on NPU AI decisions
    pub fn apply_autonomic_reallocation(&self, mitigations: &[(String, String)]) {
        info!("⚡ Executing Autonomic Kernel Resource Re-allocation (Zero-Touch Auto-Healing)...");
        for (param, val) in mitigations {
            if let Err(e) = self.inject_sysctl(param, val) {
                warn!("Auto-healing sysctl injection rejected or failed: {:#}", e);
            }
        }
        info!("Autonomic Kernel Healing cycle complete. System state optimized within safety boundaries.");
    }
}

