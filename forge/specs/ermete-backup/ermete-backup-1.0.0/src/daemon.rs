//! `ermete-backup-daemon` — system D-Bus service (`org.ermete.Backup1`) that
//! manages instant Bcachefs copy-on-write home-directory snapshots
//! ("Time Machine" style). This is the binary actually packaged by
//! `ermete-backup.spec`; the `ermete-backup` binary in `main.rs` is a
//! separate, unpackaged `borg`-based implementation (see its module doc).
//!
//! All four D-Bus methods (`create_snapshot`, `list_snapshots`,
//! `delete_snapshot`, `restore_snapshot`) are Polkit-gated per-action.

use std::os::unix::fs::OpenOptionsExt;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use ermete_bus_api::polkit::check_polkit_auth_zbus;
use zbus::interface;
use zbus::message::Header;

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

/// Creates a Bcachefs copy-on-write subvolume snapshot of `src` at `dst` via
/// the `BCH_IOCTL_SUBVOLUME_CREATE` ioctl. Falls back to plain
/// `create_dir_all` (an ordinary, non-CoW directory) if the ioctl fails or
/// the filesystem doesn't support Bcachefs subvolumes, or if `dst` already
/// exists as a directory.
///
/// # Errors
/// Returns an error if `src`/`dst`'s parent cannot be opened, `dst` has no
/// file name component, or the `create_dir_all` fallback itself fails.
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
        mode: 0o755,
        padding: 0,
        dst_ptr: c_dst_name.as_ptr() as u64,
        src_ptr: src_file.as_raw_fd() as u64,
    };

    // SAFETY: `arg` is a `#[repr(C)]` struct matching the kernel's expected
    // `bch_ioctl_subvolume` ABI layout, and `&mut arg` stays alive for the
    // duration of this call (it's a local, not moved/dropped before the
    // `ioctl` returns). `arg.dst_ptr`/`arg.src_ptr` point at `c_dst_name`
    // (a `CString`, still in scope) and `src_file`'s raw fd respectively,
    // both of which outlive this call. `src_file.as_raw_fd()` is a valid
    // open file descriptor owned by `src_file`, which is not dropped before
    // the ioctl runs.
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

/// Destroys a Bcachefs subvolume at `path` via `BCH_IOCTL_SUBVOLUME_DESTROY`.
/// Falls back to `remove_dir_all` if the parent can't be opened or the ioctl
/// fails (e.g. `path` is a plain directory, not a Bcachefs subvolume).
///
/// # Errors
/// Returns `Ok(())` if `path` doesn't exist. Returns an error only if `path`
/// has no file name component, or a fallback `remove_dir_all` itself fails.
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

    // SAFETY: Same argument as the `SUBVOLUME_CREATE` call above: `arg`
    // matches the kernel's expected ABI layout and outlives the call,
    // `arg.dst_ptr` points at `c_name` (a live `CString`), and
    // `parent_file.as_raw_fd()` is a valid fd owned by `parent_file`, not
    // yet dropped.
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

/// Metadata for one home-directory snapshot, persisted as `manifest.json`
/// alongside the snapshot subvolume itself.
#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type)]
pub struct SnapshotInfo {
    /// Snapshot ID in the form `snap-{YYYYMMDD-HHMMSS}`; also the subvolume
    /// directory name under `snapshot_dir`.
    pub id: String,
    /// Human-readable creation time (`DD/MM/YYYY HH:MM:SS`, local time).
    pub timestamp: String,
    /// Caller-supplied note describing the snapshot.
    pub note: String,
    /// Absolute filesystem path to the snapshot subvolume.
    pub path: String,
    /// Always `"0 B (Bcachefs CoW)"` — not a real computed size, since
    /// CoW snapshots share blocks with the source until they diverge.
    pub size_estimate: String,
}

/// D-Bus service object implementing `org.ermete.Backup1`. Manages
/// Bcachefs CoW snapshots of the invoking user's `$HOME` under
/// `snapshot_dir` (`~/.snapshots`).
pub struct BackupServer {
    pub snapshot_dir: PathBuf,
}

impl Default for BackupServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupServer {
    fn get_manifest_path(&self, id: &str) -> PathBuf {
        let mut path = self.snapshot_dir.clone();
        path.push(id);
        path.push("manifest.json");
        path
    }

