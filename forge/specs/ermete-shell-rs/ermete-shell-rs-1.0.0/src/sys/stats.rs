use chrono::Local;

pub fn macos_clock_string() -> String {
    Local::now().format("%a %d %b %H:%M").to_string()
}
