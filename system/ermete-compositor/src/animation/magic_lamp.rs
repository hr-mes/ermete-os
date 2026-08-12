#![allow(dead_code)]
//! Magic Lamp (Genie) window minimize and maximize animation solver.
//!
//! Reverse-engineers macOS/Deepin genie minimize/maximize transition effects.
//! Transforms window bounding geometry into horizontal curved slices that deform
//! along Bezier / S-curve trajectories toward a target dock/panel region.

use super::solver::{Matrix4, SpringConfig};
use super::spring::Spring1D;
use crate::ipc::protocol::WindowPlacement;
use tracing::debug;

/// State of the Magic Lamp animation transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicLampState {
    /// Window is in normal restored state.
    Idle,
    /// Window is actively minimizing into the dock/target.
    Minimizing,
    /// Window is fully minimized.
    Minimized,
    /// Window is actively restoring from the dock/target to normal geometry.
    Restoring,
}

/// Configuration settings for the Magic Lamp effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagicLampConfig {
    /// Spring configuration for smooth transition progress.
    pub spring_config: SpringConfig,
    /// Number of horizontal mesh slices generated for rendering deformation (default: 16).
    pub slice_count: usize,
    /// Non-linear suck-in curvature factor (default: 1.8).
    pub curvature: f64,
}

impl Default for MagicLampConfig {
    fn default() -> Self {
        Self {
            spring_config: SpringConfig::from_stiffness_and_damping_ratio(1.0, 180.0, 0.95),
            slice_count: 16,
            curvature: 1.8,
        }
    }
}

/// A horizontal cross-section slice of the deformed window mesh during Genie animation.
#[derive(Debug, Clone, PartialEq)]
pub struct GenieSlice {
    /// Normalized height offset from top (0.0) to bottom (1.0) of original window.
    pub y_ratio: f64,
    /// Absolute X position of slice center.
    pub center_x: f64,
    /// Absolute Y position of slice top.
    pub y: f64,
    /// Current width of slice.
    pub width: f64,
    /// Current height of slice.
    pub height: f64,
    /// 4x4 column-major transformation matrix mapping normalized slice [0,1]^2 to screen geometry.
    pub transform_matrix: [f32; 16],
}

impl GenieSlice {
    /// Returns 4x4 transformation matrix of the slice.
    pub fn transform_matrix(&self) -> [f32; 16] {
        self.transform_matrix
    }
}

/// Animator managing Magic Lamp (Genie) window minimize/maximize physics state.
#[derive(Debug, Clone)]
pub struct MagicLampAnimator {
    pub window_id: u64,
    pub source_rect: WindowPlacement,
    pub target_rect: WindowPlacement,
    pub progress: Spring1D,
    pub state: MagicLampState,
    pub config: MagicLampConfig,
}

impl MagicLampAnimator {
    /// Creates a new Magic Lamp animator transitioning between `source` (window geometry)
    /// and `target` (dock icon geometry).
    pub fn new(
        window_id: u64,
        source: WindowPlacement,
        target: WindowPlacement,
        config: MagicLampConfig,
    ) -> Self {
        let progress = Spring1D::new(0.0, config.spring_config);
        Self {
            window_id,
            source_rect: source,
            target_rect: target,
            progress,
            state: MagicLampState::Idle,
            config,
        }
    }

    /// Triggers window minimization toward target dock region.
    pub fn minimize(&mut self) {
        self.state = MagicLampState::Minimizing;
        self.progress.set_target(1.0);
        debug!("Window {} started Magic Lamp minimize animation", self.window_id);
    }

    /// Triggers window restoration back to original window geometry.
    pub fn restore(&mut self) {
        self.state = MagicLampState::Restoring;
        self.progress.set_target(0.0);
        debug!("Window {} started Magic Lamp restore animation", self.window_id);
    }

    /// Returns current state of animation.
    pub fn state(&self) -> MagicLampState {
        self.state
    }

    /// Returns normalized animation progress [0.0 = fully restored, 1.0 = fully minimized].
    pub fn current_progress(&self) -> f64 {
        self.progress.current().clamp(0.0, 1.0)
    }