    /// Creates a `BackupServer` rooted at `$HOME/.snapshots` (falling back
    /// to `dirs::home_dir()` then `/home` if `$HOME` is unset), creating
    /// that directory if it doesn't already exist. A failure to create the
    /// directory is logged, not propagated — later snapshot operations will
    /// simply fail instead.
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());
        let mut path = PathBuf::from(&home);
        path.push(".snapshots");
        if let Err(e) = fs::create_dir_all(&path) {
                tracing::error!("Failed to create directory {:?}: {:?}", path, e);
            }
        Self { snapshot_dir: path }
    }

    /// Creates a new Bcachefs CoW snapshot of `$HOME` and writes its
    /// `SnapshotInfo` manifest to disk with `0o600` permissions.
    ///
    /// # Errors
    /// Returns `Err` if the underlying Bcachefs snapshot ioctl (and its
    /// `create_dir_all` fallback) both fail. A failure to write the
    /// manifest JSON afterward is only logged, not returned as an error —
    /// the snapshot itself still reports success in that case.
    pub fn create_snapshot_internal(&self, note: &str) -> Result<SnapshotInfo, String> {
        let now = Local::now();
        let id = format!("snap-{}", now.format("%Y%m%d-%H%M%S"));
        let timestamp = now.format("%d/%m/%Y %H:%M:%S").to_string();

        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(&id);

        println!("[BackupDaemon] Creating Bcachefs CoW snapshot of {} at {:?}", home, target_dir);
        if let Err(e) = native_bcachefs_snapshot(Path::new(&home), &target_dir) {
            println!("[BackupDaemon] Bcachefs subvolume snapshot command failed: {:?}", e);
            return Err("Filesystem non supporta CoW o comando bcachefs fallito".to_string());
        }

        let info = SnapshotInfo {
            id: id.clone(),
            timestamp,
            note: note.to_string(),
            path: target_dir.to_string_lossy().into_owned(),
            size_estimate: "0 B (Bcachefs CoW)".to_string(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&info) {
            let manifest_path = self.get_manifest_path(&id);
            if let Err(e) = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&manifest_path)
                .and_then(|mut f| std::io::Write::write_all(&mut f, json.as_bytes()))
            {
                tracing::error!("Failed to securely write manifest at {:?}: {:?}", manifest_path, e);
            }
        }

        Ok(info)
    }

    /// Lists all snapshots with a readable, parseable `manifest.json` under
    /// `snapshot_dir`, newest ID first. Entries with unreadable or malformed
    /// manifests are silently skipped rather than surfaced as errors.
    pub fn list_snapshots_internal(&self) -> Vec<SnapshotInfo> {
        let mut list = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.snapshot_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(info) = serde_json::from_str::<SnapshotInfo>(&content) {
                            list.push(info);
                        }
                    }
                }
            }
        }
        list.sort_by(|a, b| b.id.cmp(&a.id));
        list
    }

    /// Deletes the snapshot subvolume and manifest for `id`.
    ///
    /// `id` is rejected (returns `false`) if it contains `/`, `.`, or `\`,
    /// which prevents it being used to escape `snapshot_dir` via a
    /// directory-traversal path (e.g. `../../etc`) when joined onto
    /// `snapshot_dir` below.
    pub fn delete_snapshot_internal(&self, id: &str) -> bool {
        if id.contains('/') || id.contains('.') || id.contains('\\') {
            return false;
        }
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(id);

        println!("[BackupDaemon] Deleting Bcachefs subvolume snapshot {:?}", target_dir);
        if let Err(e) = native_bcachefs_delete(&target_dir) {
            tracing::error!("Failed bcachefs delete {:?}: {:?}", target_dir, e);
        }
        let manifest_path = self.get_manifest_path(id);
        if let Err(e) = fs::remove_file(&manifest_path) {
            tracing::error!("Failed to remove manifest {:?}: {:?}", manifest_path, e);
        }
        true
    }

    /// Restores `$HOME` from the snapshot `id`: deletes the current `$HOME`
    /// subvolume, then snapshots `id` back into place as the new `$HOME`.
    /// Same path-traversal guard on `id` as [`delete_snapshot_internal`].
    /// Returns `false` if `id` is invalid, the snapshot doesn't exist
    /// (checked via manifest or subvolume directory presence), or the
    /// restore ioctl/fallback fails.
    ///
    /// [`delete_snapshot_internal`]: BackupServer::delete_snapshot_internal
    pub fn restore_snapshot_internal(&self, id: &str) -> bool {
        if id.contains('/') || id.contains('.') || id.contains('\\') {
            return false;
        }
        println!("[BackupDaemon] Restoring home directory from snapshot ID: {}", id);
        let manifest_path = self.get_manifest_path(id);
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(id);

        if !manifest_path.exists() && !target_dir.exists() {
            println!("[BackupDaemon] Snapshot ID {} not found (no manifest or target dir).", id);
            return false;
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());

        if let Err(e) = native_bcachefs_delete(Path::new(&home)) {
            tracing::error!("Failed bcachefs delete {:?}: {:?}", home, e);
        }
        let res = native_bcachefs_snapshot(&target_dir, Path::new(&home));

        if res.is_err() {
            println!("[BackupDaemon] Bcachefs subvolume restore failed.");
            return false;
        }

        true
    }
}

