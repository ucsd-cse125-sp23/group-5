@group(0) @binding(0) var box_tex: texture_cube<f32>;
@group(0) @binding(1) var box_sampler: sampler;

struct CameraUniform {
    ambient_multiplier: vec4<f32>,
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
    out.clip_position = vec4(model.position, 0.0) + camera.location;
    out.clip_position = camera.proj * camera.view * out.clip_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var diffuse = textureSample(box_tex, box_sampler, in.tex_coords);
    var color_hsv = rgb2hsv(diffuse.xyz);
    // we are assuming all lights are white, they only contribute to luminance
    color_hsv.z *= camera.ambient_multiplier.x;
    color_hsv = clamp(color_hsv, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0));
    var color = vec4<f32>(hsv2rgb(color_hsv), 1.0);
    if camera.ambient_multiplier.w == 0.0 {
        return grayscale(color);
    }
    return color;
}

fn grayscale(c: vec4<f32>) -> vec4<f32>{
    var linearized = c;
    var gray = 0.2126 * linearized.x + 0.7152 * linearized.y + 0.0722 * linearized.z;
    return vec4<f32>(gray, gray, gray, c.w);
}

// All components are in the range [0…1], including hue.
fn rgb2hsv(c: vec3<f32>) -> vec3<f32>
{
    var K: vec4<f32> = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    var p: vec4<f32> = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    var q: vec4<f32> = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    var d = q.x - min(q.w, q.y);
    var e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// All components are in the range [0…1], including hue.
fn hsv2rgb(c: vec3<f32>) -> vec3<f32>
{
    var K: vec4<f32> = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    var p: vec3<f32> = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0)), c.y);
}