use crate::dock_config::{get_dock_config_path, load_dock_config, DockConfig};
use crate::dock_data::{NiriWindowInfo, NiriWorkspaceInfo};
use ermete_niri_ipc::async_client as niri_client;
use notify::{RecursiveMode, Watcher};

pub fn fetch_current_niri_windows() -> Vec<NiriWindowInfo> {
    std::thread::spawn(|| {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                niri_client::fetch_niri_data::<Vec<NiriWindowInfo>>("Windows", "Windows").await.unwrap_or_default()
            })
        } else {
            vec![]
        }
    }).join().unwrap_or_default()
}

pub fn fetch_current_workspaces() -> Vec<NiriWorkspaceInfo> {
    std::thread::spawn(|| {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                niri_client::fetch_niri_data::<Vec<NiriWorkspaceInfo>>("Workspaces", "Workspaces").await.unwrap_or_default()
            })
        } else {
            vec![]
        }
    }).join().unwrap_or_default()
}

pub fn fetch_current_active_workspace_id() -> Option<u64> {
    let workspaces = fetch_current_workspaces();
    if let Some(focused) = workspaces.iter().find(|w| w.is_focused) {
        return Some(focused.id);
    }
    workspaces.into_iter().find(|w| w.is_active).map(|w| w.id)
}

pub fn spawn_dock_watchers(
    sender_windows: glib::Sender<Vec<NiriWindowInfo>>,
    sender_config: glib::Sender<DockConfig>,
    sender_workspaces: glib::Sender<Vec<NiriWorkspaceInfo>>,
) {
    let _ = sender_windows.send(fetch_current_niri_windows());
    let _ = sender_config.send(load_dock_config());
    let _ = sender_workspaces.send(fetch_current_workspaces());

    let win_sender = sender_windows.clone();
    let ws_sender = sender_workspaces.clone();
    niri_client::watch_niri_event_stream(move |line| {
        if line.contains("Window") || line.contains("Workspace") {
            let _ = win_sender.send(fetch_current_niri_windows());
            let _ = ws_sender.send(fetch_current_workspaces());
        }
    });

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(_) => return,
        };
        let path = get_dock_config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
            let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
        }

        while let Ok(event) = rx.recv() {
            if let Ok(ev) = event {
                if (ev.kind.is_modify() || ev.kind.is_create())
                    && ev.paths.iter().any(|p| p.file_name() == path.file_name())
                {
                    let _ = sender_config.send(load_dock_config());
                }
            }
        }
    });
}
