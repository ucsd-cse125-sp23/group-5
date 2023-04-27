// Vertex shader
struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;


const MAX_LIGHT = 16;
struct LightsUniform{
    positions: array<vec4<f32>, MAX_LIGHT>,
    colors: array<vec4<f32>, MAX_LIGHT>,
    num_lights: u32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
};
@group(2) @binding(0)
var<uniform> lights: LightsUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location( 9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_coords: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    var world_coords = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_coords = vec3<f32>(
        world_coords[0] / world_coords[3],
        world_coords[1] / world_coords[3],
        world_coords[2] / world_coords[3],
    );
    out.normal = normalize(normal_matrix * model.normal);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
struct PhongUniform{
    ambient: vec3<f32>,
    _pa: f32,
    diffuse: vec3<f32>,
    _pd: f32,
    specular: vec3<f32>,
    _ps: f32,
    shininess: f32,
    _p0: f32,
    _p1: f32,
    _p2: f32,
}
@group(0) @binding(2)
var<uniform> phong_mtl : PhongUniform;
@group(0) @binding(3)
var<uniform> flags : u32;
const HAS_DIFFUSE_TEXTURE :u32 = 1u;
const HAS_AMBIENT_TEXTURE: u32 = 2u;
const HAS_SPECULAR_TEXTURE: u32 = 4u;
const HAS_NORMAL_TEXTURE: u32 = 8u;
const HAS_SHININESS_TEXTURE: u32 = 16u;
const EMPTY_FLAG :u32 = 0u;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color : vec3<f32> = 0.1 * phong_mtl.ambient;

    // diff begins here
    var normal = normalize(in.normal);
    var c_loc = vec3<f32>(camera.location[0], camera.location[1], camera.location[2]);
    var eye_dirn : vec3<f32> = normalize(c_loc - in.world_coords);

    for (var ind: u32 = 0u; ind < lights.num_lights; ind = ind + 1u) {
        var light_dir : vec3<f32>;
        var attenuation: f32 = 1.0;
        var light_col  = vec3<f32>(lights.colors[ind][0], lights.colors[ind][1], lights.colors[ind][2]);
        if (lights.positions[ind][3] == 0.0){ // directional light
            light_dir = normalize(vec3<f32>(
                lights.positions[ind][0], 
                lights.positions[ind][1], 
                lights.positions[ind][2]
            ));
        } else { // point light
            var disp = vec3<f32>(
                lights.positions[ind][0] / lights.positions[ind][3], 
                lights.positions[ind][1] / lights.positions[ind][3], 
                lights.positions[ind][2] / lights.positions[ind][3],
            ) - in.world_coords;
            attenuation = 1.0 / (0.01 * dot(disp, disp) + 1.0);
            light_dir = normalize(disp);
        }

        // diffuse
        if (dot(normal, light_dir) * attenuation < 0.0){
            color +=  light_col * phong_mtl.diffuse * 0.0;
        } else if (dot(normal, light_dir) * attenuation < 0.3){
            color +=  light_col * phong_mtl.diffuse * 0.5;
        } else {
            color +=  light_col * phong_mtl.diffuse;
        }
        // specular
        var half_vec : vec3<f32> = normalize(eye_dirn + light_dir);
        var nDotH : f32 = dot(normal, half_vec);
        if (pow (max(nDotH, 0.0), phong_mtl.shininess) * attenuation > 0.7){
            color += light_col * phong_mtl.specular; 
        }
    }

    if ((HAS_DIFFUSE_TEXTURE & flags) != EMPTY_FLAG){
        var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        color *= vec3<f32>(t[0], t[1], t[2]);
    }


    return vec4<f32>( clamp(color, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0)) , 1.0) ;
}