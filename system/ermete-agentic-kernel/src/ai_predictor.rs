#[derive(Clone, Copy)]
pub struct AiSchedParam {
    pub target_core: u32,
    pub latency_us: u32,
    pub priority: u32,
}

unsafe impl aya::Pod for AiSchedParam {}
