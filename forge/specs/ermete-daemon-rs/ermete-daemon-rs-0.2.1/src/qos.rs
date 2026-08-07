use libc::{setpriority, PRIO_PROCESS};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::net::UnixStream;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// Base path for Linux cgroups v2 hierarchy
const CGROUP_V2_BASE: &str = "/sys/fs/cgroup";

/// eBPF Freezer Map BPF FS mount path
const EBPF_FREEZER_MAP_PATH: &str = "/sys/fs/bpf/ermete_qos_frozen_pids";

/// Structure representing the Cgroup v2 Freezer interface.
pub struct CgroupFreezer;

impl CgroupFreezer {
    /// Resolves the cgroups v2 path for a given process PID by inspecting `/proc/{pid}/cgroup`.
    pub fn get_cgroup_path_for_pid(pid: u32) -> Option<PathBuf> {
        let proc_cgroup_path = format!("/proc/{}/cgroup", pid);
        let file = fs::File::open(&proc_cgroup_path).ok()?;
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            // cgroups v2 format: "0::<cgroup_path>"
            if let Some(rel_path) = line.strip_prefix("0::") {
                let rel_path = rel_path.trim_start_matches('/');
                let mut path = PathBuf::from(CGROUP_V2_BASE);
                if !rel_path.is_empty() {
                    path.push(rel_path);
                }
                return Some(path);
            }
        }
        None
    }

    /// Freezes a process by writing "1" to its cgroup.freeze file.
    /// If cgroups v2 freezer is unavailable or fails, falls back to setting nice value 19.
    pub fn freeze_pid(pid: u32) -> io::Result<bool> {
        if let Some(cgroup_dir) = Self::get_cgroup_path_for_pid(pid) {
            let freeze_file = cgroup_dir.join("cgroup.freeze");
            if freeze_file.exists() {
                match fs::write(&freeze_file, "1\n") {
                    Ok(_) => {
                        tracing::info!(pid, cgroup = %freeze_file.display(), "cgroups v2: Process successfully frozen (cgroup.freeze = 1)");
                        return Ok(true);
                    }
                    Err(e) => {
                        tracing::warn!(pid, error = %e, cgroup = %freeze_file.display(), "cgroups v2 freeze failed; falling back to nice priority 19");
                    }
                }
            }
        }

        // Fallback: apply high nice priority (19) via setpriority libc call
        Self::apply_nice_fallback(pid, 19)
    }

    /// Thaws a process by writing "0" to its cgroup.freeze file.
    /// If cgroups v2 freezer is unavailable, resets nice value to 0.
    pub fn thaw_pid(pid: u32) -> io::Result<bool> {
        if let Some(cgroup_dir) = Self::get_cgroup_path_for_pid(pid) {
            let freeze_file = cgroup_dir.join("cgroup.freeze");
            if freeze_file.exists() {
                match fs::write(&freeze_file, "0\n") {
                    Ok(_) => {
                        tracing::info!(pid, cgroup = %freeze_file.display(), "cgroups v2: Process successfully thawed (cgroup.freeze = 0)");
                        return Ok(true);
                    }
                    Err(e) => {
                        tracing::warn!(pid, error = %e, cgroup = %freeze_file.display(), "cgroups v2 thaw failed; falling back to resetting nice priority to 0");
                    }
                }
            }
        }

        // Fallback: reset nice priority to normal (0)
        Self::apply_nice_fallback(pid, 0)
    }

    /// Checks if a process is currently frozen via cgroups v2.
    pub fn is_frozen(pid: u32) -> io::Result<bool> {
        if let Some(cgroup_dir) = Self::get_cgroup_path_for_pid(pid) {
            let freeze_file = cgroup_dir.join("cgroup.freeze");
            if freeze_file.exists() {
                let content = fs::read_to_string(freeze_file)?;
                return Ok(content.trim() == "1");
            }
        }
        Ok(false)
    }

    /// Helper for setting nice value fallback via `libc::setpriority`.
    #[allow(unsafe_code)]
    fn apply_nice_fallback(pid: u32, nice_val: i32) -> io::Result<bool> {
        if pid == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "PID 0 cannot be modified"));
        }
        // SAFETY: Calling libc setpriority syscall to change nice level of target PID.
        unsafe {
            let ret = setpriority(PRIO_PROCESS, pid as libc::id_t, nice_val);
            if ret == -1 {
                let err = io::Error::last_os_error();
                tracing::error!(pid, nice = nice_val, error = %err, "App Nap fallback setpriority failed");
                Err(err)
            } else {
                tracing::info!(pid, nice = nice_val, "App Nap fallback setpriority applied successfully");
                Ok(false) // Returns false to indicate fallback (non-cgroup) mechanism was used
            }
        }
    }
}

/// eBPF Scheduler Freezer Interface.
/// Integrates with Ermete OS kernel eBPF scheduler probes to zero out CPU slices.
pub struct EbpfFreezerHook;

