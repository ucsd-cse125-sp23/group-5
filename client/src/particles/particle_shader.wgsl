struct VertexInput {
    @location(0) tex: vec2<f32>,
};
struct InstanceInput {
    @location(1) start_pos: vec4<f32>,
    @location(2) velocity: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) spawn_time: f32,
    @location(5) size: f32,
    @location(6) tex_id: f32,
    @location(7) z_pos: f32,
    @location(8) time_elapsed: f32,
    // allow size to grow
    // if it's non-zero: 
    // option A) size = (size) / (1 + e^{-size_growth * (t - halflife)})
    // option B) size = (2*size) / (1 + e^{-size_growth * (t)}) - size
    @location(9) size_growth: f32,
    @location(10) halflife: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>
};

struct CameraUniform {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

const NUM_TEXTURES: f32 = 6.0;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vec2(model.tex[0], (model.tex[1] / (NUM_TEXTURES) + (instance.tex_id / NUM_TEXTURES)));
    var start_disp = vec3<f32>(instance.start_pos[0], instance.start_pos[1], instance.start_pos[2]);
    var start_angle = instance.start_pos[3];
    var linear_v = vec3<f32>(instance.velocity[0], instance.velocity[1], instance.velocity[2]);
    var angular_v = instance.velocity[3];
    // assuming camera homogenous coord is always 1.0
    var cpos = vec3<f32>(camera.view_pos[0], camera.view_pos[1], camera.view_pos[2]);
    var z_prime = normalize(cpos - start_disp);
    var up = vec3<f32>(0.0, 1.0, 0.0);
    var x_prime = normalize(cross(up, z_prime));
    // exact orientation of axes is not super important
    if (dot(x_prime, x_prime) == 0.0){
        up = vec3<f32>(1.0, 0.0, 0.0);
        x_prime = normalize(cross(up, z_prime));
    }
    var y_prime = cross(z_prime, x_prime);
    // get time
    var time_alive = instance.time_elapsed - instance.spawn_time;
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
    position = coord_matrix * position;
    // then move to start position
    position += start_disp;
    // then move according to velocity
    position += time_alive * linear_v;
    // then project
    out.clip_position = camera.view * vec4<f32>(position, 1.0);
    // set z, for ordering issues
    out.clip_position[2] = instance.z_pos;
    out.clip_position = camera.proj * out.clip_position;
    out.color = instance.color;
    return out;
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return t * in.color;
}
