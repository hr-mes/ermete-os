use std::collections::HashMap;
use std::error::Error;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::sync::Arc;
use tokio::io::unix::AsyncFd;
use tokio::sync::Mutex;
use zbus::{connection::Builder, interface};
use libc::{c_void, fanotify_event_metadata, fanotify_response};

const FAN_CLASS_CONTENT: u32 = 0x00000004;
const FAN_NONBLOCK: u32 = 0x00000002;
const FAN_MARK_ADD: u32 = 0x00000001;
const FAN_MARK_MOUNT: u32 = 0x00000010;
const FAN_OPEN_EXEC_PERM: u64 = 0x00010000;
const FAN_ALLOW: u32 = 0x01;
const FAN_DENY: u32 = 0x02;
const FAN_EVENT_METADATA_LEN: usize = std::mem::size_of::<fanotify_event_metadata>();

struct GatekeeperManager {
    fanotify_fd: RawFd,
    pending_events: Arc<Mutex<HashMap<u64, i32>>>, // fd_id -> event_fd
}

#[interface(name = "os.ermete.Gatekeeper")]
impl GatekeeperManager {
    async fn approve_execution(&self, fd_id: u64) -> zbus::fdo::Result<()> {
        let mut pending = self.pending_events.lock().await;
        if let Some(event_fd) = pending.remove(&fd_id) {
            let fd_path = format!("/proc/self/fd/{}", event_fd);
            let target_path = std::fs::read_link(&fd_path)
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to resolve fd: {}", e)))?;

            // Remove quarantine xattr via stable /proc/self/fd path (TOCTOU-safe)
            let _ = xattr::remove(&fd_path, "user.ermete.quarantine");

            // Spawn inside Bubblewrap sandbox, then DENY original unsandboxed execution
            let target_str = target_path.to_string_lossy().to_string();
            let sandbox_result = std::process::Command::new("bwrap")
                .arg("--unshare-all")
                .arg("--share-net")
                .arg("--ro-bind").arg("/usr").arg("/usr")
                .arg("--ro-bind").arg("/lib").arg("/lib")
                .arg("--ro-bind").arg("/lib64").arg("/lib64")
                .arg("--ro-bind").arg("/etc").arg("/etc")
                .arg("--proc").arg("/proc")
                .arg("--dev").arg("/dev")
                .arg("--dir").arg("/tmp")
                .arg("--ro-bind").arg(&target_path).arg(&target_path)
                .arg("--").arg(&target_path)
                .spawn();

            match sandbox_result {
                Ok(_child) => {
                    // Sandbox spawned — DENY original unsandboxed execution
                    let mut response = fanotify_response { fd: event_fd, response: FAN_DENY };
                    unsafe {
                        libc::write(self.fanotify_fd, &mut response as *mut _ as *const c_void, std::mem::size_of::<fanotify_response>());
                        libc::close(event_fd);
                    }
                }
                Err(e) => {
                    eprintln!("bwrap failed for {}: {}. Denying.", target_str, e);
                    let mut response = fanotify_response { fd: event_fd, response: FAN_DENY };
                    unsafe {
                        libc::write(self.fanotify_fd, &mut response as *mut _ as *const c_void, std::mem::size_of::<fanotify_response>());
                        libc::close(event_fd);
                    }
                }
            }
            Ok(())
        } else {
            Err(zbus::fdo::Error::InvalidArgs(format!("No pending event for id {}", fd_id)))
        }
    }

    async fn deny_execution(&self, fd_id: u64) -> zbus::fdo::Result<()> {
        let mut pending = self.pending_events.lock().await;
        if let Some(event_fd) = pending.remove(&fd_id) {
            // Deny execution
            let mut response = fanotify_response {
                fd: event_fd,
                response: FAN_DENY,
            };
            unsafe {
                libc::write(
                    self.fanotify_fd,
                    &mut response as *mut _ as *const c_void,
                    std::mem::size_of::<fanotify_response>(),
                );
                libc::close(event_fd);
            }
            Ok(())
        } else {
            Err(zbus::fdo::Error::InvalidArgs(format!("No pending event for id {}", fd_id)))
        }
    }

    #[zbus(signal)]
    async fn prompt_required(
        signal_ctxt: &zbus::SignalContext<'_>,
        fd_id: u64,
        app_name: &str,
    ) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Ermete Gatekeeper Daemon...");

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
        let path = std::ffi::CString::new(*mount).unwrap();
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

    let pending_events = Arc::new(Mutex::new(HashMap::new()));
    let manager = GatekeeperManager {
        fanotify_fd,
        pending_events: pending_events.clone(),
    };

    let conn = Builder::system()?
        .name("os.ermete.Gatekeeper")?
        .serve_at("/os/ermete/Gatekeeper", manager)?
        .build()
        .await?;

    let iface_ref = conn.object_server().interface::<_, GatekeeperManager>("/os/ermete/Gatekeeper").await?;
    let signal_ctxt = iface_ref.signal_context().clone();

    let async_fd = AsyncFd::new(fanotify_fd)?;
    let mut next_id: u64 = 1;

    println!("Ermete Gatekeeper listening for execution events...");

    loop {
        let mut guard = async_fd.readable().await?;
        
        let mut buf = [0u8; 4096];
        loop {
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
                
                let metadata: &fanotify_event_metadata = unsafe {
                    &*(buf.as_ptr().add(offset) as *const fanotify_event_metadata)
                };
                
                if metadata.vers != libc::FANOTIFY_METADATA_VERSION {
                    eprintln!("Mismatch fanotify version");
                    offset += metadata.event_len as usize;
                    continue;
                }

                if metadata.fd >= 0 {
                    let mut is_quarantined = false;
                    let path = format!("/proc/self/fd/{}", metadata.fd);
                    let target_path = std::fs::read_link(&path).unwrap_or_default();
                    let target_path_str = target_path.to_string_lossy().to_string();

                    // Check for quarantine attribute via stable /proc/self/fd path (TOCTOU-safe)
                    if let Ok(Some(_)) = xattr::get(&path, "user.ermete.quarantine") {
                        is_quarantined = true;
                    }

                    if is_quarantined {
                        let fd_id = next_id;
                        next_id += 1;
                        
                        println!("Intercepted execution of quarantined file: {}", target_path_str);
                        
                        // Store the fd
                        pending_events.lock().await.insert(fd_id, metadata.fd);
                        
                        // Ask the UI to prompt the user
                        if let Err(e) = GatekeeperManager::prompt_required(&signal_ctxt, fd_id, &target_path_str).await {
                            eprintln!("Failed to send prompt_required signal: {}", e);
                            // Fallback deny if UI is dead
                            let mut response = fanotify_response {
                                fd: metadata.fd,
                                response: FAN_DENY,
                            };
                            unsafe {
                                libc::write(fanotify_fd, &mut response as *mut _ as *const c_void, std::mem::size_of::<fanotify_response>());
                                libc::close(metadata.fd);
                            }
                        }
                    } else {
                        // Allow immediately
                        let mut response = fanotify_response {
                            fd: metadata.fd,
                            response: FAN_ALLOW,
                        };
                        unsafe {
                            libc::write(fanotify_fd, &mut response as *mut _ as *const c_void, std::mem::size_of::<fanotify_response>());
                            libc::close(metadata.fd);
                        }
                    }
                }
                
                offset += metadata.event_len as usize;
            }
        }
        guard.clear_ready();
    }
}
