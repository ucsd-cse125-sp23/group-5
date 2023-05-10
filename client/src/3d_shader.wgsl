// Vertex shader
struct CameraUniform {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
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
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
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
    out.clip_position = camera.proj * camera.view * model_matrix * vec4<f32>(model.position, 1.0);
    var world_coords = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_coords = vec3<f32>(
        world_coords[0] / world_coords[3],
        world_coords[1] / world_coords[3],
        world_coords[2] / world_coords[3],
    );
    out.normal = normalize(normal_matrix * model.normal);
    out.tangent = normalize(model.tangent);
    out.bitangent = normalize(model.bitangent);
    return out;
}

// Fragment shader

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
@group(0) @binding(0)
var<uniform> phong_mtl : PhongUniform;
@group(0) @binding(1)
var<uniform> flags : u32;

const HAS_DIFFUSE_TEXTURE :u32 = 1u;
const HAS_AMBIENT_TEXTURE: u32 = 2u;
const HAS_SPECULAR_TEXTURE: u32 = 4u;
const HAS_NORMAL_TEXTURE: u32 = 8u;
const HAS_SHININESS_TEXTURE: u32 = 16u;
const EMPTY_FLAG :u32 = 0u;

@group(0) @binding(2)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(3)
var s_diffuse: sampler;
@group(0) @binding(4)
var t_normal: texture_2d<f32>;
@group(0) @binding(5)
var s_normal: sampler;
@group(0) @binding(6)
var t_specular: texture_2d<f32>;
@group(0) @binding(7)
var s_specular: sampler;
@group(0) @binding(8)
var t_ambient: texture_2d<f32>;
@group(0) @binding(9)
var s_ambient: sampler;
@group(0) @binding(10)
var t_shininess: texture_2d<f32>;
@group(0) @binding(11)
var s_shininess: sampler;

@group(3) @binding(0)
var<uniform> texture_color : vec3<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ambient = phong_mtl.ambient;
    var diffuse = phong_mtl.diffuse;
    var specular = phong_mtl.specular;
    var s = phong_mtl.shininess;
    var gloss = 1.0;
    if ((HAS_DIFFUSE_TEXTURE & flags) != EMPTY_FLAG){
        var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        diffuse = vec3<f32>(t[0], t[1], t[2]);
    }
    if ((HAS_AMBIENT_TEXTURE & flags) != EMPTY_FLAG){
        var t = textureSample(t_ambient, s_ambient, in.tex_coords);
        ambient = vec3<f32>(t[0], t[1], t[2]);
    }
    if ((HAS_SPECULAR_TEXTURE & flags) != EMPTY_FLAG){
        var t = textureSample(t_specular, s_specular, in.tex_coords);
        specular = vec3<f32>(t[0], t[1], t[2]);
    }
    if ((HAS_SHININESS_TEXTURE & flags) != EMPTY_FLAG){
        var t = textureSample(t_specular, s_specular, in.tex_coords);
        // should be grayscale
        // TODO: not sure if this is something to multiply later 
        //       or if it represents Ns
        gloss = t[0];
    }

    var color : vec3<f32> = 0.1 * ambient;

    // construct coordinate system
    var normal = normalize(in.normal);
    var bitangent = normalize(cross(in.normal, in.tangent));
    var tangent = cross(bitangent, normal);
    // color = ( tangent + 1.0 ) / 2.0;

    //calculate normal
    if ((HAS_NORMAL_TEXTURE & flags) != EMPTY_FLAG){
        var t = 2.0 * textureSample(t_normal, s_normal, in.tex_coords) - 1.0;
        normal = normalize(
            t[0] * tangent + t[1] * bitangent + t[2] * normal
        );
    }

    var c_loc = vec3<f32>(camera.location[0], camera.location[1], camera.location[2]);
    var eye_dirn : vec3<f32> = normalize(c_loc - in.world_coords);

    color += diffuse;

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
        // var d_int = clamp(dot(normal, light_dir), 0.0, 1.0);
        // if ((HAS_DIFFUSE_TEXTURE & flags) != EMPTY_FLAG){
        //     var t = textureSample(t_diffuse, s_diffuse, vec2<f32>(0.0, d_int));
        //     diffuse = vec3<f32>(t[0], t[1], t[2]);
        // }
 
        // color += light_col * diffuse; //* max(d_int, 0.0);
        
        // specular
        var half_vec : vec3<f32> = normalize(eye_dirn + light_dir);
        var nDotH : f32 = dot(normal, half_vec);
        if (s > 0.0 && pow (max(nDotH, 0.0), s) > 0.8){
            color += light_col * specular * gloss; 
        }
        // if (s > 0.0){
        //     color += light_col * specular * pow(max(nDotH, 0.0), s) * gloss; 
        // }
    }
    
    color = color * texture_color;

    // color = vec3<f32>(in.tex_coords, 0.0);
    return vec4<f32>( clamp(color, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0)) , 1.0) ;
}