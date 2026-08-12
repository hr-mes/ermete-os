#![allow(dead_code)]
//! 1D and 2D/4D Spring abstractions for window geometry interpolation.

use super::solver::{MassSpringDamperSolver, SpringConfig};
use crate::ipc::protocol::WindowPlacement;

/// A single-variable (1D) spring for continuous physics-based f64 interpolation.
#[derive(Debug, Clone)]
pub struct Spring1D {
    current: f64,
    target: f64,
    velocity: f64,
    config: SpringConfig,
}

impl Spring1D {
    pub fn new(initial_val: f64, config: SpringConfig) -> Self {
        Self {
            current: initial_val,
            target: initial_val,
            velocity: 0.0,
            config,
        }
    }

    pub fn current(&self) -> f64 {
        self.current
    }

    pub fn target(&self) -> f64 {
        self.target
    }

    pub fn velocity(&self) -> f64 {
        self.velocity
    }

    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    pub fn set_current(&mut self, val: f64) {
        self.current = val;
    }

    pub fn reset(&mut self, val: f64) {
        self.current = val;
        self.target = val;
        self.velocity = 0.0;
    }

    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < self.config.precision
            && self.velocity.abs() < self.config.precision * 10.0
    }

    pub fn update(&mut self, dt: f64) {
        if self.is_settled() {
            self.current = self.target;
            self.velocity = 0.0;
            return;
        }

        let (new_x, new_v) = MassSpringDamperSolver::step_rk4(
            self.current,
            self.velocity,
            self.target,
            dt,
            &self.config,
        );
        self.current = new_x;
        self.velocity = new_v;
    }
}

/// 4-Dimensional Spring animator tracking window bounds: (x, y, width, height).
#[derive(Debug, Clone)]
pub struct WindowSpringAnimator {
    pub window_id: u64,
    pub x: Spring1D,
    pub y: Spring1D,
    pub width: Spring1D,
    pub height: Spring1D,
    pub workspace: u32,
}

impl WindowSpringAnimator {
    pub fn new(initial: WindowPlacement, config: SpringConfig) -> Self {
        Self {
            window_id: initial.window_id,
            x: Spring1D::new(initial.x as f64, config),
            y: Spring1D::new(initial.y as f64, config),
            width: Spring1D::new(initial.width as f64, config),
            height: Spring1D::new(initial.height as f64, config),
            workspace: initial.workspace,
        }
    }

    pub fn set_target(&mut self, target: &WindowPlacement) {
        self.x.set_target(target.x as f64);
        self.y.set_target(target.y as f64);
        self.width.set_target(target.width as f64);
        self.height.set_target(target.height as f64);
        self.workspace = target.workspace;
    }

    pub fn is_settled(&self) -> bool {
        self.x.is_settled()
            && self.y.is_settled()
            && self.width.is_settled()
            && self.height.is_settled()
    }

    pub fn update(&mut self, dt: f64) {
        self.x.update(dt);
        self.y.update(dt);
        self.width.update(dt);
        self.height.update(dt);
    }

    pub fn current_placement(&self) -> WindowPlacement {
        WindowPlacement {
            window_id: self.window_id,
            x: self.x.current().round() as i32,
            y: self.y.current().round() as i32,
            width: (self.width.current().max(1.0)).round() as u32,
            height: (self.height.current().max(1.0)).round() as u32,
            workspace: self.workspace,
        }
    }

    pub fn target_placement(&self) -> WindowPlacement {
        WindowPlacement {
            window_id: self.window_id,
            x: self.x.target().round() as i32,
            y: self.y.target().round() as i32,
            width: self.width.target().round() as u32,
            height: self.height.target().round() as u32,
            workspace: self.workspace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_1d_lifecycle() {
        let cfg = SpringConfig::default();
        let mut spring = Spring1D::new(0.0, cfg);

        spring.set_target(100.0);
        assert!(!spring.is_settled());

        for _ in 0..100 {
            spring.update(0.016);
        }

        assert!(spring.is_settled());
        assert_eq!(spring.current(), 100.0);
    }

    #[test]
    fn test_window_spring_animator() {
        let cfg = SpringConfig::default();
        let initial = WindowPlacement {
            window_id: 42,
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            workspace: 1,
        };

        let mut animator = WindowSpringAnimator::new(initial.clone(), cfg);
        assert!(animator.is_settled());

        let target = WindowPlacement {
            window_id: 42,
            x: 200,
            y: 150,
            width: 960,
            height: 720,
            workspace: 1,
        };

        animator.set_target(&target);
        assert!(!animator.is_settled());

        // Step physics forward
        animator.update(0.05);

        let cur = animator.current_placement();
        assert!(cur.x > 0 && cur.x <= 200);
        assert!(cur.y > 0 && cur.y <= 150);
        assert!(cur.width >= 800 && cur.width <= 960);
        assert!(cur.height >= 600 && cur.height <= 720);
    }
}
