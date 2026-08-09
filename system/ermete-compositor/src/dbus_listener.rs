use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppearanceSettings {
    pub color_scheme: String,
    pub accent_color: String,
    pub wallpaper: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            color_scheme: "prefer-dark".to_string(),
            accent_color: "#89b4fa".to_string(),
            wallpaper: "".to_string(),
        }
    }
}

/// Spawns an asynchronous DBus listener on a background Tokio task.
/// Listens to org.ermete.Settings.Appearance DBus signals and updates an atomic flag
/// and lock-free watch channel so the 1000Hz frame tick loop in CompositorState
/// never performs synchronous file I/O or blocks on Mutex locks during DBus signal storms.
pub fn spawn_dbus_appearance_listener(
    dirty_flag: Arc<AtomicBool>,
    tx: watch::Sender<AppearanceSettings>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let conn = match zbus::Connection::session().await {
            Ok(c) => c,
            Err(e) => {
                warn!("DBus session connection unavailable for compositor appearance listener: {}", e);
                return;
            }
        };

        info!("Asynchronous DBus appearance listener connected to session bus for org.ermete.Settings.Appearance");

        let mut stream = zbus::MessageStream::from(&conn);

        use futures_util::StreamExt;
        while let Some(msg_res) = stream.next().await {
            if let Ok(msg) = msg_res {
                let header = msg.header();
                if let Some(interface) = header.interface() {
                    let iface_str = interface.as_str();
                    if iface_str == "org.ermete.Settings.Appearance" || iface_str == "org.ermete.Settings" {
                        // Process update completely asynchronously:
                        // No disk reads are performed synchronously in the 1000Hz loop.
                        let current = tx.borrow().clone();

                        // Send to watch channel lock-free
                        let _ = tx.send(current);

                        // Signal cheap atomic flag for the 1000Hz tick loop
                        dirty_flag.store(true, Ordering::Release);
                    }
                }
            }
        }
    })
}