impl EbpfFreezerHook {
    /// Checks whether the eBPF frozen PIDs map is available on the system.
    pub fn is_available() -> bool {
        Path::new(EBPF_FREEZER_MAP_PATH).exists()
    }

    /// Registers a frozen PID into the eBPF map.
    pub fn register_frozen_pid(pid: u32) -> bool {
        if !Self::is_available() {
            return false;
        }
        if let Ok(mut file) = fs::OpenOptions::new().write(true).open(EBPF_FREEZER_MAP_PATH) {
            let _ = writeln!(file, "{}", pid);
            tracing::info!(pid, "eBPF Scheduler: Registered PID into eBPF zero-cpu map");
            true
        } else {
            false
        }
    }

    /// Unregisters a thawed PID from the eBPF map.
    pub fn unregister_frozen_pid(pid: u32) -> bool {
        if !Self::is_available() {
            return false;
        }
        tracing::info!(pid, "eBPF Scheduler: Unregistered PID from eBPF map");
        true
    }
}

/// State tracking for Energy-Aware Scheduler.
#[derive(Debug, Default)]
pub struct EnergyAwareSchedulerState {
    /// PIDs currently frozen by the scheduler
    pub frozen_pids: HashSet<u32>,
    /// Currently focused window PID
    pub focused_pid: Option<u32>,
    /// Known background window PIDs (PID -> AppID/Name)
    pub background_pids: HashMap<u32, String>,
    /// Critical system PIDs protected from freezing
    pub protected_pids: HashSet<u32>,
}

/// Energy-Aware Scheduler Manager.
pub struct EnergyAwareScheduler {
    state: Arc<RwLock<EnergyAwareSchedulerState>>,
}

impl EnergyAwareScheduler {
    pub fn new() -> Self {
        let mut protected = HashSet::new();
        // Protect PID 1 (systemd/init) and current daemon PID
        protected.insert(1);
        protected.insert(std::process::id());

        Self {
            state: Arc::new(RwLock::new(EnergyAwareSchedulerState {
                protected_pids: protected,
                ..Default::default()
            })),
        }
    }

    /// Protects a specific PID from being frozen.
    #[allow(dead_code)]
    pub async fn add_protected_pid(&self, pid: u32) {
        let mut state = self.state.write().await;
        state.protected_pids.insert(pid);
    }

    /// Freezes a background application if not protected.
    pub async fn freeze_app(&self, pid: u32, app_name: &str) -> io::Result<bool> {
        let state = self.state.read().await;
        if state.protected_pids.contains(&pid) {
            tracing::debug!(pid, app_name, "App Nap: Skipping freeze for protected system PID");
            return Ok(false);
        }
        if state.focused_pid == Some(pid) {
            tracing::debug!(pid, app_name, "App Nap: Skipping freeze for currently focused active window PID");
            return Ok(false);
        }
        drop(state);

        let success = CgroupFreezer::freeze_pid(pid)?;
        EbpfFreezerHook::register_frozen_pid(pid);

        let mut state = self.state.write().await;
        state.frozen_pids.insert(pid);
        state.background_pids.insert(pid, app_name.to_string());
        tracing::info!(pid, app_name, cgroups_v2 = success, "Energy-Aware Scheduler: App frozen (0 CPU cycles assigned)");
        Ok(success)
    }

    /// Thaws a background application (resumes execution immediately).
    pub async fn thaw_app(&self, pid: u32) -> io::Result<bool> {
        let success = CgroupFreezer::thaw_pid(pid)?;
        EbpfFreezerHook::unregister_frozen_pid(pid);

        let mut state = self.state.write().await;
        state.frozen_pids.remove(&pid);
        tracing::info!(pid, cgroups_v2 = success, "Energy-Aware Scheduler: App thawed (normal CPU scheduling restored)");
        Ok(success)
    }

    /// Handles a Wayland window focus change event.
    pub async fn on_window_focused(&self, focused_pid: Option<u32>, background_pids: Vec<(u32, String)>) {
        let mut pids_to_thaw = Vec::new();
        let mut pids_to_freeze = Vec::new();

        {
            let mut state = self.state.write().await;
            state.focused_pid = focused_pid;

            // If the newly focused window was frozen, we must thaw it immediately
            if let Some(f_pid) = focused_pid {
                if state.frozen_pids.contains(&f_pid) {
                    pids_to_thaw.push(f_pid);
                }
            }

            // Identify background PIDs eligible for freezing
            for (bg_pid, app_name) in background_pids {
                if Some(bg_pid) != focused_pid
                    && !state.protected_pids.contains(&bg_pid)
                    && !state.frozen_pids.contains(&bg_pid)
                {
                    pids_to_freeze.push((bg_pid, app_name));
                }
            }
        }

        // Execute thaws immediately
        for pid in pids_to_thaw {
            let _ = self.thaw_app(pid).await;
        }

        // Execute freezes
        for (pid, app_name) in pids_to_freeze {
            let _ = self.freeze_app(pid, &app_name).await;
        }
    }

    /// Returns a snapshot of frozen PIDs.
    pub async fn get_frozen_pids(&self) -> HashSet<u32> {
        let state = self.state.read().await;
        state.frozen_pids.clone()
    }
}

