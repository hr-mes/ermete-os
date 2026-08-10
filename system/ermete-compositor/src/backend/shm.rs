use thiserror::Error;
use tracing::{info, warn};

/// Default maximum allowable size for a single Wayland `wl_shm` buffer (512 MB).
pub const DEFAULT_MAX_WL_SHM_BUFFER_SIZE_BYTES: usize = 512 * 1024 * 1024;

/// Default maximum allowable size for a `wl_shm` pool mapping (512 MB).
pub const DEFAULT_MAX_WL_SHM_POOL_SIZE_BYTES: usize = 512 * 1024 * 1024;

/// Default maximum dimension (width or height) in pixels (16,384px).
pub const DEFAULT_MAX_WL_SHM_DIMENSION: i32 = 16384;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ShmError {
    #[error("Buffer size {requested} bytes exceeds maximum allowed limit of {limit} bytes")]
    BufferTooLarge { requested: usize, limit: usize },

    #[error("Pool size {requested} bytes exceeds maximum allowed limit of {limit} bytes")]
    PoolTooLarge { requested: usize, limit: usize },

    #[error("Invalid buffer dimensions: width={width}, height={height}, max={max}")]
    InvalidDimensions { width: i32, height: i32, max: i32 },

    #[error("Invalid stride: {stride} (expected positive stride >= bytes per row)")]
    InvalidStride { stride: i32 },

    #[error("Integer overflow detected when calculating buffer dimensions ({width}x{height}, stride {stride})")]
    DimensionOverflow { width: i32, height: i32, stride: i32 },
}

/// Resource limit configuration for Wayland Shared Memory (wl_shm) buffers.
#[derive(Debug, Clone)]
pub struct ShmBufferLimits {
    pub max_buffer_bytes: usize,
    pub max_pool_bytes: usize,
    pub max_dimension: i32,
}

impl Default for ShmBufferLimits {
    fn default() -> Self {
        Self {
            max_buffer_bytes: DEFAULT_MAX_WL_SHM_BUFFER_SIZE_BYTES,
            max_pool_bytes: DEFAULT_MAX_WL_SHM_POOL_SIZE_BYTES,
            max_dimension: DEFAULT_MAX_WL_SHM_DIMENSION,
        }
    }
}

/// Guard preventing Wayland clients from executing `wl_shm` Poisoning / OOM Bomb attacks.
#[derive(Debug, Clone)]
pub struct WlShmGuard {
    limits: ShmBufferLimits,
}

impl WlShmGuard {
    pub fn new(limits: ShmBufferLimits) -> Self {
        info!(
            "Initialized wl_shm Poisoning Guard: max_buffer_size={}MB, max_pool_size={}MB, max_dim={}px",
            limits.max_buffer_bytes / (1024 * 1024),
            limits.max_pool_bytes / (1024 * 1024),
            limits.max_dimension
        );
        Self { limits }
    }

    pub fn limits(&self) -> &ShmBufferLimits {
        &self.limits
    }

    /// Validates `wl_shm_pool` allocation or resize request.
    pub fn validate_pool_size(&self, requested_size_bytes: usize) -> Result<(), ShmError> {
        if requested_size_bytes > self.limits.max_pool_bytes {
            warn!(
                "Blocked wl_shm_pool allocation attempt: requested {} MB exceeds limit {} MB",
                requested_size_bytes / (1024 * 1024),
                self.limits.max_pool_bytes / (1024 * 1024)
            );
            return Err(ShmError::PoolTooLarge {
                requested: requested_size_bytes,
                limit: self.limits.max_pool_bytes,
            });
        }
        Ok(())
    }

