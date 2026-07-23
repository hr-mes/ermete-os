use std::fs;

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

pub fn get_live_state() -> LiveState {
    let mut state = LiveState::default();

    // Volume from SystemController D-Bus proxy cache
    state.volume = crate::core::system_proxies::get_global_controller().get_cached_volume() * 100.0;

    // Brightness via sysfs natively in pure Rust
    if let Ok(entries) = fs::read_dir("/sys/class/backlight") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let (Ok(cur_str), Ok(max_str)) = (
                fs::read_to_string(path.join("brightness")),
                fs::read_to_string(path.join("max_brightness")),
            ) {
                if let (Ok(cur), Ok(max)) = (cur_str.trim().parse::<f64>(), max_str.trim().parse::<f64>()) {
                    if max > 0.0 {
                        state.brightness = (cur / max) * 100.0;
                    }
                }
            }
        }
    }

    // RAM via /proc/meminfo in pure Rust
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let mut total = 0.0;
        let mut available = 0.0;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val_str) = line.split_whitespace().nth(1) {
                    total = val_str.parse::<f64>().unwrap_or(0.0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(val_str) = line.split_whitespace().nth(1) {
                    available = val_str.parse::<f64>().unwrap_or(0.0);
                }
            }
        }
        if total > 0.0 {
            state.ram_percent = ((total - available) / total) * 100.0;
        }
    }

    // Battery
    if let Ok(bat_str) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
        if let Ok(bat) = bat_str.trim().parse::<f64>() {
            state.battery_percent = bat;
            state.has_battery = true;
        }
    } else if let Ok(bat_str) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
        if let Ok(bat) = bat_str.trim().parse::<f64>() {
            state.battery_percent = bat;
            state.has_battery = true;
        }
    }

    state
}
