struct VertexInput {
    @location(0) tex: vec2<f32>,
};
struct InstanceInput {
    @location(1) start_pos: vec4<f32>,
    @location(2) velocity: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) n1: vec4<f32>,
    @location(5) n2: vec4<f32>,
    @location(6) spawn_time: f32,
    @location(7) size: f32,
    @location(8) tex_id: i32,
    @location(9) z_pos: f32,
    @location(10) time_elapsed: f32,
    // allow size to grow
    // if it's non-zero: 
    // option A) size = (size) / (1 + e^{-size_growth * (t - halflife)})
    // option B) size = (2*size) / (1 + e^{-size_growth * (t)}) - size
    @location(11) size_growth: f32,
    @location(12) halflife: f32,
    @location(13) FLAG: u32,
}

struct RibbonInstance{
    pos_1: vec4<f32>,
    pos_2: vec4<f32>,
    color: vec4<f32>,
    n1: vec4<f32>,
    n2: vec4<f32>,
    t1: f32,
    t2: f32,
    tex_id: i32,
    z_max: f32,
    time_elapsed: f32,
    visible_time: f32,
}

struct TrailInstance{
    pos_1: vec4<f32>,
    pos_2: vec4<f32>,
    pos_3: vec4<f32>,
    pos_4: vec4<f32>,
    color: vec4<f32>,
    t1: f32,
    t2: f32,
    tex_id: i32,
    z_max: f32,
    time_elapsed: f32,
    visible_time: f32,
}

const POINT_PARTICLE : u32 = 0u;
const RIBBON_PARTICLE: u32 = 1u;
const TRAIL_PARTICLE : u32 = 2u;
// color wise: plan to fade ends of ribbons/trails, but not the sides

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_id: i32,
    @location(3) FLAG: u32,
    // only useful for ribbon/trails
    @location(4) time_elapsed: f32,
    @location(5) visible_time: f32,
    @location(6) time: f32,
};

struct CameraUniform {
    ambient_multiplier: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    if (instance.FLAG == POINT_PARTICLE){
        return vs_point(model, instance);
    } else if (instance.FLAG == RIBBON_PARTICLE) {
        // only implement ribbon for now
        var ribbon: RibbonInstance;
        ribbon.pos_1 = instance.start_pos;
        ribbon.pos_2 = instance.velocity;
        ribbon.color = instance.color;
        ribbon.t1 = instance.spawn_time;
        ribbon.t2 = instance.size;
        ribbon.n1 = instance.n1;
        ribbon.n2 = instance.n2;
        ribbon.tex_id = instance.tex_id;
        ribbon.z_max = instance.z_pos;
        ribbon.time_elapsed = instance.time_elapsed;
        ribbon.visible_time = instance.halflife;
        return vs_ribbon(model, ribbon);
    } else {
        var trail: TrailInstance;
        trail.pos_1 = instance.start_pos;
        trail.pos_2 = instance.velocity;
        trail.pos_3 = instance.n1;
        trail.pos_4 = instance.n2;
        trail.color = instance.color;
        trail.t1 = instance.spawn_time;
        trail.t2 = instance.size;
        trail.tex_id = instance.tex_id;
        trail.z_max = instance.z_pos;
        trail.time_elapsed = instance.time_elapsed;
        trail.visible_time = instance.halflife;
        return vs_trail(model, trail);
    }
}

fn vs_point(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    // texture
    out.tex_coords = model.tex;
    out.tex_id = instance.tex_id;

    // calculating vertex locations
    var start_disp = vec3<f32>(instance.start_pos[0], instance.start_pos[1], instance.start_pos[2]);
    var start_angle = instance.start_pos[3];
    var linear_v = vec3<f32>(instance.velocity[0], instance.velocity[1], instance.velocity[2]);
    var angular_v = instance.velocity[3];
    // find center coordinate
    var time_alive = instance.time_elapsed - instance.spawn_time;
    start_disp += time_alive * linear_v;

    // assuming camera homogenous coord is always 1.0
    var cpos = vec3<f32>(camera.location[0], camera.location[1], camera.location[2]);
    var z_prime = normalize(cpos - start_disp);
    var up = vec3<f32>(0.0, 1.0, 0.0);
    var x_prime = normalize(cross(up, z_prime));
    // exact orientation of axes is not super important
    if (dot(x_prime, x_prime) == 0.0){
        up = vec3<f32>(1.0, 0.0, 0.0);
        x_prime = normalize(cross(up, z_prime));
    }
    var y_prime = cross(z_prime, x_prime);
    // scale first
    var size = instance.size;
    if (instance.size_growth != 0.0){
        size = (2.0 * size) / (1.0 + exp(-1.0 * instance.size_growth * time_alive)) - size;
    }
    var position = vec3<f32>(model.tex[0] - 0.5, 0.5 - model.tex[1], 0.0) * size * 0.01;
    // then rotate + angular velocity rotation
    var theta = start_angle + time_alive * angular_v;
    var rot_mat = mat3x3<f32>(
            cos(theta), sin(theta), 0.0,
        -sin(theta), cos(theta), 0.0,
        0.0, 0.0, 1.0,
    );
    position = rot_mat * position;
    // then move to alternate coordinates 
    let coord_matrix = mat3x3<f32>(
        x_prime,
        y_prime,
        z_prime,
    );
    position = coord_matrix * position + start_disp;
    // then project
    out.clip_position = camera.view * vec4<f32>(position, 1.0);
    // set z, for ordering issues
    out.clip_position[2] = instance.z_pos;
    out.clip_position = camera.proj * out.clip_position;
    out.color = instance.color;
    out.FLAG = POINT_PARTICLE;
    return out;
}

