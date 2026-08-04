use ermete_niri_ipc as niri_client;

pub struct DockController;

impl DockController {
    pub fn focus_window(win_id: u64) {
        niri_client::focus_window(win_id);
    }

    pub fn close_window(win_id: u64) {
        niri_client::close_window_by_id(win_id);
    }

    pub fn launch_app(desktop_id: &str) {
        let _ = niri_client::niri_action(serde_json::json!({
            "Action": {
                "Spawn": { "command": ["gtk-launch", desktop_id] }
            }
        }));
    }

    pub fn switch_context(context_id: &str) {
        // Implement spatial context switching here
        println!("Switching to spatial context: {}", context_id);
    }
}
