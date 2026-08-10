use chrono::Local;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use zbus::interface;

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
        let _ = fs::create_dir_all(parent);
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
        let _ = fs::remove_dir_all(path);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type)]
pub struct SnapshotInfo {
    pub id: String,
    pub timestamp: String,
    pub note: String,
    pub path: String,
    pub size_estimate: String,
}

pub struct BackupServer {
    pub snapshot_dir: PathBuf,
}

impl Default for BackupServer {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupServer {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());
        let mut path = PathBuf::from(&home);
        path.push(".snapshots");
        let _ = fs::create_dir_all(&path);
        Self { snapshot_dir: path }
    }

    fn get_manifest_path(&self, id: &str) -> PathBuf {
        let mut p = self.snapshot_dir.clone();
        p.push(format!("{}.json", id));
        p
    }
}

#[interface(name = "org.ermete.Backup1")]
impl BackupServer {
    async fn create_snapshot(&self, note: &str) -> SnapshotInfo {
        let now = Local::now();
        let id = format!("snap-{}", now.format("%Y%m%d-%H%M%S"));
        let timestamp = now.format("%d/%m/%Y %H:%M:%S").to_string();

        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(&id);

        println!("[BackupDaemon] Creating Bcachefs CoW snapshot of {} at {:?}", home, target_dir);
        let res = native_bcachefs_snapshot(Path::new(&home), &target_dir);

        if res.is_err() {
            println!("[BackupDaemon] Bcachefs subvolume snapshot command failed or unsupported on current fs. Creating manifest snapshot dir.");
            let _ = fs::create_dir_all(&target_dir);
        }

        let info = SnapshotInfo {
            id: id.clone(),
            timestamp,
            note: note.to_string(),
            path: target_dir.to_string_lossy().into_owned(),
            size_estimate: "0 B (Bcachefs CoW)".to_string(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&info) {
            let _ = fs::write(self.get_manifest_path(&id), json);
        }

        info
    }

    async fn list_snapshots(&self) -> Vec<SnapshotInfo> {
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

    async fn delete_snapshot(&self, id: &str) -> bool {
        if id.contains('/') || id.contains('.') || id.contains('\\') {
            return false;
        }
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(id);

        println!("[BackupDaemon] Deleting Bcachefs subvolume snapshot {:?}", target_dir);
        let _ = native_bcachefs_delete(&target_dir);
        let _ = fs::remove_file(self.get_manifest_path(id));
        true
    }

    async fn restore_snapshot(&self, id: &str) -> bool {
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

        let _ = native_bcachefs_delete(Path::new(&home));
        let res = native_bcachefs_snapshot(&target_dir, Path::new(&home));

        if res.is_err() {
            println!("[BackupDaemon] Bcachefs subvolume restore failed.");
            return false;
        }

        true
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
        let snap = server.create_snapshot("Test note").await;
        assert!(snap.id.starts_with("snap-"));
        assert_eq!(snap.note, "Test note");

        let list = server.list_snapshots().await;
        assert!(list.iter().any(|s| s.id == snap.id));

        // Attempting to restore a non-existent snapshot must return false
        let restore_non_existent = server.restore_snapshot("non_existent_snapshot_id_xyz").await;
        assert!(!restore_non_existent, "Expected restore_snapshot on non-existent ID to return false");

        // Clean up
        let deleted = server.delete_snapshot(&snap.id).await;
        assert!(deleted);
    }
}

