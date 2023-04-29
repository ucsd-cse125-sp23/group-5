struct VertexInput {
    @location(0) tex: vec2<f32>,
};
struct InstanceInput {
    @location(1) start_pos: vec4<f32>,
    @location(2) velocity: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) spawn_time: u32,
    @location(5) size: f32,
    @location(6) tex_id: u32,
    @location(7) _pad0: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InfoUniform{
    lifetime: u32, // in ms
    num_textures: u32,
}
@group(1) @binding(2)
var<uniform> info: InfoUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    // TODO
    out.clip_position = vec4(model.tex, 0.0, 1.0);
    out.tex_coords = model.tex;
    return out;
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return t;
}