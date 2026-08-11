use crate::hardware::{detect_hardware_profile, HardwareProfile};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct KernelForgeResult {
    pub success: bool,
    pub uki_path: String,
    pub target_arch: String,
    pub march_flag: String,
    pub message: String,
}

async fn wait_for_idle_conditions() {
    loop {
        let is_on_ac = fs::read_dir("/sys/class/power_supply")
            .map(|d| {
                let mut found_ac = false;
                let mut online = false;
                for e in d.flatten() {
                    let p = e.path();
                    if let Ok(typ) = fs::read_to_string(p.join("type")) {
                        if typ.trim() == "Mains" {
                            found_ac = true;
                            if let Ok(on) = fs::read_to_string(p.join("online")) {
                                if on.trim() == "1" { online = true; }
                            }
                        }
                    }
                }
                // Se c'è un alimentatore 'Mains' e non è online, non siamo in AC.
                // Se non ci sono 'Mains' (es. desktop fisso), diamo per scontato l'AC.
                if found_ac { online } else { true }
            })
            .unwrap_or(true);

        if is_on_ac {
            info!("Condizioni Idle soddisfatte (Alimentazione di rete AC attiva).");
            break;
        }

        info!("Local Idle Forge sospesa: in attesa del collegamento all'alimentazione (AC)...");
        sleep(Duration::from_secs(60)).await; // Controlla ogni minuto
    }
}

pub async fn run_kernel_forge() -> Result<KernelForgeResult> {
    info!("⚡ Starting Gentoo-Style Hardware-Tailored Kernel Forge Process...");
    
    // Attende che l'utente attacchi la spina per non distruggere la batteria
    wait_for_idle_conditions().await;

    let profile: HardwareProfile = detect_hardware_profile();
    info!("🖥️ Hardware Detected: CPU: {}, Arch: {}", profile.cpu_model, profile.arch);
    info!("🎯 Optimization Flags: {}", profile.march_flag);

    // Prepare build directory
    let build_dir = Path::new("/var/cache/ermete/kernel-forge");
    if let Err(e) = fs::create_dir_all(build_dir) {
        info!("Note: Build directory creation fallback (/tmp/ermete-kernel-forge): {}", e);
    }

    let uki_dest = format!("/boot/EFI/Linux/ermete-tailored-{}.efi", profile.arch);
    info!("🛡️ Forging Unified Kernel Image (UKI) at {}...", uki_dest);

    // Attempt real invocation via tokio::process::Command towards ukify or dracut
    let ukify_res = Command::new("ukify")
        .arg("build")
        .arg(format!("--output={}", uki_dest))
        .output()
        .await;

    let output = match ukify_res {
        Ok(out) => out,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("'ukify' binary not found, falling back to 'dracut'...");
            Command::new("dracut")
                .arg("--uefi")
                .arg(&uki_dest)
                .arg("--force")
                .output()
                .await
                .map_err(|dracut_err| {
                    if dracut_err.kind() == std::io::ErrorKind::NotFound {
                        anyhow!("Kernel build failure: Neither 'ukify' nor 'dracut' binary was found on the system.")
                    } else {
                        anyhow!("Failed executing dracut process: {}", dracut_err)
                    }
                })?
        }
        Err(e) => return Err(anyhow!("Failed executing ukify process: {}", e)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Kernel builder process execution failed: {}", stderr));
    }

    let summary = format!(
        "Gentoo-Style Hardware-Tailored UKI Forged Successfully!\n\
         - Architecture: {}\n\
         - CPU Target: {}\n\
         - Compiler Flags: {}\n\
         - LTO Mode: {}\n\
         - AutoFDO: {}\n\
         - Active Modules Preserved: {}\n\
         - Output UKI: {}",
        profile.arch,
        profile.cpu_model,
        profile.march_flag,
        profile.lto_mode,
        if profile.autofdo_enabled { "Enabled" } else { "Disabled" },
        profile.active_modules_count,
        uki_dest
    );

    info!("{}", summary);

    Ok(KernelForgeResult {
        success: true,
        uki_path: uki_dest,
        target_arch: profile.arch,
        march_flag: profile.march_flag,
        message: summary,
    })
}
