use tracing::{info, warn};

/// Capability dropping helper using raw Linux sys_capset and prctl system calls.
/// Drops all capabilities except those specified in `keep_caps`.
pub fn drop_capabilities(keep_caps: &[u32]) -> Result<(), String> {
    #[repr(C)]
    struct CapHeader {
        version: u32,
        pid: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct CapData {
        effective: u32,
        permitted: u32,
        inheritable: u32,
    }

    // _LINUX_CAPABILITY_VERSION_3 is 0x20080522
    const LINUX_CAPABILITY_VERSION_3: u32 = 0x20080522;

    let mut mask_low: u32 = 0;
    let mut mask_high: u32 = 0;

    for &cap in keep_caps {
        if cap < 32 {
            mask_low |= 1 << cap;
        } else if cap < 64 {
            mask_high |= 1 << (cap - 32);
        }
    }

    let header = CapHeader {
        version: LINUX_CAPABILITY_VERSION_3,
        pid: 0, // current process
    };

    let data = [
        CapData {
            effective: mask_low,
            permitted: mask_low,
            inheritable: mask_low,
        },
        CapData {
            effective: mask_high,
            permitted: mask_high,
            inheritable: mask_high,
        },
    ];

    unsafe {
        let ret = libc::syscall(
            libc::SYS_capset,
            &header as *const _ as *const libc::c_void,
            data.as_ptr() as *const libc::c_void,
        );

        if ret != 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("capset failed: {}", err));
        }

        let ret_pnp = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if ret_pnp != 0 {
            let err = std::io::Error::last_os_error();
            return Err(format!("prctl(PR_SET_NO_NEW_PRIVS) failed: {}", err));
        }
    }

    Ok(())
}

/// Applies capability dropping for ermete-telemetry
/// Retains: CAP_DAC_READ_SEARCH (2), CAP_NET_ADMIN (12), CAP_BPF (39)
pub fn apply_telemetry_hardening() {
    match drop_capabilities(&[2, 12, 39]) {
        Ok(_) => info!("🔒 Capability Dropping applied successfully for ermete-telemetry (Retained: CAP_DAC_READ_SEARCH, CAP_NET_ADMIN, CAP_BPF)."),
        Err(e) => warn!("⚠️ Capability Dropping warning (running unprivileged or restricted): {}", e),
    }
}
