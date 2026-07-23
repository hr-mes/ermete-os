#[derive(Debug, Clone, Copy)]
pub struct SpringConfig {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        // Sensible defaults for UI animations
        Self {
            mass: 1.0,
            stiffness: 180.0,
            damping: 26.6,
        }
    }
}

impl SpringConfig {
    pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        Self { mass, stiffness, damping }
    }
}

#[derive(Debug, Clone)]
pub struct SpringState {
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    pub config: SpringConfig,
}

impl SpringState {
    pub fn new(initial_value: f32, config: SpringConfig) -> Self {
        Self {
            value: initial_value,
            target: initial_value,
            velocity: 0.0,
            config,
        }
    }

    /// Sets a new target value to animate towards
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Snaps the spring immediately to a specific value and stops velocity
    pub fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }

    /// Performs one physics step based on the provided delta time
    pub fn step(&mut self, dt: f32) -> f32 {
        let f_spring = -self.config.stiffness * (self.value - self.target);
        let f_damper = -self.config.damping * self.velocity;
        
        let acceleration = (f_spring + f_damper) / self.config.mass;
        
        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;
        
        self.value
    }
    
    /// Performs one physics step optimized for 144Hz displays (~6.94ms per frame)
    pub fn step_144hz(&mut self) -> f32 {
        self.step(1.0 / 144.0)
    }

    /// Checks if the spring has settled at its target
    pub fn is_at_rest(&self, threshold: f32) -> bool {
        (self.value - self.target).abs() < threshold && self.velocity.abs() < threshold
    }
}