    /// Returns `true` if the animation is currently active (minimizing or restoring).
    pub fn is_animating(&self) -> bool {
        match self.state {
            MagicLampState::Minimizing | MagicLampState::Restoring => !self.progress.is_settled(),
            MagicLampState::Idle | MagicLampState::Minimized => false,
        }
    }

    /// Advances simulation by `dt` seconds.
    pub fn update(&mut self, dt: f64) {
        if !self.is_animating() {
            return;
        }

        self.progress.update(dt);

        if self.progress.is_settled() {
            match self.state {
                MagicLampState::Minimizing => {
                    self.state = MagicLampState::Minimized;
                    debug!("Window {} Magic Lamp minimize finished", self.window_id);
                }
                MagicLampState::Restoring => {
                    self.state = MagicLampState::Idle;
                    debug!("Window {} Magic Lamp restore finished", self.window_id);
                }
                _ => {}
            }
        }
    }

    /// Computes list of horizontal slices defining the deformed Genie mesh at the current frame tick.
    pub fn compute_genie_mesh(&self) -> Vec<GenieSlice> {
        let p = self.current_progress();
        let n = self.config.slice_count.max(2);
        let mut slices = Vec::with_capacity(n);

        let src_cx = self.source_rect.x as f64 + self.source_rect.width as f64 / 2.0;
        let src_y = self.source_rect.y as f64;
        let src_w = self.source_rect.width as f64;
        let src_h = self.source_rect.height as f64;

        let tgt_cx = self.target_rect.x as f64 + self.target_rect.width as f64 / 2.0;
        let tgt_y = self.target_rect.y as f64;
        let tgt_w = self.target_rect.width as f64;
        let tgt_h = self.target_rect.height as f64;

        let slice_src_h = src_h / n as f64;

        for i in 0..n {
            let y_ratio = i as f64 / (n - 1) as f64;

            // Delayed progress for upper slices to create characteristic funnel / lamp neck
            let delay = (1.0 - y_ratio) * 0.35;
            let slice_progress = if p <= delay {
                0.0
            } else {
                ((p - delay) / (1.0 - delay)).clamp(0.0, 1.0)
            };

            // Non-linear curvature warping
            let curved_t = slice_progress.powf(self.config.curvature);

            // Interpolate X center along cubic Bezier S-curve trajectory
            let control_cx = src_cx + (tgt_cx - src_cx) * 0.2;
            let center_x = (1.0 - curved_t).powi(2) * src_cx
                + 2.0 * (1.0 - curved_t) * curved_t * control_cx
                + curved_t.powi(2) * tgt_cx;

            // Interpolate width and Y position
            let width = ((1.0 - curved_t) * src_w + curved_t * tgt_w).max(1.0);
            let target_slice_y = tgt_y + y_ratio * tgt_h;
            let orig_slice_y = src_y + i as f64 * slice_src_h;

            let slice_y = (1.0 - curved_t) * orig_slice_y + curved_t * target_slice_y;
            let slice_h = ((1.0 - curved_t) * slice_src_h + curved_t * (tgt_h / n as f64)).max(1.0);

            let slice_left = center_x - width / 2.0;
            let transform_matrix = Matrix4::affine_2d(
                slice_left as f32,
                slice_y as f32,
                width as f32,
                slice_h as f32,
                0.0,
                0.0,
                0.0,
            );

            slices.push(GenieSlice {
                y_ratio,
                center_x,
                y: slice_y,
                width,
                height: slice_h,
                transform_matrix,
            });
        }

        slices
    }

    /// Returns transformation matrices for all mesh slices in the Magic Lamp effect.
    pub fn compute_transform_matrices(&self) -> Vec<[f32; 16]> {
        self.compute_genie_mesh()
            .into_iter()
            .map(|slice| slice.transform_matrix)
            .collect()
    }

