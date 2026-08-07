#[derive(Debug, Clone)]
pub struct SysMonState {
    pub cpu_fraction: f64,
    pub cpu_text: String,
    pub ram_fraction: f64,
    pub ram_text: String,
    pub info_text: String,
}

impl Default for SysMonState {
    fn default() -> Self {
        Self {
            cpu_fraction: 0.0,
            cpu_text: "Processore\nCarico: In attesa...".to_string(),
            ram_fraction: 0.0,
            ram_text: "Memoria RAM\nIn attesa...".to_string(),
            info_text: "Wayland / Niri Compositor — Forgia Atomica RPM".to_string(),
        }
    }
}

pub enum SysMonIntent {
    RefreshMetrics,
}

pub struct SysMonViewModel;

impl SysMonViewModel {
    pub fn get_initial_state() -> SysMonState {
        SysMonState::default()
    }

    pub fn subscribe_metrics<F: Fn(SysMonState) + 'static>(_on_metrics: F) {
        // Can poll system metrics or receive metrics stream if available
    }

    pub fn execute_intent(intent: SysMonIntent) {
        match intent {
            SysMonIntent::RefreshMetrics => {}
        }
    }
}
