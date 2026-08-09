//! Wobbly Windows physics effect simulation on window dragging.
//!
//! Deforms a window geometry into a mass-spring lattice mesh.
//! When a window is dragged or moved, inertial resistance and spring forces
//! cause the corners and edges to lag behind and oscillate naturally (elastic wobble effect).

use super::solver::{MassSpringDamperSolver, Matrix4, SpringConfig};
use crate::ipc::protocol::WindowPlacement;
use tracing::debug;

/// Grid vertex node in the 2D wobbly window mass-spring mesh.
#[derive(Debug, Clone, Copy)]
pub struct WobblyNode {
    /// Current position (x, y)
    pub x: f64,
    pub y: f64,
    /// Current velocity (vx, vy)
    pub vx: f64,
    pub vy: f64,
    /// Normalized UV grid coordinates in range [0.0, 1.0]
    pub u: f64,
    pub v: f64,
}

/// Configuration parameters for Wobbly Windows physics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WobblyConfig {
    /// Spring configuration for mesh nodes.
    pub spring_config: SpringConfig,
    /// Grid resolution along X (columns, default: 3).
    pub grid_cols: usize,
    /// Grid resolution along Y (rows, default: 3).
    pub grid_rows: usize,
    /// Maximum deformation strain displacement cap in pixels (default: 80.0).
    pub max_displacement: f64,
    /// Drag lag factor determining how much corners lag behind velocity (default: 0.12).
    pub drag_lag: f64,
}

impl Default for WobblyConfig {
    fn default() -> Self {
        Self {
            spring_config: SpringConfig {
                mass: 1.0,
                stiffness: 220.0,
                damping: 18.0, // Underdamped (zeta ~ 0.6) for bouncy elastic wobble
                precision: 0.05,
            },
            grid_cols: 3,
            grid_rows: 3,
            max_displacement: 80.0,
            drag_lag: 0.12,
        }
    }
}

/// Wobbly Windows physics engine managing interactive window dragging and inertia.
#[derive(Debug, Clone)]
pub struct WobblyWindowAnimator {
    pub window_id: u64,
    pub target_placement: WindowPlacement,
    pub nodes: Vec<WobblyNode>,
    pub is_dragging: bool,
    pub config: WobblyConfig,
    last_drag_pos: Option<(f64, f64)>,
}

impl WobblyWindowAnimator {
    /// Initializes a new Wobbly Window spring mesh for the given window placement.
    pub fn new(initial: WindowPlacement, config: WobblyConfig) -> Self {
        let cols = config.grid_cols.max(2);
        let rows = config.grid_rows.max(2);
        let mut nodes = Vec::with_capacity(cols * rows);

        for r in 0..rows {
            let v = r as f64 / (rows - 1) as f64;
            let node_y = initial.y as f64 + v * initial.height as f64;

            for c in 0..cols {
                let u = c as f64 / (cols - 1) as f64;
                let node_x = initial.x as f64 + u * initial.width as f64;

                nodes.push(WobblyNode {
                    x: node_x,
                    y: node_y,
                    vx: 0.0,
                    vy: 0.0,
                    u,
                    v,
                });
            }
        }

        Self {
            window_id: initial.window_id,
            target_placement: initial,
            nodes,
            is_dragging: false,
            config,
            last_drag_pos: None,
        }
    }

    /// Notifies animator that a window drag interaction has started at cursor `(cursor_x, cursor_y)`.
    pub fn start_drag(&mut self, cursor_x: f64, cursor_y: f64) {
        self.is_dragging = true;
        self.last_drag_pos = Some((cursor_x, cursor_y));
        debug!("Window {} started wobbly drag at ({}, {})", self.window_id, cursor_x, cursor_y);
    }

    /// Updates window target placement during cursor motion, injecting momentum into mesh nodes.
    pub fn move_drag(&mut self, new_placement: WindowPlacement, cursor_x: f64, cursor_y: f64) {
        if let Some((lx, ly)) = self.last_drag_pos {
            let dx = cursor_x - lx;
            let dy = cursor_y - ly;

            // Apply velocity impulse / drag lag proportional to distance from cursor center
            let win_cx = new_placement.x as f64 + new_placement.width as f64 / 2.0;
            let win_cy = new_placement.y as f64 + new_placement.height as f64 / 2.0;

            for node in &mut self.nodes {
                // Nodes further from drag point experience higher inertial lag
                let dist_factor = ((node.x - win_cx).powi(2) + (node.y - win_cy).powi(2)).sqrt() / 500.0;
                let lag = (dist_factor * self.config.drag_lag).min(1.0);

                node.vx -= dx * lag * 30.0;
                node.vy -= dy * lag * 30.0;
            }
        }

        self.last_drag_pos = Some((cursor_x, cursor_y));
        self.target_placement = new_placement;
    }