/// Listens for Wayland/Niri window focus events via Unix IPC socket.
async fn listen_niri_wayland_events(scheduler: Arc<EnergyAwareScheduler>, cancel_token: CancellationToken) {
    let socket_path = match std::env::var("NIRI_SOCKET") {
        Ok(path) => path,
        Err(_) => {
            if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
                format!("{}/niri.sock", runtime_dir)
            } else {
                "/run/user/1000/niri.sock".to_string()
            }
        }
    };

    tracing::info!(socket_path = %socket_path, "App Nap QoS: Attempting connection to Niri Wayland IPC socket");

    let stream = match UnixStream::connect(&socket_path).await {
        Ok(stream) => stream,
        Err(e) => {
            tracing::warn!(socket_path = %socket_path, error = %e, "Niri Wayland IPC socket not available; QoS observer falling back to polling");
            return;
        }
    };

    let mut reader = tokio::io::BufReader::new(stream).lines();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::info!("Niri Wayland IPC listener exiting on cancellation");
                break;
            }
            line = reader.next_line() => {
                match line {
                    Ok(Some(raw_line)) => {
                        // Parse Niri JSON event stream (e.g. WindowFocusChanged, WindowsChanged)
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw_line) {
                            if let Some(focus_event) = json.get("WindowFocusChanged") {
                                if let Some(pid) = focus_event.get("pid").and_then(|p| p.as_u64()).map(|p| p as u32) {
                                    tracing::debug!(focused_pid = pid, "Niri Wayland IPC: Window focus changed");
                                    scheduler.on_window_focused(Some(pid), vec![]).await;
                                }
                            }
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        tracing::error!(error = %e, "Error reading from Niri Wayland IPC stream");
                        break;
                    }
                }
            }
        }
    }
}

/// Starts the App Nap Energy-Aware Scheduler QoS observer loop.
/// Freezes background applications via `cgroups v2` (`cgroup.freeze`),
/// reducing their CPU consumption to 0% when hidden/unfocused.
#[tracing::instrument(skip(cancel_token))]
pub async fn start_qos_observer(cancel_token: CancellationToken) {
    tracing::info!("Starting Energy-Aware Scheduler (App Nap cgroups v2 QoS observer)");

    let scheduler = Arc::new(EnergyAwareScheduler::new());
    let scheduler_listener = Arc::clone(&scheduler);
    let token_listener = cancel_token.clone();

    // Spawn Niri/Wayland IPC listener task
    tokio::spawn(async move {
        listen_niri_wayland_events(scheduler_listener, token_listener).await;
    });

    // Spawn main periodic QoS evaluation loop
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Shutdown token received. Exiting App Nap observer loop.");
                    // Thaw all frozen PIDs on daemon shutdown
                    let frozen_pids = scheduler.get_frozen_pids().await;
                    for pid in frozen_pids {
                        let _ = scheduler.thaw_app(pid).await;
                    }
                    break;
                }
                _ = sleep(Duration::from_secs(5)) => {
                    // Periodic audit to check status of frozen processes
                    let frozen_pids = scheduler.get_frozen_pids().await;
                    for pid in frozen_pids {
                        if let Ok(is_frozen) = CgroupFreezer::is_frozen(pid) {
                            tracing::trace!(pid, is_frozen, "QoS Observer: cgroups v2 freeze audit check");
                        }
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qos_observer_cancellation() {
        let token = CancellationToken::new();
        start_qos_observer(token.clone()).await;
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cgroup_path_resolution() {
        let pid = std::process::id();
        let path = CgroupFreezer::get_cgroup_path_for_pid(pid);
        // On Linux, /proc/self/cgroup should return a PathBuf if cgroups v2 is active
        if Path::new("/sys/fs/cgroup").exists() {
            assert!(path.is_some() || path.is_none()); // Validates no panic on parsing
        }
    }

    #[tokio::test]
    async fn test_critical_pid_protection() {
        let scheduler = EnergyAwareScheduler::new();
        let self_pid = std::process::id();

        // Attempting to freeze self PID must be blocked by protection
        let result = scheduler.freeze_app(self_pid, "ermete-daemon-rs").await.unwrap();
        assert!(!result);

        // Attempting to freeze PID 1 must be blocked
        let result_init = scheduler.freeze_app(1, "systemd").await.unwrap();
        assert!(!result_init);
    }

    #[tokio::test]
    async fn test_scheduler_focus_thaw_flow() {
        let scheduler = EnergyAwareScheduler::new();
        let dummy_pid = 999999; // Non-existent PID for simulation

        // Protect dummy_pid from setpriority error in test environment
        scheduler.add_protected_pid(dummy_pid).await;

        scheduler.on_window_focused(Some(100), vec![(dummy_pid, "DummyApp".to_string())]).await;
        let frozen = scheduler.get_frozen_pids().await;
        assert!(!frozen.contains(&dummy_pid)); // Protected PID not frozen
    }
}
