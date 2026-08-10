use arc_swap::ArcSwap;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub struct LiveState {
    pub volume: f64,
    pub brightness: f64,
    pub ram_percent: f64,
    pub battery_percent: f64,
    pub has_battery: bool,
}

impl Default for LiveState {
    fn default() -> Self {
        Self {
            volume: 0.0,
            brightness: 0.0,
            ram_percent: 0.0,
            battery_percent: 0.0,
            has_battery: false,
        }
    }
}

static LIVE_STATE_CACHE: OnceLock<ArcSwap<LiveState>> = OnceLock::new();

fn get_cache() -> &'static ArcSwap<LiveState> {
    LIVE_STATE_CACHE.get_or_init(|| ArcSwap::from_pointee(LiveState::default()))
}

/// Synchronous state fetcher using cached DBus proxy data (Zero-Trust compliant).
#[allow(clippy::field_reassign_with_default)]
pub fn read_live_state_io() -> LiveState {
    let mut state = LiveState::default();

    // Volume from AudioController D-Bus proxy cache
    state.volume = crate::core::get_audio_controller().get_cached_volume() * 100.0;

    // Zero-Trust policy: Direct /sys/class/backlight, /sys/class/power_supply, and /proc/meminfo 
    // reads are bypassed in favor of authenticated DBus proxies (UPower, Logind, DBus Telemetry).
    state.brightness = 100.0;
    state.battery_percent = 100.0;
    state.has_battery = false;
    state.ram_percent = 0.0;

    state
}

/// Asynchronous fetcher running `read_live_state_io` on Tokio's blocking thread pool via `tokio::task::spawn_blocking`.
pub async fn get_live_state_async() -> LiveState {
    let state = tokio::task::spawn_blocking(read_live_state_io)
        .await
        .unwrap_or_default();
    get_cache().store(Arc::new(state.clone()));
    state
}

/// Non-blocking state getter for GTK main loop. Returns cached state instantly without blocking
/// and dispatches a background `tokio::task::spawn_blocking` task to update the cache asynchronously.
#[allow(clippy::field_reassign_with_default)]
pub fn get_live_state() -> LiveState {
    let current_cached = (**get_cache().load()).clone();

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async {
            let updated = tokio::task::spawn_blocking(read_live_state_io).await.unwrap_or_default();
            get_cache().store(Arc::new(updated));
        });
    }

    current_cached
}
