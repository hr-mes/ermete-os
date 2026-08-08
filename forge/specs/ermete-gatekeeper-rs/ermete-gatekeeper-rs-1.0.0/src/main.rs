#![allow(unsafe_code)]

use ermete_gatekeeper_rs::allocator::BareMetalScudoAllocator;

#[global_allocator]
static GLOBAL: BareMetalScudoAllocator = BareMetalScudoAllocator::new();

mod bcachefs;
mod dbus;
mod fanotify;
mod hypervisor;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::io::unix::AsyncFd;
use zbus::connection::Builder;
use libc::{c_void, fanotify_event_metadata};

use bcachefs::{restore_bcachefs_snapshot_impl, take_bcachefs_snapshot};
use dbus::GatekeeperManager;
use fanotify::{
    respond_and_close, FAN_ALLOW, FAN_CLASS_CONTENT, FAN_DENY, FAN_EVENT_METADATA_LEN,
    FAN_MARK_ADD, FAN_MARK_MOUNT, FAN_NONBLOCK, FAN_OPEN_EXEC_PERM,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Ermete Gatekeeper Daemon with Bcachefs Rollback Architect Engine...");

    // SAFETY: Call libc fanotify_init syscall to create fanotify file descriptor.
    let fanotify_fd = unsafe {
        libc::fanotify_init(FAN_CLASS_CONTENT | FAN_NONBLOCK, (libc::O_RDONLY | libc::O_LARGEFILE) as u32)
    };

    if fanotify_fd < 0 {
        let err = std::io::Error::last_os_error();
        eprintln!("Failed to initialize fanotify (are you root?): {}", err);
        return Err(err.into());
    }

    println!("fanotify initialized. Marking mounts...");

    let mounts = ["/var/home", "/tmp", "/var/tmp", "/opt"];
    for mount in mounts.iter() {
        let path = match std::ffi::CString::new(*mount) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Invalid mount path {}: {}", mount, e);
                continue;
            }
        };
        // SAFETY: Call libc fanotify_mark syscall with valid CString pointer.
        let ret = unsafe {
            libc::fanotify_mark(
                fanotify_fd,
                FAN_MARK_ADD | FAN_MARK_MOUNT,
                FAN_OPEN_EXEC_PERM,
                libc::AT_FDCWD,
                path.as_ptr(),
            )
        };
        if ret < 0 {
            eprintln!("Failed to mark {}: {}", mount, std::io::Error::last_os_error());
        } else {
            println!("Marked {} for execution monitoring.", mount);
        }
    }

    let pending_events = Arc::new(std::sync::Mutex::new(HashMap::new()));
    let pending_snapshots = Arc::new(std::sync::Mutex::new(HashMap::new()));
    let manager = GatekeeperManager::new(
        fanotify_fd,
        pending_events.clone(),
        pending_snapshots.clone(),
    );

    let conn = Builder::system()?
        .name("os.ermete.Gatekeeper")?
        .serve_at("/os/ermete/Gatekeeper", manager)?
        .build()
        .await?;

    let iface_ref = conn.object_server().interface::<_, GatekeeperManager>("/os/ermete/Gatekeeper").await?;
    let signal_ctxt = iface_ref.signal_context().clone();

    let async_fd = AsyncFd::new(fanotify_fd)?;

    println!("Ermete Gatekeeper listening for execution events...");

    loop {
        let mut guard = async_fd.readable().await?;
        
        let mut buf = [0u8; 4096];
        loop {
            // SAFETY: Read from fanotify_fd into local byte buffer.
            let n = unsafe {
                libc::read(fanotify_fd, buf.as_mut_ptr() as *mut c_void, buf.len())
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
                eprintln!("Error reading fanotify: {}", err);
                break;
            }
            if n == 0 {
                break;
            }

            let mut offset = 0;
            while offset < n as usize {
                if offset + FAN_EVENT_METADATA_LEN > n as usize {
                    break;
                }
                
                // SAFETY: Calculate pointer offset within read buffer bounds.
                let ptr = unsafe { buf.as_ptr().add(offset) as *const fanotify_event_metadata };
                // SAFETY: Dereference pointer to fanotify_event_metadata struct after bounds check.
                let metadata: &fanotify_event_metadata = unsafe { &*ptr };
                
                if metadata.vers != libc::FANOTIFY_METADATA_VERSION {
                    eprintln!("Mismatch fanotify version");
                    offset += metadata.event_len as usize;
                    continue;
                }

                if metadata.fd >= 0 {
                    let path_str = format!("/proc/self/fd/{}", metadata.fd);
                    let target_path = tokio::fs::read_link(&path_str).await.unwrap_or_default();
                    let target_path_str = target_path.to_string_lossy().into_owned();

                    // Check for quarantine attribute via stable /proc/self/fd path (TOCTOU-safe) offloaded to spawn_blocking
                    let path_str_clone = path_str.clone();
                    let is_quarantined = tokio::task::spawn_blocking(move || {
                        xattr::get(&path_str_clone, "user.ermete.quarantine")
                            .ok()
                            .flatten()
                            .is_some()
                    }).await.unwrap_or(false);

                    if is_quarantined {
                        let fd_id = uuid::Uuid::new_v4().to_string();
                        
                        println!("Intercepted execution of quarantined file: {}", target_path_str);

                        // Take atomic Bcachefs subvolume snapshot BEFORE prompt or kill
                        if let Some(snap_path) = take_bcachefs_snapshot(&fd_id).await {
                            pending_snapshots.lock().unwrap_or_else(|e| e.into_inner()).insert(fd_id.clone(), snap_path);
                        }
                        
                        // Store the fd
                        pending_events.lock().unwrap_or_else(|e| e.into_inner()).insert(fd_id.clone(), metadata.fd);
                        
                        // Ask the UI to prompt the user
                        if let Err(e) = GatekeeperManager::prompt_required(&signal_ctxt, &fd_id, &target_path_str).await {
                            eprintln!("Failed to send prompt_required signal: {}", e);
                            // Fallback deny if UI is dead: trigger instant restore and deny execution
                            let _ = restore_bcachefs_snapshot_impl(&fd_id, &pending_snapshots).await;
                            respond_and_close(fanotify_fd, metadata.fd, FAN_DENY);
                        }
                    } else {
                        // Allow immediately
                        respond_and_close(fanotify_fd, metadata.fd, FAN_ALLOW);
                    }
                }
                
                offset += metadata.event_len as usize;
            }
        }
        guard.clear_ready();
    }
}
