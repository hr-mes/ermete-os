#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MprisState {
    pub title: String,
    pub artist: String,
    pub status: String,
}

pub fn get_mpris_state() -> Option<MprisState> {
    crate::core::system_proxies::get_global_controller().get_cached_mpris_state()
}
