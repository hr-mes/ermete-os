//! Animation Manager orchestrating multi-window mass-spring-damper physics.

use super::solver::SpringConfig;
use super::spring::WindowSpringAnimator;
use crate::ipc::protocol::WindowPlacement;
use std::collections::HashMap;
use tracing::debug;

/// High-level Animation Engine maintaining spring solvers for active compositor windows.
#[derive(Debug)]
pub struct AnimationEngine {
    config: SpringConfig,
    animators: HashMap<u64, WindowSpringAnimator>,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new(SpringConfig::default())
    }
}

impl AnimationEngine {
    pub fn new(config: SpringConfig) -> Self {
        Self {
            config,
            animators: HashMap::new(),
        }
    }

    pub fn set_config(&mut self, config: SpringConfig) {
        self.config = config;
    }

    pub fn config(&self) -> &SpringConfig {
        &self.config
    }

    /// Sets or updates target geometry for a window. If window has no animator, registers a new one.
    pub fn update_window_target(&mut self, target: WindowPlacement) {
        let window_id = target.window_id;
        if let Some(animator) = self.animators.get_mut(&window_id) {
            animator.set_target(&target);
        } else {
            self.animators
                .insert(window_id, WindowSpringAnimator::new(target, self.config));
        }
    }

    /// Bulk update target placements for multiple windows.
    pub fn update_targets(&mut self, placements: &[WindowPlacement]) {
        for p in placements {
            self.update_window_target(p.clone());
        }
    }

    /// Removes animation tracking when a window is closed.
    pub fn remove_window(&mut self, window_id: u64) {
        self.animators.remove(&window_id);
    }

    /// Advances physics simulation for all active window springs by `dt` seconds.
    /// Returns `true` if any spring was actively in motion during this tick.
    pub fn tick(&mut self, dt: f64) -> bool {
        let mut active = false;
        for animator in self.animators.values_mut() {
            if !animator.is_settled() {
                animator.update(dt);
                active = true;
            }
        }
        if active {
            debug!("Animation engine frame tick applied: dt={:.4}s", dt);
        }
        active
    }

    /// Obtains current interpolated placement for a window.
    pub fn current_placement(&self, target: &WindowPlacement) -> WindowPlacement {
        if let Some(animator) = self.animators.get(&target.window_id) {
            animator.current_placement()
        } else {
            target.clone()
        }
    }

    /// Returns `true` if any window is currently animating.
    pub fn is_animating(&self) -> bool {
        self.animators.values().any(|a| !a.is_settled())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_engine_multi_window() {
        let mut engine = AnimationEngine::default();

        let w1 = WindowPlacement {
            window_id: 1,
            x: 0,
            y: 0,
            width: 500,
            height: 500,
            workspace: 1,
        };
        let w2 = WindowPlacement {
            window_id: 2,
            x: 500,
            y: 0,
            width: 500,
            height: 500,
            workspace: 1,
        };

        engine.update_targets(&[w1.clone(), w2.clone()]);
        assert!(!engine.is_animating());

        // Update target for w1
        let w1_new = WindowPlacement {
            window_id: 1,
            x: 100,
            y: 100,
            width: 600,
            height: 600,
            workspace: 1,
        };
        engine.update_window_target(w1_new.clone());
        assert!(engine.is_animating());

        // Tick simulation
        let active = engine.tick(0.016);
        assert!(active);

        let cur_w1 = engine.current_placement(&w1_new);
        assert!(cur_w1.x > 0);

        // Remove window
        engine.remove_window(1);
        assert!(!engine.is_animating());
    }
}
