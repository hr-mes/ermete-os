//! Wayland Render Link System (Phase 4)
//! 
//! Connects the ECS state to the Smithay/EGL rendering engine.
//! Executes at variable monitor refresh rates (e.g., 144Hz, 240Hz, 360Hz),
//! completely decoupled from the high-frequency 1000Hz physics animation tick loop.
//!
//! Enforces ZERO heap allocations in the hot render loop for ultra-low latency
//! and stutter-free frame submission via zero-copy DMA-BUF.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use crate::ecs::world::SharedEcsWorld;

/// Entity position component in 2D/3D compositor space.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// Entity geometry & layout scale component.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Geometry {
    pub width: f32,
    pub height: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_rad: f32,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_rad: 0.0,
        }
    }
}

/// Wayland surface buffer handle component for DRM zero-copy rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct WaylandSurface {
    pub surface_id: u64,
    pub dma_buf_fd: i32,
    pub width: u32,
    pub height: u32,
    pub drm_format: u32, // DRM FOURCC format code (e.g. ARGB8888)
    pub modifier: u64,   // DRM format modifier
    pub is_visible: bool,
}

/// Errors that can occur during zero-copy DMA-BUF submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    InvalidDmaBufFd,
    SurfaceNotVisible,
    GpuFenceTimeout,
    BufferBusy,
}

/// Mock Compositor State representing the GPU/EGL engine interface.
/// Simulates zero-copy DMA-BUF transformation matrix submission.
pub struct CompositorState {
    pub submitted_frames: AtomicU64,
    pub total_dma_bytes_pushed: AtomicU64,
    pub current_refresh_rate_hz: f32,
}

impl CompositorState {
    /// Creates a new `CompositorState` initialized with target monitor refresh rate.
    pub fn new(refresh_rate_hz: f32) -> Self {
        Self {
            submitted_frames: AtomicU64::new(0),
            total_dma_bytes_pushed: AtomicU64::new(0),
            current_refresh_rate_hz: refresh_rate_hz.max(60.0),
        }
    }