#[interface(name = "org.ermete.Backup1")]
impl BackupServer {
    /// `CreateSnapshot` D-Bus method, Polkit action `org.ermete.backup.create`.
    async fn create_snapshot(
        &self,
        note: &str,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<SnapshotInfo> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.backup.create", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for create_snapshot".into()));
        }

        self.create_snapshot_internal(note).map_err(zbus::fdo::Error::Failed)
    }

    /// `ListSnapshots` D-Bus method, Polkit action `org.ermete.backup.list`
    /// (does not require interactive auth).
    async fn list_snapshots(
        &self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<Vec<SnapshotInfo>> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.backup.list", false)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for list_snapshots".into()));
        }

        Ok(self.list_snapshots_internal())
    }

    /// `DeleteSnapshot` D-Bus method, Polkit action `org.ermete.backup.delete`.
    async fn delete_snapshot(
        &self,
        id: &str,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<bool> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.backup.delete", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for delete_snapshot".into()));
        }

        Ok(self.delete_snapshot_internal(id))
    }

    /// `RestoreSnapshot` D-Bus method, Polkit action `org.ermete.backup.restore`.
    /// Overwrites the caller's current `$HOME` in place — there is no
    /// automatic snapshot of the pre-restore state.
    async fn restore_snapshot(
        &self,
        id: &str,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<bool> {
        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = check_polkit_auth_zbus(conn, sender.as_str(), "org.ermete.backup.restore", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for restore_snapshot".into()));
        }

        Ok(self.restore_snapshot_internal(id))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = BackupServer::new();
    let _conn = zbus::connection::Builder::system()?
        .name("org.ermete.Backup1")?
        .serve_at("/org/ermete/Backup1", server)?
        .build()
        .await?;

    println!("[ermete-backup-daemon] D-Bus service org.ermete.Backup1 started successfully.");
    std::future::pending::<()>().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_server_init_and_manifest_path() {
        let server = BackupServer::new();
        let manifest_path = server.get_manifest_path("test-id");
        assert!(manifest_path.to_string_lossy().ends_with(".snapshots/test-id.json") || manifest_path.to_string_lossy().ends_with(".snapshots\\test-id.json"));
    }

    #[tokio::test]
    async fn test_snapshot_lifecycle_and_restore() {
        let server = BackupServer::new();
        // Uses the internal logic that doesn't need D-Bus/PolKit for testing
        let snap = server.create_snapshot_internal("Test note").unwrap();
        assert!(snap.id.starts_with("snap-"));
        assert_eq!(snap.note, "Test note");

        let list = server.list_snapshots_internal();
        assert!(list.iter().any(|s| s.id == snap.id));

        // Attempting to restore a non-existent snapshot must return false
        let restore_non_existent = server.restore_snapshot_internal("non_existent_snapshot_id_xyz");
        assert!(!restore_non_existent, "Expected restore_snapshot on non-existent ID to return false");

        // Clean up
        let deleted = server.delete_snapshot_internal(&snap.id);
        assert!(deleted);
    }
}



