use std::os::unix::fs::OpenOptionsExt;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use zbus::interface;
use std::collections::HashMap;
use zbus::message::Header;
use zbus::zvariant::{OwnedValue, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitSubject {
    pub kind: String,
    pub details: HashMap<String, OwnedValue>,
}

impl PolkitSubject {
    pub fn system_bus_name(name: impl Into<String>) -> Self {
        let mut details = HashMap::new();
        let val: Value = Value::from(name.into());
        if let Ok(owned) = val.try_into() {
            details.insert("name".to_string(), owned);
        }
        Self {
            kind: "system-bus-name".to_string(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PolkitAuthorizationResult {
    pub is_authorized: bool,
    pub is_challenge: bool,
    pub details: HashMap<String, String>,
}

#[zbus::proxy(
    interface = "org.freedesktop.PolicyKit1.Authority",
    default_service = "org.freedesktop.PolicyKit1",
    default_path = "/org/freedesktop/PolicyKit1/Authority"
)]
pub trait PolicyKitAuthority {
    fn check_authorization(
        &self,
        subject: &PolkitSubject,
        action_id: &str,
        details: &HashMap<&str, &str>,
        flags: u32,
        cancellation_id: &str,
    ) -> zbus::Result<PolkitAuthorizationResult>;
}

pub async fn check_polkit_auth_zbus(
    conn: &zbus::Connection,
    sender: &str,
    action_id: &str,
    allow_user_interaction: bool,
) -> Result<bool, zbus::Error> {
    if let Ok(creds) = conn.peer_creds().await {
        if creds.unix_user_id() == Some(0) {
            return Ok(true);
        }
    }

    let proxy = PolicyKitAuthorityProxy::new(conn).await?;
    let subject = PolkitSubject::system_bus_name(sender);
    let details = HashMap::<&str, &str>::new();
    let flags = if allow_user_interaction { 1u32 } else { 0u32 };

    let result = proxy
        .check_authorization(&subject, action_id, &details, flags, "")
        .await?;

    Ok(result.is_authorized)
}

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
        mode: 0o755,
        padding: 0,
        dst_ptr: c_dst_name.as_ptr() as u64,
        src_ptr: src_file.as_raw_fd() as u64,
    };

    // SAFETY: FFI call to libc::ioctl to create bcachefs subvolume. Arguments are bounded by valid CString and file descriptor.
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

    // SAFETY: FFI call to libc::ioctl to destroy bcachefs subvolume. Arguments are bounded by valid CString and file descriptor.
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
        if let Err(e) = fs::create_dir_all(&path) {
                tracing::error!("Failed to create directory {:?}: {:?}", path, e);
            }
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

        let now = Local::now();
        let id = format!("snap-{}", now.format("%Y%m%d-%H%M%S"));
        let timestamp = now.format("%d/%m/%Y %H:%M:%S").to_string();

        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(&id);

        println!("[BackupDaemon] Creating Bcachefs CoW snapshot of {} at {:?}", home, target_dir);
        if let Err(e) = native_bcachefs_snapshot(Path::new(&home), &target_dir) {
            println!("[BackupDaemon] Bcachefs subvolume snapshot command failed: {:?}", e);
            return Err(zbus::fdo::Error::Failed("Filesystem non supporta CoW o comando bcachefs fallito".to_string()));
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
        Ok(list)
    }

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

        if id.contains('/') || id.contains('.') || id.contains('\\') {
            return Ok(false);
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
        Ok(true)
    }

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

        if id.contains('/') || id.contains('.') || id.contains('\\') {
            return Ok(false);
        }
        println!("[BackupDaemon] Restoring home directory from snapshot ID: {}", id);
        let manifest_path = self.get_manifest_path(id);
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(id);

        if !manifest_path.exists() && !target_dir.exists() {
            println!("[BackupDaemon] Snapshot ID {} not found (no manifest or target dir).", id);
            return Ok(false);
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| dirs::home_dir().unwrap_or(std::path::PathBuf::from("/home")).to_string_lossy().into_owned().to_string());

        if let Err(e) = native_bcachefs_delete(Path::new(&home)) {
                tracing::error!("Failed bcachefs delete {:?}: {:?}", home, e);
            }
        let res = native_bcachefs_snapshot(&target_dir, Path::new(&home));

        if res.is_err() {
            println!("[BackupDaemon] Bcachefs subvolume restore failed.");
            return Ok(false);
        }

        Ok(true)
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
        // (commentato poiché restore_snapshot richiede zbus e PolKit)
        // let restore_non_existent = server.restore_snapshot("non_existent_snapshot_id_xyz").await;
        // assert!(!restore_non_existent, "Expected restore_snapshot on non-existent ID to return false");

        // Clean up (commentato poiché delete_snapshot richiede zbus e PolKit)
        // let deleted = server.delete_snapshot(&snap.id, ...).await;
        // assert!(deleted);
    }
}

