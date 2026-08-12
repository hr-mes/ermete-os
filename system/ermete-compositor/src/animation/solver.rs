#![allow(dead_code)]
//! Mass-Spring-Damper ODE Solver (Damped Harmonic Oscillator).
//!
//! Solves the second-order differential equation:
//! m * x''(t) + c * x'(t) + k * (x(t) - target) = 0
//!
//! Provides high-precision f64 physics interpolation for window movement and scaling.

/// Configuration parameters for a Mass-Spring-Damper system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    /// Mass (m) in kg (default: 1.0)
    pub mass: f64,
    /// Spring stiffness constant (k) in N/m (default: 200.0)
    pub stiffness: f64,
    /// Damping coefficient (c) in N s/m (default: 25.0)
    pub damping: f64,
    /// Convergence threshold for position and velocity to be considered settled
    pub precision: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 200.0,
            damping: 25.0,
            precision: 0.001,
        }
    }
}

impl SpringConfig {
    /// Constructs a spring configuration from mass, stiffness, and desired damping ratio (zeta).
    /// zeta = 1.0 -> Critically Damped (no overshoot, fastest non-oscillatory return)
    /// zeta < 1.0 -> Underdamped (springy/bouncy overshoot)
    /// zeta > 1.0 -> Overdamped (sluggish return)
    pub fn from_stiffness_and_damping_ratio(mass: f64, stiffness: f64, zeta: f64) -> Self {
        let safe_mass = mass.max(1e-6);
        let safe_stiffness = stiffness.max(1e-6);
        let damping = 2.0 * zeta * (safe_mass * safe_stiffness).sqrt();
        Self {
            mass: safe_mass,
            stiffness: safe_stiffness,
            damping,
            precision: 0.001,
        }
    }

    /// Natural frequency omega_0 = sqrt(k / m)
    pub fn natural_frequency(&self) -> f64 {
        (self.stiffness / self.mass.max(1e-6)).sqrt()
    }

    /// Damping ratio zeta = c / (2 * sqrt(m * k))
    pub fn damping_ratio(&self) -> f64 {
        let mk = self.mass * self.stiffness;
        if mk <= 0.0 {
            0.0
        } else {
            self.damping / (2.0 * mk.sqrt())
        }
    }
}

/// 4x4 Column-Major Matrix utilities for GPU render transformations using glam primitives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4;

impl Matrix4 {
    /// Returns identity matrix [f32; 16] in column-major order.
    pub fn identity() -> [f32; 16] {
        glam::Mat4::IDENTITY.to_cols_array()
    }

