// Vertex shader
struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) @interpolate(perspective, center) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) @interpolate(perspective, center) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.normal = normalize(model.normal);
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // For now, assume light at (1.0, 1.0, 1.0)
    var light_dir : vec3<f32> = normalize(vec3<f32>(1.0, 1.0, 1.0));
    // return vec4((in.normal + 1.0) / 2.0, 1.0);
    if (dot(in.normal, light_dir) < 0.0){
        return vec4<f32>(in.color * 0.2, 1.0);
    } else if (dot(in.normal, light_dir) < 0.3){
        return vec4<f32>(in.color * 0.5, 1.0);
    } else {
        return vec4<f32>(in.color, 1.0);
    }
}
