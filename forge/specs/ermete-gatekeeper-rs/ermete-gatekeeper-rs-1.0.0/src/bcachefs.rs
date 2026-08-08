use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Takes an atomic Bcachefs subvolume snapshot of `/var/home/ermete` prior to prompt or kill.
pub async fn take_bcachefs_snapshot(fd_id: &str) -> Option<PathBuf> {
    let snapshot_dir = PathBuf::from("/var/home/.snapshots");
    let _ = tokio::fs::create_dir_all(&snapshot_dir).await;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let snapshot_path = snapshot_dir.join(format!("gatekeeper-pre-exec-{}-{}", fd_id, timestamp));

    println!(
        "[Bcachefs Rollback Architect] Creating atomic CoW snapshot of /var/home/ermete at {:?}",
        snapshot_path
    );

    let status = tokio::process::Command::new("bcachefs")
        .args([
            "subvolume",
            "snapshot",
            "/var/home/ermete",
            snapshot_path.to_str().unwrap_or(""),
        ])
        .status()
        .await;

    if matches!(status, Ok(ref s) if s.success()) {
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
    pending_snapshots: &Arc<std::sync::Mutex<HashMap<String, PathBuf>>>,
) -> zbus::fdo::Result<bool> {
    let snapshot_path = {
        let mut snapshots = pending_snapshots.lock().unwrap_or_else(|e| e.into_inner());
        snapshots.remove(fd_id)
    };

    if let Some(snapshot_path) = snapshot_path {
        println!(
            "[Bcachefs Rollback Architect] Confirmed infection / execution denial for fd_id {}. Triggering instant Bcachefs restore from {:?}",
            fd_id, snapshot_path
        );

        let target_subvol = "/var/home/ermete";
        let del_status = tokio::process::Command::new("bcachefs")
            .args(["subvolume", "delete", target_subvol])
            .status()
            .await;

        if !matches!(del_status, Ok(ref s) if s.success()) {
            eprintln!("[Bcachefs Rollback Architect] Subvolume delete returned non-zero; attempting snapshot restore & fallback...");
        }

        let restore_status = tokio::process::Command::new("bcachefs")
            .args([
                "subvolume",
                "snapshot",
                snapshot_path.to_str().unwrap_or(""),
                target_subvol,
            ])
            .status()
            .await;

        if matches!(restore_status, Ok(ref s) if s.success()) {
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
                    &format!("{}/", target_subvol),
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
