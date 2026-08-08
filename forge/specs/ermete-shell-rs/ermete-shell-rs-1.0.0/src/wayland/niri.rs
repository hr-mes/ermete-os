use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct NiriWorkspace {
    pub id: u64,
    pub idx: u64,
    pub name: Option<String>,
    pub output: String,
    pub is_active: bool,
    pub is_focused: bool,
}

pub fn spawn_niri_workspace_watcher(sender: glib::Sender<Vec<NiriWorkspace>>) {
    let s1 = sender.clone();
    glib::MainContext::default().spawn_local(async move {
        if let Some(workspaces) = ermete_niri_ipc::async_client::fetch_niri_data::<Vec<NiriWorkspace>>("Workspaces", "Workspaces").await {
            let _ = s1.send(workspaces);
        }
    });

    let s2 = sender.clone();
    ermete_niri_ipc::async_client::watch_niri_event_stream(move |line| {
        if line.contains("Workspace") {
            let s = s2.clone();
            glib::MainContext::default().spawn_local(async move {
                if let Some(workspaces) = ermete_niri_ipc::async_client::fetch_niri_data::<Vec<NiriWorkspace>>("Workspaces", "Workspaces").await {
                    let _ = s.send(workspaces);
                }
            });
        }
    });
}

#[derive(Debug, Default, Clone)]
pub struct NiriState {
    pub active_workspace_id: Option<u64>,
    pub total_workspaces: u64,
    pub focused_window_title: Option<String>,
}

pub fn get_niri_state() -> NiriState {
    NiriState::default()
}
