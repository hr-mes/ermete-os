use ermete_niri_ipc::async_client as niri_client;

pub struct DockController;

impl DockController {
    pub fn focus_window(win_id: u64) {
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(async { niri_client::focus_window(win_id).await; });
            }
        });
    }

    pub fn close_window(win_id: u64) {
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(async { niri_client::close_window_by_id(win_id).await; });
            }
        });
    }

    pub fn launch_app(desktop_id: &str) {
        let desktop_id = desktop_id.to_string();
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(async {
                    let _ = niri_client::niri_action(serde_json::json!({
                        "Action": {
                            "Spawn": { "command": ["gtk-launch", desktop_id] }
                        }
                    })).await;
                });
            }
        });
    }

    pub fn switch_context(context_id: &str) {
        println!("Switching to spatial context: {}", context_id);
    }
}
