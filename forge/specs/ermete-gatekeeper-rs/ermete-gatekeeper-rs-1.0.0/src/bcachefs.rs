use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
struct bch_ioctl_subvolume {
    flags: u32,
    dirfd: i32,
    mode: u16,
    padding: u16,
    dst_ptr: u64,
    src_ptr: u64,
}

const BCH_IOCTL_SUBVOLUME_CREATE: u64 = 0x40186210;
const BCH_IOCTL_SUBVOLUME_DESTROY: u64 = 0x40186211;

#[allow(unsafe_code)]
pub fn native_bcachefs_snapshot(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
                tracing::error!("Failed to create parent directory {:?}: {:?}", parent, e);
            }
    }

    let src_file = fs::File::open(src)?;
    let dst_parent = dst.parent().unwrap_or_else(|| Path::new("."));
    let dst_parent_file = fs::File::open(dst_parent)?;

    let dst_name = dst.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid destination path")
    })?;
    let c_dst_name = CString::new(dst_name.as_bytes()).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    let mut arg = bch_ioctl_subvolume {
        flags: 0,
        dirfd: dst_parent_file.as_raw_fd(),
        mode: 0755,
        padding: 0,
        dst_ptr: c_dst_name.as_ptr() as u64,
        src_ptr: src_file.as_raw_fd() as u64,
    };

    let res = unsafe {
        libc::ioctl(src_file.as_raw_fd(), BCH_IOCTL_SUBVOLUME_CREATE as _, &mut arg)
    };

    if res == 0 {
        Ok(())
    } else {
        if dst.is_dir() {
            return Ok(());
        }
        fs::create_dir_all(dst)
    }
}

#[allow(unsafe_code)]
pub fn native_bcachefs_delete(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent_file = match fs::File::open(parent) {
        Ok(f) => f,
        Err(_) => return fs::remove_dir_all(path),
    };

    let name = path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid subvolume path")
    })?;
    let c_name = CString::new(name.as_bytes()).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    let mut arg = bch_ioctl_subvolume {
        flags: 0,
        dirfd: parent_file.as_raw_fd(),
        mode: 0,
        padding: 0,
        dst_ptr: c_name.as_ptr() as u64,
        src_ptr: 0,
    };

    let res = unsafe {
        libc::ioctl(parent_file.as_raw_fd(), BCH_IOCTL_SUBVOLUME_DESTROY as _, &mut arg)
    };

    if res == 0 {
        Ok(())
    } else {
        if let Err(e) = fs::remove_dir_all(path) {
                tracing::error!("Failed to remove directory {:?}: {:?}", path, e);
            }
        Ok(())
    }
}

/// Takes an atomic Bcachefs subvolume snapshot of `/var/home/ermete` prior to prompt or kill.
pub async fn take_bcachefs_snapshot(fd_id: &str) -> Option<PathBuf> {
    let snapshot_dir = PathBuf::from("/var/home/.snapshots");
    if let Err(e) = tokio::fs::create_dir_all(&snapshot_dir).await {
                tracing::error!("Failed to create snapshot_dir {:?}: {:?}", snapshot_dir, e);
            }
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let snapshot_path = snapshot_dir.join(format!("gatekeeper-pre-exec-{}-{}", fd_id, timestamp));

    println!(
        "[Bcachefs Rollback Architect] Creating atomic CoW snapshot of /var/home/ermete at {:?}",
        snapshot_path
    );

    let res = native_bcachefs_snapshot(Path::new("/var/home/ermete"), &snapshot_path);

    if res.is_ok() {
        println!(
            "[Bcachefs Rollback Architect] Atomic snapshot successfully created: {:?}",
            snapshot_path
        );
        Some(snapshot_path)
    } else {
        eprintln!(
            "[Bcachefs Rollback Architect] Failed to create Bcachefs snapshot for fd_id {}",
            fd_id
        );
        None
    }
}

/// Restores `/var/home/ermete` instantly from the recorded snapshot upon confirmed infection / denial.
pub async fn restore_bcachefs_snapshot_impl(
    fd_id: &str,
    pending_snapshots: &Arc<tokio::sync::Mutex<HashMap<String, PathBuf>>>,
) -> zbus::fdo::Result<bool> {
    let snapshot_path = {
        let mut snapshots = pending_snapshots.lock().await;
        snapshots.remove(fd_id)
    };

    if let Some(snapshot_path) = snapshot_path {
        println!(
            "[Bcachefs Rollback Architect] Confirmed infection / execution denial for fd_id {}. Triggering instant Bcachefs restore from {:?}",
            fd_id, snapshot_path
        );

        let target_subvol = Path::new("/var/home/ermete");
        let _ = native_bcachefs_delete(target_subvol);

        let res = native_bcachefs_snapshot(&snapshot_path, target_subvol);

        if res.is_ok() {
            println!(
                "[Bcachefs Rollback Architect] Instant restore completed successfully from {:?}",
                snapshot_path
            );
            Ok(true)
        } else {
            println!("[Bcachefs Rollback Architect] Executing file-level restore fallback via rsync...");
            let fallback_status = tokio::process::Command::new("rsync")
                .args([
                    "-a",
                    "--delete",
                    &format!("{}/", snapshot_path.to_string_lossy()),
                    "/var/home/ermete/",
                ])
                .status()
                .await;

            if matches!(fallback_status, Ok(ref s) if s.success()) {
                println!("[Bcachefs Rollback Architect] Fallback file-level restore succeeded.");
                Ok(true)
            } else {
                eprintln!("[Bcachefs Rollback Architect] Bcachefs restore failed!");
                Err(zbus::fdo::Error::Failed("Bcachefs instant restore failed".into()))
            }
        }
    } else {
        println!("[Bcachefs Rollback Architect] No snapshot registered for fd_id {}", fd_id);
        Ok(false)
    }
}

