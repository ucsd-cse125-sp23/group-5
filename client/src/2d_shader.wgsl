struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture: vec2<f32>,
};
struct InstanceInput {
    @location(3) inst_matrix_0: vec4<f32>,
    @location(4) inst_matrix_1: vec4<f32>,
    @location(5) inst_matrix_2: vec4<f32>,
    @location(6) inst_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

const DEPTH :f32 = 0.0;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.inst_matrix_0,
        instance.inst_matrix_1,
        instance.inst_matrix_2,
        instance.inst_matrix_3,
    );
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = model_matrix * vec4(model.position, DEPTH, 1.0);
    out.tex_coords = model.texture;
    return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;


@group(1) @binding(0)
var t_mask: texture_2d<f32>;
@group(1) @binding(1)
var s_mask: sampler;

struct CameraUniform {
    ambient_multiplier: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(2) @binding(0)
var<uniform> camera: CameraUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var mask = textureSample(t_mask, s_mask, in.tex_coords);
    var color = in.color * mask * t;
    return color;
}

fn grayscale(c: vec4<f32>) -> vec4<f32>{
    var linearized = c;
    var gray = 0.2126 * linearized.x + 0.7152 * linearized.y + 0.0722 * linearized.z;
    return vec4<f32>(gray, gray, gray, c.w);
}