    /// Computes an overall 4x4 transformation matrix for the entire window bounding box.
    pub fn overall_transform_matrix(&self) -> [f32; 16] {
        let cur = self.current_placement();
        let src = &self.source_rect;

        let tx = cur.x as f32;
        let ty = cur.y as f32;
        let sx = cur.width as f32 / src.width.max(1) as f32;
        let sy = cur.height as f32 / src.height.max(1) as f32;

        let p = self.current_progress();
        let tilt = (p * (std::f64::consts::PI / 16.0)) as f32;

        Matrix4::affine_2d(tx, ty, sx, sy, 0.0, 0.0, tilt)
    }

    /// Returns current interpolated overall bounding placement of the window.
    pub fn current_placement(&self) -> WindowPlacement {
        let slices = self.compute_genie_mesh();
        if slices.is_empty() {
            return self.source_rect.clone();
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for slice in &slices {
            let left = slice.center_x - slice.width / 2.0;
            let right = slice.center_x + slice.width / 2.0;
            let top = slice.y;
            let bottom = slice.y + slice.height;

            if left < min_x { min_x = left; }
            if right > max_x { max_x = right; }
            if top < min_y { min_y = top; }
            if bottom > max_y { max_y = bottom; }
        }

        WindowPlacement {
            window_id: self.window_id,
            x: min_x.round() as i32,
            y: min_y.round() as i32,
            width: (max_x - min_x).max(1.0).round() as u32,
            height: (max_y - min_y).max(1.0).round() as u32,
            workspace: self.source_rect.workspace,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_lamp_minimize_lifecycle() {
        let cfg = MagicLampConfig::default();
        let src = WindowPlacement {
            window_id: 10,
            x: 100,
            y: 100,
            width: 800,
            height: 600,
            workspace: 1,
        };
        let tgt = WindowPlacement {
            window_id: 10,
            x: 900,
            y: 1000,
            width: 60,
            height: 60,
            workspace: 1,
        };

        let mut animator = MagicLampAnimator::new(10, src, tgt, cfg);
        assert_eq!(animator.state(), MagicLampState::Idle);

        animator.minimize();
        assert_eq!(animator.state(), MagicLampState::Minimizing);
        assert!(animator.is_animating());

        // Step 60 frames (1 sec)
        for _ in 0..60 {
            animator.update(0.016);
        }

        assert_eq!(animator.state(), MagicLampState::Minimized);
        assert!(!animator.is_animating());
        assert!((animator.current_progress() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_magic_lamp_restore_lifecycle() {
        let cfg = MagicLampConfig::default();
        let src = WindowPlacement {
            window_id: 10,
            x: 100,
            y: 100,
            width: 800,
            height: 600,
            workspace: 1,
        };
        let tgt = WindowPlacement {
            window_id: 10,
            x: 900,
            y: 1000,
            width: 60,
            height: 60,
            workspace: 1,
        };

        let mut animator = MagicLampAnimator::new(10, src, tgt, cfg);
        animator.minimize();
        for _ in 0..60 {
            animator.update(0.016);
        }
        assert_eq!(animator.state(), MagicLampState::Minimized);

        animator.restore();
        assert_eq!(animator.state(), MagicLampState::Restoring);
        assert!(animator.is_animating());

        for _ in 0..60 {
            animator.update(0.016);
        }

        assert_eq!(animator.state(), MagicLampState::Idle);
        assert!(!animator.is_animating());
        assert!((animator.current_progress() - 0.0).abs() < 1e-3);
    }

    #[test]
    fn test_genie_slice_mesh_bounds() {
        let cfg = MagicLampConfig::default();
        let src = WindowPlacement {
            window_id: 1,
            x: 200,
            y: 200,
            width: 1000,
            height: 800,
            workspace: 1,
        };
        let tgt = WindowPlacement {
            window_id: 1,
            x: 500,
            y: 1020,
            width: 48,
            height: 48,
            workspace: 1,
        };

        let mut animator = MagicLampAnimator::new(1, src, tgt, cfg);
        animator.minimize();
        animator.update(0.05);

        let slices = animator.compute_genie_mesh();
        assert_eq!(slices.len(), cfg.slice_count);

        for s in slices {
            assert!(s.width >= 1.0);
            assert!(s.height >= 1.0);
            assert!(s.center_x.is_finite());
            assert!(s.y.is_finite());
        }
    }
}
