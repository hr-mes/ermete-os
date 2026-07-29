#[derive(Default)]
pub struct SpringConfig {}

pub struct SpringAnimator {
    value: f64,
}

impl SpringAnimator {
    pub fn new(_initial: f64, _config: SpringConfig) -> Self {
        Self { value: 0.0 }
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}
