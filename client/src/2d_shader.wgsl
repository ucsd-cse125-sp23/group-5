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
    @builtin(position) @invariant clip_position: vec4<f32>,
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
var<uniform> texture_color : vec3<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return in.color * t * vec4(texture_color, 1.0);
}