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
    @location(7) _pad0: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    location: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InfoUniform{
    lifetime: f32, // in sec
    num_textures: f32,
}
@group(1) @binding(2)
var<uniform> info: InfoUniform;

@group(2) @binding(0)
var<uniform> time_elapsed: f32;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vec2(model.tex[0], (model.tex[1] / (info.num_textures) + (instance.tex_id / info.num_textures)));
    if (instance.spawn_time > time_elapsed || instance.spawn_time + info.lifetime < time_elapsed){
        out.clip_position = vec4(2.0, 2.0, 2.0, 1.0);
    } else {
        // TODO
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
        // scale first
        var position = vec3<f32>(model.tex[0] - 0.5, 0.5 - model.tex[1], 0.0) * instance.size * 0.01;
        // TODO: then rotate + angular velocity rotation
        var time_alive = time_elapsed - instance.spawn_time;
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
        out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
        // TODO: remove
        // out.clip_position[2] = 0.0; // set z to 0 so we can see it
    }
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
    // var t = vec4<f32>(in.clip_position[0], in.clip_position[0], 0.0, 0.1);
    return t;
}
