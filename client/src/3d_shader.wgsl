// Vertex shader
struct CameraUniform {
    ambient_multiplier: vec4<f32>,
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
    positions_2: array<vec4<f32>, MAX_LIGHT>,
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
    var hom_loc = camera.inv_view_proj * in.clip_position;
    var loc = hom_loc.xyz / hom_loc.w;

    var ambient = phong_mtl.ambient;
    var diffuse = phong_mtl.diffuse;
    var specular = phong_mtl.specular;
    var s = phong_mtl.shininess;
    var gloss = 0.3;
    // FOR DEBUG
    // specular = vec3<f32>(1.0, 1.0, 1.0);
    // s = 100.0;
    //----
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

    var shine_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    // construct coordinate system
    var normal = normalize(in.normal);
    var bitangent = normalize(cross(in.normal, in.tangent));
    var tangent = cross(bitangent, normal);

    //calculate normal
    if ((HAS_NORMAL_TEXTURE & flags) != EMPTY_FLAG){
        var t = 2.0 * textureSample(t_normal, s_normal, in.tex_coords) - 1.0;
        normal = normalize(
            t[0] * tangent + t[1] * bitangent + t[2] * normal
        );
    }

    var c_loc = vec3<f32>(camera.location[0], camera.location[1], camera.location[2]);
    var eye_dirn : vec3<f32> = normalize(c_loc - in.world_coords);

    // We're assuming everything comes from ambient lighting
    // extra lighting just changes the amount of light we get
    var light = camera.ambient_multiplier;

    for (var ind: u32 = 0u; ind < lights.num_lights; ind = ind + 1u) {
        var light_dir : vec3<f32>;
        var attenuation: f32 = 1.0;
        var light_col  = lights.colors[ind].xyz;
        if (lights.positions_2[ind].w > 0.0){ // Line segment light
            // we'll use linear attenuation
            // assume last term of position is the maximum reach of the light
            var p1 = lights.positions[ind].xyz;
            var p2 = lights.positions_2[ind].xyz;
            var proj_dist = dot(in.world_coords - p1, normalize(p2 - p1));
            var light_len = sqrt(dot(p2 - p1, p2 - p1));
            var max_dist: f32;
            if (proj_dist < 0.0){
                max_dist = lights.positions[ind].w;
                light_dir = p1 - in.world_coords;
            } else if (proj_dist > light_len){
                max_dist = lights.positions_2[ind].w;
                light_dir = p2 - in.world_coords;
            } else {
                proj_dist /= light_len;
                max_dist = lights.positions[ind].w + (lights.positions_2[ind].w - lights.positions[ind].w) * proj_dist;
                light_dir = p1 + (p2 - p1) * proj_dist - in.world_coords;
            }
            // calculate attenuation
            var dist = sqrt(dot(light_dir, light_dir)) / max_dist;
            if (dist > 1.0){
                // zero contribution
                continue;
            }
            attenuation = 1.0 - (3.0 * pow(dist, 2.0) - 2.0 * pow(dist, 3.0));
        }
        else if (lights.positions[ind].w == 0.0){ // directional light
            light_dir = normalize(vec3<f32>(
                lights.positions[ind][0], 
                lights.positions[ind][1], 
                lights.positions[ind][2]
            ));
        } 
        else { // point light
            var disp = vec3<f32>(
                lights.positions[ind][0] / lights.positions[ind][3], 
                lights.positions[ind][1] / lights.positions[ind][3], 
                lights.positions[ind][2] / lights.positions[ind][3],
            ) - in.world_coords;
            attenuation = 1.0 / (dot(disp, disp) + 1.0);
            light_dir = normalize(disp);
        }

        // diffuse
        var d_int = clamp(dot(normal, light_dir), 0.0, 1.0);
        // alpha set to 0 since we're adding and it should already be 1.0
        // light += vec4<f32>(light_col, 0.0) * attenuation; // w.o normals
        light += vec4<f32>(light_col, 0.0) * d_int * attenuation;
        
        // specular
        var half_vec : vec3<f32> = normalize(eye_dirn + light_dir);
        var nDotH : f32 = dot(normal, half_vec);
        if (s > 0.0){
            shine_color += light_col * specular * pow(max(nDotH, 0.0), s) * gloss * d_int; 
        }
    }
    
    diffuse = diffuse * texture_color;
    // limit diffuse to given colors from texture
    var color_hsv = rgb2hsv(diffuse.xyz);
    // we are assuming all lights are white, they only contribute to luminance
    color_hsv.z *= light.x;
    color_hsv = clamp(color_hsv, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0));
    var color = vec4<f32>(hsv2rgb(color_hsv), 1.0);
    // var color = vec4<f32>(diffuse, 1.0) * clamp(light, vec4<f32>(0.0, 0.0, 0.0, 1.0), vec4<f32>(2.0, 2.0, 2.0, 1.0));
    color += vec4<f32>(clamp(shine_color, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0)), 0.0);

    // color = vec3<f32>(in.tex_coords, 0.0);
    return clamp(color, vec4<f32>(0.0, 0.0, 0.0, 1.0), vec4<f32>(1.0, 1.0, 1.0, 1.0));
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