    /// Validates individual `wl_shm` buffer creation request (`width`, `height`, `stride`, `format`).
    /// Returns the computed total buffer size in bytes if valid.
    pub fn validate_buffer_allocation(
        &self,
        width: i32,
        height: i32,
        stride: i32,
        _format: u32,
    ) -> Result<usize, ShmError> {
        if width <= 0 || height <= 0 || width > self.limits.max_dimension || height > self.limits.max_dimension {
            return Err(ShmError::InvalidDimensions {
                width,
                height,
                max: self.limits.max_dimension,
            });
        }

        if stride <= 0 {
            return Err(ShmError::InvalidStride { stride });
        }

        // Calculate required buffer size safely checking for multiplication overflow
        let height_u = height as usize;
        let stride_u = stride as usize;

        let total_bytes = height_u
            .checked_mul(stride_u)
            .ok_or(ShmError::DimensionOverflow {
                width,
                height,
                stride,
            })?;

        if total_bytes > self.limits.max_buffer_bytes {
            warn!(
                "Blocked malicious wl_shm buffer creation: {}x{} (stride {}) requires {} MB (exceeds {} MB limit)",
                width,
                height,
                stride,
                total_bytes / (1024 * 1024),
                self.limits.max_buffer_bytes / (1024 * 1024)
            );
            return Err(ShmError::BufferTooLarge {
                requested: total_bytes,
                limit: self.limits.max_buffer_bytes,
            });
        }

        Ok(total_bytes)
    }
}

/// Smithay Wayland SHM state wrapper enforcing buffer size boundaries.
#[derive(Debug)]
pub struct CompositorShmState {
    pub guard: WlShmGuard,
}

impl CompositorShmState {
    pub fn new(limits: ShmBufferLimits) -> Self {
        Self {
            guard: WlShmGuard::new(limits),
        }
    }

    pub fn with_default_limits() -> Self {
        Self::new(ShmBufferLimits::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_guard_blocks_oversized_buffers() {
        let guard = WlShmGuard::new(ShmBufferLimits::default());

        // 8000x8000 32bpp (stride 32000) -> 256,000,000 bytes (244 MB) -> Should pass
        let res = guard.validate_buffer_allocation(8000, 8000, 32000, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 256_000_000);

        // 16000x16000 32bpp (stride 64000) -> 1,024,000,000 bytes (976 MB) -> Exceeds 512MB limit -> Should be rejected
        let res = guard.validate_buffer_allocation(16000, 16000, 64000, 0);
        assert_eq!(
            res,
            Err(ShmError::BufferTooLarge {
                requested: 1_024_000_000,
                limit: DEFAULT_MAX_WL_SHM_BUFFER_SIZE_BYTES
            })
        );
    }

    #[test]
    fn test_shm_guard_blocks_overflows_and_invalid_dimensions() {
        let guard = WlShmGuard::new(ShmBufferLimits::default());

        // Negative dimension
        assert_eq!(
            guard.validate_buffer_allocation(-10, 100, 400, 0),
            Err(ShmError::InvalidDimensions {
                width: -10,
                height: 100,
                max: DEFAULT_MAX_WL_SHM_DIMENSION
            })
        );

        // Dimension exceeding max
        assert_eq!(
            guard.validate_buffer_allocation(20000, 100, 80000, 0),
            Err(ShmError::InvalidDimensions {
                width: 20000,
                height: 100,
                max: DEFAULT_MAX_WL_SHM_DIMENSION
            })
        );

        // Arithmetic overflow
        assert_eq!(
            guard.validate_buffer_allocation(1000, i32::MAX, i32::MAX, 0),
            Err(ShmError::InvalidDimensions {
                width: 1000,
                height: i32::MAX,
                max: DEFAULT_MAX_WL_SHM_DIMENSION
            })
        );
    }

    #[test]
    fn test_shm_guard_blocks_oversized_pools() {
        let guard = WlShmGuard::new(ShmBufferLimits::default());

        assert!(guard.validate_pool_size(256 * 1024 * 1024).is_ok());
        assert_eq!(
            guard.validate_pool_size(600 * 1024 * 1024),
            Err(ShmError::PoolTooLarge {
                requested: 600 * 1024 * 1024,
                limit: DEFAULT_MAX_WL_SHM_POOL_SIZE_BYTES
            })
        );
    }
}
