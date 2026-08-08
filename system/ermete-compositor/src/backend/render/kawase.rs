//! Dual-Pass Kawase Blur Pipeline Implementation for Ermete OS Compositor.

use tracing::info;

/// Uniform parameters sent to the GPU shader per pass.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct KawaseUniforms {
    pub texel_size: [f32; 2],
    pub offset: f32,
    pub pass_type: f32, // 0.0 = Downsample, 1.0 = Upsample
}

/// Configuration settings for the Dual-Pass Kawase Blur.
#[derive(Debug, Clone)]
pub struct KawaseBlurConfig {
    pub passes: usize,
    pub radius: f32,
    #[allow(dead_code)]
    pub downscale_factor: f32,
}

impl Default for KawaseBlurConfig {
    fn default() -> Self {
        Self {
            passes: 4,
            radius: 3.5,
            downscale_factor: 2.0,
        }
    }
}

/// Descriptor for a single pass execution within the blur chain.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KawasePassDescriptor {
    pub pass_index: usize,
    pub is_upsample: bool,
    pub uniforms: KawaseUniforms,
    pub target_width: u32,
    pub target_height: u32,
}

/// Dual-Pass Kawase Blur Render Pipeline orchestrator.
pub struct KawaseBlurPipeline {
    config: KawaseBlurConfig,
}

impl KawaseBlurPipeline {
    /// Creates a new Kawase Blur pipeline with specified parameters.
    pub fn new(config: KawaseBlurConfig) -> Self {
        info!(
            "Initializing Dual-Pass Kawase Blur Render Pipeline (passes={}, radius={:.2})",
            config.passes, config.radius
        );
        Self { config }
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &KawaseBlurConfig {
        &self.config
    }

    #[allow(dead_code)]
    pub fn set_config(&mut self, config: KawaseBlurConfig) {
        self.config = config;
    }

    /// Computes the offset kernel parameter for a given pass iteration and base radius.
    #[allow(dead_code)]
    pub fn compute_pass_offset(pass_idx: usize, base_radius: f32) -> f32 {
        base_radius + (pass_idx as f32 * 0.75)
    }

    /// Generates the full sequence of downsample and upsample pass descriptors
    /// for rendering a given input texture size (width, height).
    #[allow(dead_code)]
    pub fn build_pass_chain(&self, input_width: u32, input_height: u32) -> Vec<KawasePassDescriptor> {
        let passes = self.config.passes.max(1);
        let mut descriptors = Vec::with_capacity(passes * 2);

        let mut cur_w = input_width;
        let mut cur_h = input_height;

        // 1. Downsample Passes
        for i in 0..passes {
            cur_w = (cur_w as f32 / self.config.downscale_factor).max(1.0) as u32;
            cur_h = (cur_h as f32 / self.config.downscale_factor).max(1.0) as u32;

            let offset = Self::compute_pass_offset(i, self.config.radius);
            let texel_size = [1.0 / cur_w.max(1) as f32, 1.0 / cur_h.max(1) as f32];

            descriptors.push(KawasePassDescriptor {
                pass_index: i,
                is_upsample: false,
                uniforms: KawaseUniforms {
                    texel_size,
                    offset,
                    pass_type: 0.0,
                },
                target_width: cur_w,
                target_height: cur_h,
            });
        }

        // 2. Upsample Passes
        for i in (0..passes).rev() {
            cur_w = (cur_w as f32 * self.config.downscale_factor).min(input_width as f32) as u32;
            cur_h = (cur_h as f32 * self.config.downscale_factor).min(input_height as f32) as u32;

            let offset = Self::compute_pass_offset(i, self.config.radius);
            let texel_size = [1.0 / cur_w.max(1) as f32, 1.0 / cur_h.max(1) as f32];

            descriptors.push(KawasePassDescriptor {
                pass_index: i,
                is_upsample: true,
                uniforms: KawaseUniforms {
                    texel_size,
                    offset,
                    pass_type: 1.0,
                },
                target_width: cur_w,
                target_height: cur_h,
            });
        }

        descriptors
    }

    /// Accessor for GLSL and WGSL shader source code.
    #[allow(dead_code)]
    pub fn glsl_vertex_source() -> &'static str {
        super::shaders::KAWASE_VERTEX_GLSL
    }

    #[allow(dead_code)]
    pub fn glsl_downsample_source() -> &'static str {
        super::shaders::KAWASE_DOWNSAMPLE_GLSL
    }

    #[allow(dead_code)]
    pub fn glsl_upsample_source() -> &'static str {
        super::shaders::KAWASE_UPSAMPLE_GLSL
    }

    #[allow(dead_code)]
    pub fn wgsl_source() -> &'static str {
        super::shaders::KAWASE_BLUR_WGSL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kawase_pass_chain_building() {
        let pipeline = KawaseBlurPipeline::new(KawaseBlurConfig::default());
        let chain = pipeline.build_pass_chain(1920, 1080);
        assert_eq!(chain.len(), 8); // 4 downsample + 4 upsample
        assert!(!chain[0].is_upsample);
        assert!(chain[7].is_upsample);
    }
}
