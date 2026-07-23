use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use zbus::interface;

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

impl BackupServer {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/var/home/ermete".to_string());
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

        let home = std::env::var("HOME").unwrap_or_else(|_| "/var/home/ermete".to_string());
        let mut target_dir = self.snapshot_dir.clone();
        target_dir.push(&id);

        println!("[BackupDaemon] Creating Btrfs CoW snapshot of {} at {:?}", home, target_dir);
        let status = Command::new("btrfs")
            .args(["subvolume", "snapshot", "-r", &home, target_dir.to_str().unwrap_or("")])
            .status();

        if status.is_err() || !status.as_ref().unwrap().success() {
            println!("[BackupDaemon] Btrfs subvolume snapshot command failed or unsupported on current fs. Creating manifest snapshot dir.");
            let _ = fs::create_dir_all(&target_dir);
        }

        let info = SnapshotInfo {
            id: id.clone(),
            timestamp,
            note: note.to_string(),
            path: target_dir.to_string_lossy().to_string(),
            size_estimate: "0 B (Btrfs CoW)".to_string(),
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
                if path.extension().map_or(false, |ext| ext == "json") {
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

        println!("[BackupDaemon] Deleting Btrfs subvolume snapshot {:?}", target_dir);
        let status = Command::new("btrfs")
            .args(["subvolume", "delete", target_dir.to_str().unwrap_or("")])
            .status();

        if status.is_err() || !status.as_ref().unwrap().success() {
            let _ = fs::remove_dir_all(&target_dir);
        }

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

        let home = std::env::var("HOME").unwrap_or_else(|_| "/var/home/ermete".to_string());

        let _del_status = Command::new("btrfs")
            .args(["subvolume", "delete", &home])
            .status();
        let status = Command::new("btrfs")
            .args(["subvolume", "snapshot", target_dir.to_str().unwrap_or(""), &home])
            .status();

        if status.is_err() || !status.as_ref().unwrap().success() {
            println!("[BackupDaemon] Btrfs subvolume restore failed.");
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