    /// Creates a 2D affine 4x4 matrix given scale (sx, sy), translation (tx, ty), shear (shear_x, shear_y), and rotation angle in radians.
    pub fn affine_2d(tx: f32, ty: f32, sx: f32, sy: f32, shear_x: f32, shear_y: f32, rotation_rad: f32) -> [f32; 16] {
        let translation = glam::Mat4::from_translation(glam::Vec3::new(tx, ty, 0.0));
        let rotation = glam::Mat4::from_rotation_z(rotation_rad);
        let scale_shear = glam::Mat4::from_cols(
            glam::Vec4::new(sx, -shear_x, 0.0, 0.0),
            glam::Vec4::new(shear_y, sy, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        (translation * rotation * scale_shear).to_cols_array()
    }

    /// Computes quad bilinear mapping transformation matrix for mapping unit quad [0,1]^2 to quadrilateral (p0, p1, p2, p3).
    #[allow(clippy::too_many_arguments)]
    pub fn quad_transform(
        x0: f32, y0: f32,
        x1: f32, y1: f32,
        x2: f32, y2: f32,
        x3: f32, y3: f32,
    ) -> [f32; 16] {
        let min_x = x0.min(x1).min(x2).min(x3);
        let max_x = x0.max(x1).max(x2).max(x3);
        let min_y = y0.min(y1).min(y2).min(y3);
        let max_y = y0.max(y1).max(y2).max(y3);

        let width = (max_x - min_x).max(1e-4);
        let height = (max_y - min_y).max(1e-4);

        let shear_x = ((x1 - x0) - (x2 - x3)) / width;
        let shear_y = ((y3 - y0) - (y2 - y1)) / height;

        Self::affine_2d(min_x, min_y, width, height, shear_x, shear_y, 0.0)
    }

    /// Multiplies two 4x4 column-major matrices.
    pub fn multiply(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
        let ma = glam::Mat4::from_cols_array(a);
        let mb = glam::Mat4::from_cols_array(b);
        (ma * mb).to_cols_array()
    }
}


/// 4th-Order Runge-Kutta (RK4) and analytical solvers for Mass-Spring-Damper systems.
pub struct MassSpringDamperSolver;

impl MassSpringDamperSolver {
    /// Computes instantaneous acceleration:
    /// a(x, v) = (-c * v - k * (x - target)) / m
    #[inline]
    pub fn acceleration(x: f64, v: f64, target: f64, config: &SpringConfig) -> f64 {
        let displacement = x - target;
        let mass = config.mass.max(1e-6);
        (-config.damping * v - config.stiffness * displacement) / mass
    }

    /// Advances state (x, v) by time step `dt` using 4th-Order Runge-Kutta (RK4) with sub-stepping.
    /// Returns the updated `(position, velocity)`.
    pub fn step_rk4(x: f64, v: f64, target: f64, dt: f64, config: &SpringConfig) -> (f64, f64) {
        if dt <= 0.0 {
            return (x, v);
        }

        // Sub-step large dt intervals into <= 4ms steps for numerical stability
        let max_substep = 0.004;
        let steps = (dt / max_substep).ceil() as usize;
        let sub_dt = dt / steps as f64;

        let mut cur_x = x;
        let mut cur_v = v;

        for _ in 0..steps {
            let k1_x = cur_v;
            let k1_v = Self::acceleration(cur_x, cur_v, target, config);

            let k2_x = cur_v + 0.5 * sub_dt * k1_v;
            let k2_v = Self::acceleration(cur_x + 0.5 * sub_dt * k1_x, cur_v + 0.5 * sub_dt * k1_v, target, config);

            let k3_x = cur_v + 0.5 * sub_dt * k2_v;
            let k3_v = Self::acceleration(cur_x + 0.5 * sub_dt * k2_x, cur_v + 0.5 * sub_dt * k2_v, target, config);

            let k4_x = cur_v + sub_dt * k3_v;
            let k4_v = Self::acceleration(cur_x + sub_dt * k3_x, cur_v + sub_dt * k3_v, target, config);

            cur_x += (sub_dt / 6.0) * (k1_x + 2.0 * k2_x + 2.0 * k3_x + k4_x);
            cur_v += (sub_dt / 6.0) * (k1_v + 2.0 * k2_v + 2.0 * k3_v + k4_v);
        }

        // Snap to target if position & velocity are within settlement threshold
        if (cur_x - target).abs() < config.precision && cur_v.abs() < config.precision * 10.0 {
            (target, 0.0)
        } else {
            (cur_x, cur_v)
        }
    }

    /// Evaluates exact analytical solution step for damped harmonic oscillator.
    pub fn step_analytical(x: f64, v: f64, target: f64, dt: f64, config: &SpringConfig) -> (f64, f64) {
        if dt <= 0.0 {
            return (x, v);
        }

        let m = config.mass.max(1e-6);
        let k = config.stiffness;
        let c = config.damping;
        let y0 = x - target;
        let v0 = v;

        let delta = c * c - 4.0 * m * k;
        let (new_y, new_v) = if delta.abs() < 1e-9 {
            // Critically Damped
            let w0 = (k / m).sqrt();
            let a = y0;
            let b = v0 + w0 * y0;
            let exp_term = (-w0 * dt).exp();
            let pos = (a + b * dt) * exp_term;
            let vel = (b - w0 * (a + b * dt)) * exp_term;
            (pos, vel)
        } else if delta < 0.0 {
            // Underdamped
            let w0 = (k / m).sqrt();
            let zeta = c / (2.0 * (m * k).sqrt());
            let wd = w0 * (1.0 - zeta * zeta).sqrt();
            let exp_term = (-zeta * w0 * dt).exp();
            let a = y0;
            let b = (v0 + zeta * w0 * y0) / wd;

            let cos_term = (wd * dt).cos();
            let sin_term = (wd * dt).sin();

            let pos = exp_term * (a * cos_term + b * sin_term);
            let vel = -zeta * w0 * pos + exp_term * (-a * wd * sin_term + b * wd * cos_term);
            (pos, vel)
        } else {
            // Overdamped
            let sqrt_delta = delta.sqrt();
            let r1 = (-c + sqrt_delta) / (2.0 * m);
            let r2 = (-c - sqrt_delta) / (2.0 * m);

            let c2 = (v0 - r1 * y0) / (r2 - r1);
            let c1 = y0 - c2;

            let exp1 = (r1 * dt).exp();
            let exp2 = (r2 * dt).exp();

            let pos = c1 * exp1 + c2 * exp2;
            let vel = c1 * r1 * exp1 + c2 * r2 * exp2;
            (pos, vel)
        };

        let new_x = new_y + target;
        if (new_x - target).abs() < config.precision && new_v.abs() < config.precision * 10.0 {
            (target, 0.0)
        } else {
            (new_x, new_v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_config_damping_ratio() {
        let cfg = SpringConfig::from_stiffness_and_damping_ratio(1.0, 100.0, 1.0);
        assert!((cfg.damping_ratio() - 1.0).abs() < 1e-6);
        assert!((cfg.damping - 20.0).abs() < 1e-6);

        let underdamped = SpringConfig::from_stiffness_and_damping_ratio(1.0, 100.0, 0.7);
        assert!((underdamped.damping_ratio() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_rk4_convergence_critically_damped() {
        let cfg = SpringConfig::from_stiffness_and_damping_ratio(1.0, 200.0, 1.0);
        let mut x = 0.0;
        let mut v = 0.0;
        let target = 500.0;
        let dt = 0.016; // 60 FPS

        // Simulate 60 frames (1 second)
        for _ in 0..60 {
            let (nx, nv) = MassSpringDamperSolver::step_rk4(x, v, target, dt, &cfg);
            x = nx;
            v = nv;
        }

        // Should converge close to 500.0 without exploding
        assert!((x - target).abs() < 1.0, "Expected x near target, got {}", x);
        assert!(v.abs() < 5.0, "Expected velocity to decay, got {}", v);
    }

    #[test]
    fn test_rk4_vs_analytical_agreement() {
        let cfg = SpringConfig::from_stiffness_and_damping_ratio(1.0, 150.0, 0.85);
        let x0 = 10.0;
        let v0 = 5.0;
        let target = 300.0;
        let dt = 0.05;

        let (rk4_x, rk4_v) = MassSpringDamperSolver::step_rk4(x0, v0, target, dt, &cfg);
        let (ana_x, ana_v) = MassSpringDamperSolver::step_analytical(x0, v0, target, dt, &cfg);

        // RK4 sub-stepped should match analytical closely
        assert!((rk4_x - ana_x).abs() < 0.5, "RK4 position {} vs Analytical {}", rk4_x, ana_x);
        assert!((rk4_v - ana_v).abs() < 2.0, "RK4 velocity {} vs Analytical {}", rk4_v, ana_v);
    }
}
