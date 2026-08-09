//! Animation Manager orchestrating multi-window mass-spring-damper physics,
//! Magic Lamp (Genie) minimize/restore transitions, and Wobbly Windows drag physics.

use super::magic_lamp::{MagicLampAnimator, MagicLampConfig};
use super::solver::SpringConfig;
use super::spring::WindowSpringAnimator;
use super::wobbly::{WobblyConfig, WobblyWindowAnimator};
use crate::ipc::protocol::WindowPlacement;
use std::collections::HashMap;
use tracing::debug;

/// High-level Animation Engine maintaining spring solvers, Genie transitions,
/// and Wobbly Windows physics for active compositor windows.
#[derive(Debug)]
pub struct AnimationEngine {
    config: SpringConfig,
    animators: HashMap<u64, WindowSpringAnimator>,
    magic_lamps: HashMap<u64, MagicLampAnimator>,
    wobbly_windows: HashMap<u64, WobblyWindowAnimator>,
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
            magic_lamps: HashMap::new(),
            wobbly_windows: HashMap::new(),
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
                .insert(window_id, WindowSpringAnimator::new(target.clone(), self.config));
        }

        if let Some(wobbly) = self.wobbly_windows.get_mut(&window_id) {
            wobbly.set_target(target);
        }
    }

    /// Bulk update target placements for multiple windows.
    pub fn update_targets(&mut self, placements: &[WindowPlacement]) {
        for p in placements {
            self.update_window_target(p.clone());
        }
    }

    /// Starts Magic Lamp (Genie) minimize animation for a window toward dock target placement.
    pub fn start_magic_lamp_minimize(
        &mut self,
        window_id: u64,
        source: WindowPlacement,
        target: WindowPlacement,
    ) {
        let mut animator = MagicLampAnimator::new(window_id, source, target, MagicLampConfig::default());
        animator.minimize();
        self.magic_lamps.insert(window_id, animator);
    }

    /// Starts Magic Lamp (Genie) restore animation for a window.
    pub fn start_magic_lamp_restore(&mut self, window_id: u64) {
        if let Some(animator) = self.magic_lamps.get_mut(&window_id) {
            animator.restore();
        }
    }

    /// Obtains reference to active Magic Lamp animator for a window if present.
    pub fn magic_lamp(&self, window_id: u64) -> Option<&MagicLampAnimator> {
        self.magic_lamps.get(&window_id)
    }

    /// Initiates Wobbly Windows physics effect on interactive window dragging.
    pub fn start_wobbly_drag(
        &mut self,
        window_id: u64,
        placement: WindowPlacement,
        cursor_x: f64,
        cursor_y: f64,
    ) {
        let mut wobbly = WobblyWindowAnimator::new(placement, WobblyConfig::default());
        wobbly.start_drag(cursor_x, cursor_y);
        self.wobbly_windows.insert(window_id, wobbly);
    }

    /// Updates drag cursor position for active Wobbly Windows physics simulation.
    pub fn move_wobbly_drag(
        &mut self,
        window_id: u64,
        new_placement: WindowPlacement,
        cursor_x: f64,
        cursor_y: f64,
    ) {
        if let Some(wobbly) = self.wobbly_windows.get_mut(&window_id) {
            wobbly.move_drag(new_placement, cursor_x, cursor_y);
        }
    }

    /// Concludes drag interaction for Wobbly Windows effect, letting physics settle.
    pub fn end_wobbly_drag(&mut self, window_id: u64) {
        if let Some(wobbly) = self.wobbly_windows.get_mut(&window_id) {
            wobbly.end_drag();
        }
    }

    /// Obtains reference to active Wobbly Window animator for a window if present.
    pub fn wobbly_window(&self, window_id: u64) -> Option<&WobblyWindowAnimator> {
        self.wobbly_windows.get(&window_id)
    }

    /// Removes animation tracking when a window is closed.
    pub fn remove_window(&mut self, window_id: u64) {
        self.animators.remove(&window_id);
        self.magic_lamps.remove(&window_id);
        self.wobbly_windows.remove(&window_id);
    }

    /// Advances physics simulation for all active window springs, Magic Lamp transitions,
    /// and Wobbly Windows drag meshes by `dt` seconds (on the 60Hz tick).
    /// Returns `true` if any animator was actively in motion during this tick.
    pub fn tick(&mut self, dt: f64) -> bool {
        let mut active = false;

        // 1. Tick standard spring animators
        for animator in self.animators.values_mut() {
            if !animator.is_settled() {
                animator.update(dt);
                active = true;
            }
        }

        // 2. Tick Magic Lamp (Genie) animators
        for genie in self.magic_lamps.values_mut() {
            if genie.is_animating() {
                genie.update(dt);
                active = true;
            }
        }

        // 3. Tick Wobbly Windows physics animators
        for wobbly in self.wobbly_windows.values_mut() {
            if !wobbly.is_settled() {
                wobbly.update(dt);
                active = true;
            }
        }

        if active {
            debug!("Animation engine frame tick applied: dt={:.4}s", dt);
        }
        active
    }

    /// Obtains current interpolated placement for a window, evaluating active Genie effects,
    /// Wobbly mesh deformations, or standard spring interpolations.
    pub fn current_placement(&self, target: &WindowPlacement) -> WindowPlacement {
        let wid = target.window_id;

        // Priority 1: Magic Lamp (Genie) animation taking place
        if let Some(genie) = self.magic_lamps.get(&wid) {
            if genie.is_animating() {
                return genie.current_placement();
            }
        }

        // Priority 2: Wobbly Windows drag physics active or settling
        if let Some(wobbly) = self.wobbly_windows.get(&wid) {
            if !wobbly.is_settled() {
                return wobbly.current_placement();
            }
        }

        // Priority 3: Standard spring animator
        if let Some(animator) = self.animators.get(&wid) {
            animator.current_placement()
        } else {
            target.clone()
        }
    }

    /// Returns active Magic Lamp slice transformation matrices for a window if animating.
    pub fn get_magic_lamp_matrices(&self, window_id: u64) -> Option<Vec<[f32; 16]>> {
        self.magic_lamps
            .get(&window_id)
            .filter(|g| g.is_animating())
            .map(|g| g.compute_transform_matrices())
    }

    /// Returns active Wobbly Windows quad transformation matrices for a window if active.
    pub fn get_wobbly_matrices(&self, window_id: u64) -> Option<Vec<[f32; 16]>> {
        self.wobbly_windows
            .get(&window_id)
            .filter(|w| !w.is_settled())
            .map(|w| w.compute_quad_transforms())
    }

    /// Computes overall 4x4 transformation matrix for a window.
    pub fn get_window_transform_matrix(&self, window_id: u64, target: &WindowPlacement) -> [f32; 16] {
        if let Some(genie) = self.magic_lamps.get(&window_id) {
            if genie.is_animating() {
                return genie.overall_transform_matrix();
            }
        }

        if let Some(wobbly) = self.wobbly_windows.get(&window_id) {
            if !wobbly.is_settled() {
                return wobbly.overall_transform_matrix();
            }
        }

        let cur = self.current_placement(target);
        let tx = cur.x as f32;
        let ty = cur.y as f32;
        let sx = cur.width as f32 / target.width.max(1) as f32;
        let sy = cur.height as f32 / target.height.max(1) as f32;

        super::solver::Matrix4::affine_2d(tx, ty, sx, sy, 0.0, 0.0, 0.0)
    }

    /// Returns `true` if any window in the compositor is currently animating or wobbling.
    pub fn is_animating(&self) -> bool {
        self.animators.values().any(|a| !a.is_settled())
            || self.magic_lamps.values().any(|g| g.is_animating())
            || self.wobbly_windows.values().any(|w| !w.is_settled())
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

    #[test]
    fn test_animation_engine_magic_lamp_and_wobbly_integration() {
        let mut engine = AnimationEngine::default();
        let src = WindowPlacement {
            window_id: 42,
            x: 100,
            y: 100,
            width: 800,
            height: 600,
            workspace: 1,
        };
        let tgt = WindowPlacement {
            window_id: 42,
            x: 500,
            y: 900,
            width: 50,
            height: 50,
            workspace: 1,
        };

        engine.update_window_target(src.clone());
        engine.start_magic_lamp_minimize(42, src.clone(), tgt);
        assert!(engine.is_animating());

        let active = engine.tick(0.016);
        assert!(active);

        // Check magic lamp transformation matrices
        let genie_mats = engine.get_magic_lamp_matrices(42);
        assert!(genie_mats.is_some());
        assert!(!genie_mats.unwrap().is_empty());

        let window_mat = engine.get_window_transform_matrix(42, &src);
        assert_ne!(window_mat, [0.0; 16]);

        // Wobbly drag test
        engine.start_wobbly_drag(100, src.clone(), 200.0, 200.0);
        assert!(engine.is_animating());

        engine.move_wobbly_drag(100, src.clone(), 250.0, 250.0);
        engine.tick(0.016);

        let wobbly_mats = engine.get_wobbly_matrices(100);
        assert!(wobbly_mats.is_some());
        assert!(!wobbly_mats.unwrap().is_empty());

        engine.end_wobbly_drag(100);
    }
}