fn vs_ribbon(
    model: VertexInput,
    instance: RibbonInstance,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex;
    out.color = instance.color;
    out.tex_id = instance.tex_id;
    out.FLAG = RIBBON_PARTICLE;

    // pick which position
    var which = instance.pos_1;
    out.time = instance.t1;
    var ribbon_dir = instance.n1.xyz;
    if (model.tex[0] > 0.0){
        which = instance.pos_2;
        out.time = instance.t2;
        ribbon_dir = instance.n2.xyz;
    }
    var pos = which.xyz;

    // build coordinates
    // assuming camera homogenous coord is always 1.0
    var cpos = vec3<f32>(camera.location[0], camera.location[1], camera.location[2]);
    var z_prime = normalize(cpos - pos);
    var y_prime = normalize(cross(z_prime, ribbon_dir));

    if (model.tex[1] > 0.0){ // lower vertex
        pos -= which.w * y_prime * 0.01;
    } else { // upper vertex
        pos += which.w * y_prime * 0.01;
    }
    out.clip_position = camera.view * vec4<f32>(pos, 1.0);
    if (out.clip_position[2] > instance.z_max){
        out.clip_position[2] = instance.z_max;
    }
    out.clip_position = camera.proj * out.clip_position;

    // fill out constants
    out.time_elapsed = instance.time_elapsed;
    out.visible_time = instance.visible_time;
    return out;
}

fn vs_trail(
    model: VertexInput,
    instance: TrailInstance,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex;
    out.color = instance.color;
    out.tex_id = instance.tex_id;
    out.FLAG = TRAIL_PARTICLE;

    // pick which position
    var which: vec4<f32>;
    if (model.tex[0] > 0.0){
        which = instance.pos_3;
        out.time = instance.t2;
        if (model.tex[0] > 0.0) {
            which = instance.pos_4;
        }
    } else{
        out.time = instance.t1;
        which = instance.pos_2;
        if (model.tex[0] > 0.0) {
            which = instance.pos_1;
        }
    }
    var pos = which.xyz;
    out.clip_position = camera.view * vec4<f32>(pos, 1.0);
    if (out.clip_position[2] > instance.z_max){
        out.clip_position[2] = instance.z_max;
    }
    out.clip_position =  camera.proj * out.clip_position;    

    // fill out constants
    out.time_elapsed = instance.time_elapsed;
    out.visible_time = instance.visible_time;
    return out;
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// we will use the first and last 5% of the visible portion to fade in/out for ribbon
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = textureSample(t_diffuse, s_diffuse, in.tex_coords, in.tex_id);
    if (in.FLAG == POINT_PARTICLE){
        return t * in.color * camera.ambient_multiplier;
    } else {
        var time = in.time;
        // return vec4<f32>(in.time / 10.0 * vec3<f32>(1.0, 1.0, 1.0), 1.0);
        if (time < in.time_elapsed || time > in.time_elapsed + in.visible_time){
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        // return t * in.color * camera.ambient_multiplier;
        var time_multiplier = 1.0;
        var percent_time = (time - in.time_elapsed) / in.visible_time;
        if (percent_time < 0.8){
            var x = percent_time * 1.25; // dividing by 0.8
            time_multiplier = 3.0 * x * x - 2.0 * x * x * x;
        }
        var c = t * in.color * camera.ambient_multiplier;
        c[3] *= time_multiplier;
        return c;
    }
}