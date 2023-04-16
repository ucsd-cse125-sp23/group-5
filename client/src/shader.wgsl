// Vertex shader
struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) @interpolate(linear, center) normal: vec3<f32>,
    @location(2) @interpolate(flat) ambient: vec3<f32>,
    @location(3) @interpolate(flat) diffuse: vec3<f32>,
    @location(4) @interpolate(flat) specular: vec3<f32>,
    @location(5) @interpolate(flat) emission: vec3<f32>,
    @location(6) @interpolate(flat) s: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) @interpolate(linear, center) normal: vec3<f32>,
    @location(2) @interpolate(flat) ambient: vec3<f32>,
    @location(3) @interpolate(flat) diffuse: vec3<f32>,
    @location(4) @interpolate(flat) specular: vec3<f32>,
    @location(5) @interpolate(flat) emission: vec3<f32>,
    @location(6) @interpolate(flat) s: f32,
    @location(7) world_coords: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.ambient = model.ambient;
    out.diffuse = model.diffuse;
    out.specular = model.specular;
    out.s = model.s;
    out.world_coords = model.position;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var normal = normalize(in.normal);

    // For now, assume light at (1.0, 0.0, 0.0)
    var light_dir : vec3<f32> = normalize(vec3<f32>(1.0, 0.0, 0.0));
    // return vec4<f32>(in.color * dot(in.normal, light_dir), 1.0);

    var color : vec3<f32> = in.ambient;
    color += in.emission;
    // diffuse
    if (dot(normal, light_dir) < 0.0){
        color += in.diffuse * 0.2;
    } else if (dot(normal, light_dir) < 0.3){
        color +=  in.diffuse * 0.5;
    } else {
        color +=  in.diffuse;
    }
    // specular
    var c_loc = vec3<f32>(camera.location[0], camera.location[1], camera.location[2]);
    var eye_dirn : vec3<f32> = normalize(c_loc - in.world_coords);
    var half_vec : vec3<f32> = normalize(eye_dirn + light_dir);
    var nDotH : f32 = dot(normal, half_vec);
    if (pow (max(nDotH, 0.0), in.s) > 0.7){
        color += in.specular; 
    }

    // color = half_vec;
    // return vec4((normalize(in.world_coords) + 1.0) / 2.0, 1.0);

    return vec4<f32>( clamp(color, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0)), 1.0);
}
