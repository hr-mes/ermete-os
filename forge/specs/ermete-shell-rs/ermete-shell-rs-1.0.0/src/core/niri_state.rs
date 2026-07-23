use crate::core::niri_client;

#[derive(Debug, Default, Clone)]
pub struct NiriState {
    pub active_workspace_id: Option<u64>,
    pub total_workspaces: u64,
    pub focused_window_title: Option<String>,
}

pub fn get_niri_state() -> NiriState {
    let mut state = NiriState::default();

    // Fetch workspaces natively over UNIX socket
    if let Some(resp) = niri_client::niri_request("Workspaces") {
        if let Some(workspaces) = resp.get("Ok").and_then(|ok| ok.get("Workspaces")).and_then(|w| w.as_array()) {
            state.total_workspaces = workspaces.len() as u64;
            for ws in workspaces {
                if ws.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false) {
                    state.active_workspace_id = ws.get("id").and_then(|v| v.as_u64());
                    break;
                }
            }
        }
    }

    // Fetch windows natively over UNIX socket
    if let Some(resp) = niri_client::niri_request("Windows") {
        if let Some(windows) = resp.get("Ok").and_then(|ok| ok.get("Windows")).and_then(|w| w.as_array()) {
            for win in windows {
                if win.get("is_focused").and_then(|v| v.as_bool()).unwrap_or(false) {
                    state.focused_window_title = win.get("title")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    break;
                }
            }
        }
    }

    state
}
