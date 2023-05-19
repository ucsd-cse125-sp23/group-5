@group(0) @binding(0) var box_tex: texture_cube<f32>;
@group(0) @binding(1) var sampler: sampler;

struct CameraUniform {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.position;
    // move the vertex, then calculate clip position
    out.clip_position = vec4(model.position, 1.0) + camera.location;
    out.clip_position = camera.proj * camera.view * out.clip_position;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(box_tex, samples, in.tex_coords);
}