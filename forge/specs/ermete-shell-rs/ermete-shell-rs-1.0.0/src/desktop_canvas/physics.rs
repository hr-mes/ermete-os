use std::f64::consts::FRAC_PI_2;

/// 2D Spring physics solver for smooth spring-damper animation transitions
#[derive(Debug, Clone, Copy)]
pub struct Spring2D {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl Spring2D {
    pub fn new(x: f64, y: f64, stiffness: f64, damping: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            target_x: x,
            target_y: y,
            stiffness,
            damping,
            mass: 1.0,
        }
    }

    pub fn set_target(&mut self, target_x: f64, target_y: f64) {
        self.target_x = target_x;
        self.target_y = target_y;
    }

    /// Step the spring physics by delta-time `dt` (in seconds).
    /// Returns `true` if the spring is still in motion, or `false` when settled.
    pub fn update(&mut self, dt: f64) -> bool {
        let fx = -self.stiffness * (self.x - self.target_x) - self.damping * self.vx;
        let fy = -self.stiffness * (self.y - self.target_y) - self.damping * self.vy;

        let ax = fx / self.mass;
        let ay = fy / self.mass;

        self.vx += ax * dt;
        self.vy += ay * dt;

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        let dist_sq = (self.x - self.target_x).powi(2) + (self.y - self.target_y).powi(2);
        let speed_sq = self.vx.powi(2) + self.vy.powi(2);

        // Threshold for settling
        !(dist_sq < 0.01 && speed_sq < 0.01)
    }
}

/// Fan-out layout configuration algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanLayout {
    Arc,
    Grid,
    Horizontal,
}

pub fn calculate_fan_out_positions(
    center_x: f64,
    center_y: f64,
    count: usize,
    radius: f64,
    layout: FanLayout,
) -> Vec<(f64, f64)> {
    let mut positions = Vec::with_capacity(count);
    if count == 0 {
        return positions;
    }

    match layout {
        FanLayout::Arc => {
            let spread_angle = 1.6; // radians
            let start_angle = -FRAC_PI_2 - (spread_angle / 2.0);
            let step = if count > 1 {
                spread_angle / (count - 1) as f64
            } else {
                0.0
            };
            for i in 0..count {
                let angle = start_angle + i as f64 * step;
                let x = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();
                positions.push((x, y));
            }
        }
        FanLayout::Grid => {
            let cols = (count as f64).sqrt().ceil() as usize;
            let card_w = 120.0;
            let card_h = 90.0;
            let spacing = 16.0;
            for i in 0..count {
                let col = i % cols;
                let row = i / cols;
                let x = center_x + 140.0 + (col as f64 * (card_w + spacing));
                let y = center_y - 40.0 + (row as f64 * (card_h + spacing));
                positions.push((x, y));
            }
        }
        FanLayout::Horizontal => {
            let spacing = 130.0;
            for i in 0..count {
                let x = center_x + (i as f64 + 1.0) * spacing;
                let y = center_y;
                positions.push((x, y));
            }
        }
    }

    positions
}