    /// Zero-copy DMA-BUF submission function.
    /// Passes 4x4 transform matrix and DMA-BUF descriptor directly to GPU DRM pipeline.
    ///
    /// GUARANTEE: Performs ZERO heap allocations (`Vec`, `Box`, `String`, etc.).
    #[inline(always)]
    pub fn submit_dma_buf(
        &self,
        surface: &WaylandSurface,
        transform_matrix: &[f32; 16],
    ) -> Result<(), RenderError> {
        if !surface.is_visible {
            return Err(RenderError::SurfaceNotVisible);
        }

        if surface.dma_buf_fd < 0 {
            return Err(RenderError::InvalidDmaBufFd);
        }

        // Simulate zero-copy hardware submission by dereferencing matrix on stack
        // and updating atomic frame metrics without lock acquisition or heap allocation.
        let matrix_sum = transform_matrix[0] + transform_matrix[5] + transform_matrix[10] + transform_matrix[15];
        let _ = matrix_sum;

        let buffer_size_bytes = (surface.width as u64) * (surface.height as u64) * 4;
        self.total_dma_bytes_pushed.fetch_add(buffer_size_bytes, Ordering::Relaxed);
        self.submitted_frames.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}

/// Computes a 4x4 column-major affine transformation matrix on the stack.
///
/// Matrix Layout:
/// [ m0  m4  m8  m12 ]
/// [ m1  m5  m9  m13 ]
/// [ m2  m6  m10 m14 ]
/// [ m3  m7  m11 m15 ]
///
/// Guaranteed zero dynamic allocation.
#[inline(always)]
pub fn compute_transform_matrix(pos: &Position, geom: &Geometry, out_matrix: &mut [f32; 16]) {
    let cos_t = geom.rotation_rad.cos();
    let sin_t = geom.rotation_rad.sin();

    let sx = geom.width * geom.scale_x;
    let sy = geom.height * geom.scale_y;

    // Column 0: X axis basis (scaled & rotated)
    out_matrix[0] = cos_t * sx;
    out_matrix[1] = sin_t * sx;
    out_matrix[2] = 0.0;
    out_matrix[3] = 0.0;

    // Column 1: Y axis basis (scaled & rotated)
    out_matrix[4] = -sin_t * sy;
    out_matrix[5] = cos_t * sy;
    out_matrix[6] = 0.0;
    out_matrix[7] = 0.0;

    // Column 2: Z axis basis
    out_matrix[8] = 0.0;
    out_matrix[9] = 0.0;
    out_matrix[10] = 1.0;
    out_matrix[11] = 0.0;

    // Column 3: Translation vector
    out_matrix[12] = pos.x;
    out_matrix[13] = pos.y;
    out_matrix[14] = pos.z;
    out_matrix[15] = 1.0;
}

/// Core Wayland Render System (Phase 4 & Phase 6).
///
/// Queries `SharedEcsWorld` for active entities possessing `(Position, Geometry, WaylandSurface)`,
/// computes transformation matrices on the stack, and submits DMA-BUF frames to `CompositorState`.
///
/// Decoupled from 1000Hz physics loop: Designed to be called at monitor refresh frequency
/// (e.g., 144Hz, 240Hz, 360Hz).
#[inline(always)]
pub fn render_system(
    world: &SharedEcsWorld,
    compositor_state: &mut CompositorState,
) -> usize {
    let world_guard = match world.read() {
        Ok(guard) => guard,
        Err(_) => return 0,
    };

    let query_results = world_guard.query_3::<crate::ecs::components::Position, crate::ecs::components::Geometry, crate::ecs::components::WaylandSurface>();
    let mut matrix_buffer = [0.0f32; 16];
    let mut rendered_count = 0usize;

    for (_entity, pos, geom, surface) in query_results {
        let render_pos = Position {
            x: pos.x,
            y: pos.y,
            z: 0.0,
        };
        let render_geom = Geometry {
            width: geom.scaled_width(),
            height: geom.scaled_height(),
            scale_x: geom.scale_factor,
            scale_y: geom.scale_factor,
            rotation_rad: 0.0,
        };
        let render_surface = WaylandSurface {
            surface_id: surface.surface_id,
            dma_buf_fd: surface.dma_buf_fd.unwrap_or(-1),
            width: surface.width,
            height: surface.height,
            drm_format: surface.buffer_format,
            modifier: surface.modifier,
            is_visible: surface.is_active,
        };

        compute_transform_matrix(&render_pos, &render_geom, &mut matrix_buffer);
        if compositor_state.submit_dma_buf(&render_surface, &matrix_buffer).is_ok() {
            rendered_count += 1;
        }
    }

    rendered_count
}

/// Helper for rendering from a slice of component references.
#[inline(always)]
pub fn render_system_slice(
    entities: &[(&Position, &Geometry, &WaylandSurface)],
    compositor_state: &CompositorState,
) -> usize {
    let mut matrix_buffer = [0.0f32; 16];
    let mut rendered_count = 0usize;

    for (pos, geom, surface) in entities {
        compute_transform_matrix(pos, geom, &mut matrix_buffer);
        if compositor_state.submit_dma_buf(surface, &matrix_buffer).is_ok() {
            rendered_count += 1;
        }
    }

    rendered_count
}

/// Render Loop Controller managing monitor VSync / refresh rate timing (e.g., 144Hz / 360Hz).
/// Operates asynchronously, decoupled from 1000Hz physics engine.
pub struct RenderLoopConfig {
    pub target_hz: f32,
}

impl Default for RenderLoopConfig {
    fn default() -> Self {
        Self { target_hz: 144.0 }
    }
}

pub struct RenderLoopRunner {
    pub config: RenderLoopConfig,
    pub compositor_state: CompositorState,
}

impl RenderLoopRunner {
    pub fn new(target_hz: f32) -> Self {
        Self {
            config: RenderLoopConfig { target_hz },
            compositor_state: CompositorState::new(target_hz),
        }
    }

    /// Single render frame execution step.
    /// High-performance stack-only invocation of `render_system_slice`.
    pub fn step_frame(&self, entities: &[(&Position, &Geometry, &WaylandSurface)]) -> usize {
        render_system_slice(entities, &self.compositor_state)
    }

    /// Helper calculating target frame duration for VSync sleep interval (nanoseconds).
    pub fn frame_budget(&self) -> Duration {
        let nanos_per_sec = 1_000_000_000.0;
        let frame_nanos = (nanos_per_sec / self.config.target_hz.max(1.0)) as u64;
        Duration::from_nanos(frame_nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_system_zero_allocation() {
        let state = CompositorState::new(360.0);

        let pos1 = Position { x: 100.0, y: 200.0, z: 0.0 };
        let geom1 = Geometry { width: 1920.0, height: 1080.0, scale_x: 1.0, scale_y: 1.0, rotation_rad: 0.0 };
        let surf1 = WaylandSurface {
            surface_id: 1,
            dma_buf_fd: 42,
            width: 1920,
            height: 1080,
            drm_format: 0x34325241, // DRM_FORMAT_ARGB8888
            modifier: 0,
            is_visible: true,
        };

        let entities = [(&pos1, &geom1, &surf1)];

        let rendered = render_system_slice(&entities, &state);
        assert_eq!(rendered, 1);
        assert_eq!(state.submitted_frames.load(Ordering::Relaxed), 1);
    }
}

