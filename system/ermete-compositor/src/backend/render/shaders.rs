//! Dual-Pass Kawase Blur Shaders (GLSL & WGSL) for Ermete OS Compositor.

/// GLSL 450 Vertex Shader for Screen Quad Rendering.
#[allow(dead_code)]
pub const KAWASE_VERTEX_GLSL: &str = r#"#version 450

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_uv;

layout(location = 0) out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

/// GLSL 450 Fragment Shader for Dual-Pass Kawase Downsample.
#[allow(dead_code)]
pub const KAWASE_DOWNSAMPLE_GLSL: &str = r#"#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(binding = 0) uniform sampler2D u_texture;

layout(binding = 1) uniform BlurUniforms {
    vec2 u_texel_size;
    float u_offset;
    float u_pass_index;
};

// Dual-Pass Kawase Downsample: 5 texture fetches
vec4 kawase_downsample(vec2 uv, vec2 halfpixel) {
    vec4 sum = texture(u_texture, uv) * 4.0;
    sum += texture(u_texture, uv - halfpixel);
    sum += texture(u_texture, uv + halfpixel);
    sum += texture(u_texture, uv + vec2(halfpixel.x, -halfpixel.y));
    sum += texture(u_texture, uv + vec2(-halfpixel.x, halfpixel.y));
    return sum * 0.125;
}

void main() {
    vec2 halfpixel = u_texel_size * (u_offset + 0.5);
    fragColor = kawase_downsample(v_uv, halfpixel);
}
"#;

/// GLSL 450 Fragment Shader for Dual-Pass Kawase Upsample.
#[allow(dead_code)]
pub const KAWASE_UPSAMPLE_GLSL: &str = r#"#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 fragColor;

layout(binding = 0) uniform sampler2D u_texture;

layout(binding = 1) uniform BlurUniforms {
    vec2 u_texel_size;
    float u_offset;
    float u_pass_index;
};

// Dual-Pass Kawase Upsample: 8 texture fetches
vec4 kawase_upsample(vec2 uv, vec2 halfpixel) {
    vec4 sum = vec4(0.0);
    sum += texture(u_texture, uv + vec2(-halfpixel.x * 2.0, 0.0));
    sum += texture(u_texture, uv + vec2(-halfpixel.x, halfpixel.y * 2.0));
    sum += texture(u_texture, uv + vec2(0.0, halfpixel.y * 2.0));
    sum += texture(u_texture, uv + vec2(halfpixel.x, halfpixel.y * 2.0));
    sum += texture(u_texture, uv + vec2(halfpixel.x * 2.0, 0.0));
    sum += texture(u_texture, uv + vec2(halfpixel.x, -halfpixel.y * 2.0));
    sum += texture(u_texture, uv + vec2(0.0, -halfpixel.y * 2.0));
    sum += texture(u_texture, uv + vec2(-halfpixel.x, -halfpixel.y * 2.0));
    return sum * 0.125;
}

void main() {
    vec2 halfpixel = u_texel_size * (u_offset + 0.5);
    fragColor = kawase_upsample(v_uv, halfpixel);
}
"#;

/// WebGPU Shading Language (WGSL) Dual-Pass Kawase Shader Module.
#[allow(dead_code)]
pub const KAWASE_BLUR_WGSL: &str = r#"struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BlurUniforms {
    texel_size: vec2<f32>,
    offset: f32,
    pass_type: f32, // 0.0 = Downsample, 1.0 = Upsample
};

@group(0) @binding(0) var u_texture: texture_2d<f32>;
@group(0) @binding(1) var u_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: BlurUniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Fullscreen triangle mapping to quad [-1..1]
    var x = f32(i32(in_vertex_index & 1u) * 4 - 1);
    var y = f32(i32(in_vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn downsample(uv: vec2<f32>, halfpixel: vec2<f32>) -> vec4<f32> {
    var sum = textureSample(u_texture, u_sampler, uv) * 4.0;
    sum += textureSample(u_texture, u_sampler, uv - halfpixel);
    sum += textureSample(u_texture, u_sampler, uv + halfpixel);
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(halfpixel.x, -halfpixel.y));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(-halfpixel.x, halfpixel.y));
    return sum * 0.125;
}

fn upsample(uv: vec2<f32>, halfpixel: vec2<f32>) -> vec4<f32> {
    var sum = vec4<f32>(0.0);
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(-halfpixel.x * 2.0, 0.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(-halfpixel.x, halfpixel.y * 2.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0, halfpixel.y * 2.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(halfpixel.x, halfpixel.y * 2.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(halfpixel.x * 2.0, 0.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(halfpixel.x, -halfpixel.y * 2.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(0.0, -halfpixel.y * 2.0));
    sum += textureSample(u_texture, u_sampler, uv + vec2<f32>(-halfpixel.x, -halfpixel.y * 2.0));
    return sum * 0.125;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let halfpixel = uniforms.texel_size * (uniforms.offset + 0.5);
    if (uniforms.pass_type < 0.5) {
        return downsample(in.uv, halfpixel);
    } else {
        return upsample(in.uv, halfpixel);
    }
}
"#;
