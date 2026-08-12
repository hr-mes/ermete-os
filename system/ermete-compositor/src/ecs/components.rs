#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::fmt;

/// 2D Position component (pixels or layout coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub const ZERO: Position = Position { x: 0.0, y: 0.0 };

    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    #[inline]
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 2D Velocity vector (pixels per second).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

impl Velocity {
    pub const ZERO: Velocity = Velocity { vx: 0.0, vy: 0.0 };

    #[inline]
    pub fn new(vx: f32, vy: f32) -> Self {
        Self { vx, vy }
    }

    #[inline]
    pub fn speed(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }

    #[inline]
    pub fn is_moving(&self, threshold: f32) -> bool {
        self.speed() > threshold
    }

    #[inline]
    pub fn stop(&mut self) {
        self.vx = 0.0;
        self.vy = 0.0;
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Target 2D Position component for 1000Hz spring physics animation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TargetPosition {
    pub x: f32,
    pub y: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl TargetPosition {
    pub const ZERO: TargetPosition = TargetPosition {
        x: 0.0,
        y: 0.0,
        stiffness: 300.0,
        damping: 30.0,
        mass: 1.0,
    };

    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            stiffness: 300.0,
            damping: 30.0,
            mass: 1.0,
        }
    }

    #[inline]
    pub fn with_spring(x: f32, y: f32, stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            x,
            y,
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
            mass: mass.max(0.001),
        }
    }

    #[inline]
    pub fn set_target(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

impl Default for TargetPosition {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Window / Surface Geometry component (width, height, HiDPI scaling, rounded corners).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Geometry {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub corner_radius: f32,
    pub aspect_ratio: Option<(u32, u32)>,
}

impl Geometry {
    #[inline]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
            corner_radius: 0.0,
            aspect_ratio: None,
        }
    }

    #[inline]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale_factor = scale.max(0.1);
        self
    }

    #[inline]
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius.max(0.0);
        self
    }

    #[inline]
    pub fn scaled_width(&self) -> f32 {
        self.width as f32 * self.scale_factor
    }

    #[inline]
    pub fn scaled_height(&self) -> f32 {
        self.height as f32 * self.scale_factor
    }

    #[inline]
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    #[inline]
    pub fn contains_point(&self, pos: &Position, px: f32, py: f32) -> bool {
        px >= pos.x
            && px <= pos.x + self.scaled_width()
            && py >= pos.y
            && py <= pos.y + self.scaled_height()
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

/// DMA-BUF / Wayland Surface handle component.
/// Contains surface metadata, DMA-BUF file descriptor, FourCC format codes,
/// damage state, and process details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaylandSurface {
    pub surface_id: u64,
    pub client_pid: u32,
    pub dma_buf_fd: Option<i32>,
    pub buffer_format: u32, // DRM FourCC format (e.g., DRM_FORMAT_ARGB8888)
    pub modifier: u64,      // DRM format modifier (e.g. DRM_FORMAT_MOD_LINEAR)
    pub stride: u32,
    pub width: u32,
    pub height: u32,
    pub is_damaged: bool,
    pub wl_title: String,
    pub app_id: String,
    pub is_active: bool,
}

impl WaylandSurface {
    pub fn new(
        surface_id: u64,
        client_pid: u32,
        wl_title: impl Into<String>,
        app_id: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            surface_id,
            client_pid,
            dma_buf_fd: None,
            buffer_format: 0x34325241, // DRM_FORMAT_ARGB8888 (0x34325241)
            modifier: 0,               // DRM_FORMAT_MOD_LINEAR
            stride: width * 4,
            width,
            height,
            is_damaged: true,
            wl_title: wl_title.into(),
            app_id: app_id.into(),
            is_active: true,
        }
    }

    /// Attaches or imports a DMA buffer handle.
    pub fn attach_dma_buffer(&mut self, fd: i32, format: u32, modifier: u64, stride: u32) {
        self.dma_buf_fd = Some(fd);
        self.buffer_format = format;
        self.modifier = modifier;
        self.stride = stride;
        self.is_damaged = true;
    }

    #[inline]
    pub fn mark_damaged(&mut self) {
        self.is_damaged = true;
    }

    #[inline]
    pub fn clear_damage(&mut self) {
        self.is_damaged = false;
    }
}

/// Wayland Compositor Layer hierarchy enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LayerType {
    Background = 0,
    Bottom = 10,
    Normal = 20,
    Top = 30,
    Overlay = 40,
    Cursor = 50,
}

/// RenderLayer component defining depth ordering, opacity, visibility, and Kawase blur settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderLayer {
    pub layer_type: LayerType,
    pub z_index: i32,
    pub opacity: f32, // 0.0 (transparent) to 1.0 (opaque)
    pub visible: bool,
    pub blur_enabled: bool,
}

impl RenderLayer {
    pub fn new(layer_type: LayerType, z_index: i32) -> Self {
        Self {
            layer_type,
            z_index,
            opacity: 1.0,
            visible: true,
            blur_enabled: false,
        }
    }

    #[inline]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    #[inline]
    pub fn with_blur(mut self, blur: bool) -> Self {
        self.blur_enabled = blur;
        self
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible && self.opacity > 0.001
    }

    /// Combined composite key for sorting render lists (layer_type first, then z_index).
    #[inline]
    pub fn composite_depth_key(&self) -> (i32, i32) {
        (self.layer_type as i32, self.z_index)
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self::new(LayerType::Normal, 0)
    }
}

/// Mass-Spring-Damper physics component for smooth 1000Hz AI auto-tiling animations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhysicsSpring {
    pub stiffness: f32, // Spring constant (k)
    pub damping: f32,   // Damping coefficient (c)
    pub mass: f32,      // Mass (m)
    pub target_x: f32,
    pub target_y: f32,
}

impl PhysicsSpring {
    pub fn new(target_x: f32, target_y: f32) -> Self {
        Self {
            stiffness: 300.0,
            damping: 30.0,
            mass: 1.0,
            target_x,
            target_y,
        }
    }

    #[inline]
    pub fn set_target(&mut self, tx: f32, ty: f32) {
        self.target_x = tx;
        self.target_y = ty;
    }
}

impl Default for PhysicsSpring {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl fmt::Display for WaylandSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WaylandSurface(id={}, app='{}', pid={}, dma={:?}, damaged={})",
            self.surface_id, self.app_id, self.client_pid, self.dma_buf_fd, self.is_damaged
        )
    }
}