    /// Ends window drag interaction, allowing springs to oscillate back to equilibrium.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.last_drag_pos = None;
        debug!("Window {} ended wobbly drag", self.window_id);
    }

    /// Sets new static target placement (e.g. tiling recalculation) without explicit drag.
    pub fn set_target(&mut self, placement: WindowPlacement) {
        self.target_placement = placement;
    }

    /// Returns `true` if all nodes in the mesh have settled back to target resting positions.
    pub fn is_settled(&self) -> bool {
        if self.is_dragging {
            return false;
        }

        let placement = &self.target_placement;
        let prec = self.config.spring_config.precision;

        for node in &self.nodes {
            let target_x = placement.x as f64 + node.u * placement.width as f64;
            let target_y = placement.y as f64 + node.v * placement.height as f64;

            if (node.x - target_x).abs() > prec * 10.0
                || (node.y - target_y).abs() > prec * 10.0
                || node.vx.abs() > prec * 50.0
                || node.vy.abs() > prec * 50.0
            {
                return false;
            }
        }

        true
    }

    /// Returns 4x4 transformation matrices for each quad grid cell in the deformed wobbly mesh.
    pub fn compute_quad_transforms(&self) -> Vec<[f32; 16]> {
        let cols = self.config.grid_cols.max(2);
        let rows = self.config.grid_rows.max(2);
        let num_quads = (cols - 1) * (rows - 1);
        let mut matrices = Vec::with_capacity(num_quads);

        for r in 0..(rows - 1) {
            for c in 0..(cols - 1) {
                let idx_top_left = r * cols + c;
                let idx_top_right = r * cols + c + 1;
                let idx_bottom_right = (r + 1) * cols + c + 1;
                let idx_bottom_left = (r + 1) * cols + c;

                let p0 = &self.nodes[idx_top_left];
                let p1 = &self.nodes[idx_top_right];
                let p2 = &self.nodes[idx_bottom_right];
                let p3 = &self.nodes[idx_bottom_left];

                let matrix = Matrix4::quad_transform(
                    p0.x as f32, p0.y as f32,
                    p1.x as f32, p1.y as f32,
                    p2.x as f32, p2.y as f32,
                    p3.x as f32, p3.y as f32,
                );

                matrices.push(matrix);
            }
        }

        matrices
    }

    /// Computes overall 4x4 transformation matrix for the wobbly window bounding box.
    pub fn overall_transform_matrix(&self) -> [f32; 16] {
        let cur = self.current_placement();
        let tgt = &self.target_placement;

        let tx = cur.x as f32;
        let ty = cur.y as f32;
        let sx = cur.width as f32 / tgt.width.max(1) as f32;
        let sy = cur.height as f32 / tgt.height.max(1) as f32;

        let cols = self.config.grid_cols.max(2);
        let top_left = &self.nodes[0];
        let top_right = &self.nodes[cols - 1];

        let dx = top_right.x - top_left.x;
        let dy = top_right.y - top_left.y;
        let tilt_rad = dy.atan2(dx) as f32;

        Matrix4::affine_2d(tx, ty, sx, sy, 0.0, 0.0, tilt_rad)
    }

    /// Advances physics simulation for all wobbly mesh nodes by `dt` seconds.
    pub fn update(&mut self, dt: f64) {
        if self.is_settled() && !self.is_dragging {
            // Snap nodes precisely to rest positions
            let placement = &self.target_placement;
            for node in &mut self.nodes {
                node.x = placement.x as f64 + node.u * placement.width as f64;
                node.y = placement.y as f64 + node.v * placement.height as f64;
                node.vx = 0.0;
                node.vy = 0.0;
            }
            return;
        }

        let placement = self.target_placement.clone();
        let max_disp = self.config.max_displacement;

        // Compute inter-node mesh structural spring forces
        let cols = self.config.grid_cols.max(2);
        let rows = self.config.grid_rows.max(2);
        let mesh_k = 30.0;

        let mut forces_x = vec![0.0f64; self.nodes.len()];
        let mut forces_y = vec![0.0f64; self.nodes.len()];

        for r in 0..rows {
            for c in 0..cols {
                let idx = r * cols + c;
                let u = self.nodes[idx].u;
                let v = self.nodes[idx].v;

                if c + 1 < cols {
                    let r_idx = r * cols + c + 1;
                    let rest_dx = (self.nodes[r_idx].u - u) * placement.width as f64;
                    let cur_dx = self.nodes[r_idx].x - self.nodes[idx].x;
                    let force = (cur_dx - rest_dx) * mesh_k;
                    forces_x[idx] += force;
                    forces_x[r_idx] -= force;
                }

                if r + 1 < rows {
                    let b_idx = (r + 1) * cols + c;
                    let rest_dy = (self.nodes[b_idx].v - v) * placement.height as f64;
                    let cur_dy = self.nodes[b_idx].y - self.nodes[idx].y;
                    let force = (cur_dy - rest_dy) * mesh_k;
                    forces_y[idx] += force;
                    forces_y[b_idx] -= force;
                }
            }
        }

        for (idx, node) in self.nodes.iter_mut().enumerate() {
            let target_x = placement.x as f64 + node.u * placement.width as f64 + forces_x[idx] * 0.05;
            let target_y = placement.y as f64 + node.v * placement.height as f64 + forces_y[idx] * 0.05;

            // Solve X component spring physics
            let (nx, nvx) = MassSpringDamperSolver::step_rk4(
                node.x,
                node.vx,
                target_x,
                dt,
                &self.config.spring_config,
            );

            // Solve Y component spring physics
            let (ny, nvy) = MassSpringDamperSolver::step_rk4(
                node.y,
                node.vy,
                target_y,
                dt,
                &self.config.spring_config,
            );

            // Clamp max displacement strain to prevent extreme mesh tearing
            let base_target_x = placement.x as f64 + node.u * placement.width as f64;
            let base_target_y = placement.y as f64 + node.v * placement.height as f64;
            node.x = nx.clamp(base_target_x - max_disp, base_target_x + max_disp);
            node.y = ny.clamp(base_target_y - max_disp, base_target_y + max_disp);
            node.vx = nvx;
            node.vy = nvy;
        }
    }

    /// Returns current overall bounding placement enclosing all wobbly mesh nodes.
    pub fn current_placement(&self) -> WindowPlacement {
        if self.nodes.is_empty() {
            return self.target_placement.clone();
        }

        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for node in &self.nodes {
            if node.x < min_x { min_x = node.x; }
            if node.x > max_x { max_x = node.x; }
            if node.y < min_y { min_y = node.y; }
            if node.y > max_y { max_y = node.y; }
        }

        WindowPlacement {
            window_id: self.window_id,
            x: min_x.round() as i32,
            y: min_y.round() as i32,
            width: (max_x - min_x).max(1.0).round() as u32,
            height: (max_y - min_y).max(1.0).round() as u32,
            workspace: self.target_placement.workspace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wobbly_window_drag_and_settle() {
        let cfg = WobblyConfig::default();
        let initial = WindowPlacement {
            window_id: 5,
            x: 100,
            y: 100,
            width: 600,
            height: 400,
            workspace: 1,
        };

        let mut animator = WobblyWindowAnimator::new(initial.clone(), cfg);
        assert!(animator.is_settled());

        animator.start_drag(400.0, 300.0);
        assert!(!animator.is_settled());

        let new_target = WindowPlacement {
            window_id: 5,
            x: 250,
            y: 200,
            width: 600,
            height: 400,
            workspace: 1,
        };

        animator.move_drag(new_target, 550.0, 400.0);

        // Step physics
        animator.update(0.016);

        let cur = animator.current_placement();
        assert!(cur.width > 0);
        assert!(cur.height > 0);

        animator.end_drag();

        // Simulate release & return to equilibrium
        for _ in 0..120 {
            animator.update(0.016);
        }

        assert!(animator.is_settled());
        let final_placement = animator.current_placement();
        assert_eq!(final_placement.x, 250);
        assert_eq!(final_placement.y, 200);
    }
}
