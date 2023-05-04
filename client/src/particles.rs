use crate::model::Vertex;
use crate::texture;
use wgpu::util::DeviceExt;
extern crate nalgebra_glm as glm;
use std::{
    f32::consts::{FRAC_PI_2, PI},
    ops::IndexMut,
};
// use rand_distr::Distribution;
use rand::Rng;
use rand_distr::{Distribution, Normal, Poisson, Uniform};

#[rustfmt::skip]
pub const PVertex : &[[f32; 2];4] = &[
    [0.0, 1.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
];

const PV_ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
    0 => Float32x2,
];

const NUM_TEXTURES: u32 = 5;

fn PV_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &PV_ATTRIBS,
    }
}

#[rustfmt::skip]
pub const PInd : &[u16; 6] = &[
    0, 1, 2,
    0, 2, 3,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
//// For particle instances
/// start_pos[3]: angle at the start
/// velocity[3]: angular velocity
/// color: color to tint the particle
/// spawn_time: relative to the time which the system was created in seconds
/// size: diameter in cm
pub struct Particle {
    // use last f32 as angluar position/velocity
    start_pos: [f32; 4],
    velocity: [f32; 4],
    color: [f32; 4],
    spawn_time: f32,
    size: f32,
    tex_id: f32,
    z_pos: f32,
    time_elapsed: f32,
    size_growth: f32,
    halflife: f32,
    _pad2: f32,
}

impl Particle {
    const ATTRIBS: [wgpu::VertexAttribute; 11] = wgpu::vertex_attr_array![
        1 => Float32x4, 2 => Float32x4, 3 => Float32x4,
        4 => Float32 ,  5 => Float32,   6 => Float32,
        7 => Float32,   8 => Float32,   9 => Float32,
        10 => Float32, 11 => Float32,
    ];
}

impl crate::model::Vertex for Particle {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait ParticleGenerator {
    ////
    /// list: vector to place generated particles in
    /// spawning time: amount of time to keep spawning
    /// spawn rate: average rate of spawning in particles per second
    /// num_textures: number of possible particle textures in one file (arranged vertically)
    /// returns: number of particles generated
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32,
        halflife: f32,
        tex_range: (u32, u32),
        color: glm::Vec4,
        rng: &mut rand::rngs::ThreadRng,
    ) -> u32;
}

pub struct FanGenerator {
    source: glm::Vec3,
    dir: glm::Vec3,
    up: glm::Vec3,
    right: glm::Vec3,
    half_spread: f32,  // in radians
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl FanGenerator{
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        up: glm::Vec3,
        spread: f32,  // in degrees
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self{
        let right = glm::normalize(&glm::cross(&dir, &up));
        Self { 
            source,
            dir: glm::normalize(&dir),
            up: glm::normalize(&glm::cross(&right, &dir)),
            right,
            // half degree in radians = degree / 2 * 2pi / 360 
            //  = degree * pi / 180 = degree * (pi/2) * 360
            half_spread: (spread) / 360.0 * FRAC_PI_2,
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for FanGenerator{
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32,
        halflife: f32,
        tex_range: (u32, u32),
        color: glm::Vec4,
        rng: &mut rand::rngs::ThreadRng,
    ) -> u32 {
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let ang_dir = Uniform::new(-self.half_spread, self.half_spread);
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        // let v = self.dir;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let lin_scale = lin_dist.sample(rng);
            let angle = ang_dir.sample(rng);
            let v: glm::Vec3 = (angle.cos() * self.dir + angle.sin() * self.right) * lin_scale;
            list.push(Particle {
                start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                color: color.into(),
                velocity: [
                    v[0] * lin_scale,
                    v[1] * lin_scale,
                    v[2] * lin_scale,
                    ang_dist.sample(rng),
                ],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as f32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                _pad2: 0.0,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        return list.len() as u32;
    }
}

pub struct LineGenerator {
    source: glm::Vec3,
    dir: glm::Vec3,
    linear_speed: f32,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    size_growth: f32,
    poisson_generation: bool,
}

impl LineGenerator {
    //// Line particle generator
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        linear_speed: f32,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        size_growth: f32,
        poisson_generation: bool,
    ) -> Self {
        Self {
            source,
            dir: glm::normalize(&dir),
            linear_speed,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            size_growth,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for LineGenerator {
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32,
        halflife: f32,
        tex_range: (u32, u32),
        color: glm::Vec4,
        rng: &mut rand::rngs::ThreadRng,
    ) -> u32 {
        let lin_dist = Normal::new(self.linear_speed, self.linear_variance).unwrap();
        let ang_dist = Normal::new(self.angular_velocity, self.angular_variance).unwrap();
        let size_dist = Normal::new(self.size, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0 / spawn_rate).unwrap();
        let mut spawn_time = 0.0;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time {
            let v = self.dir * lin_dist.sample(rng);
            list.push(Particle {
                start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                color: color.into(),
                velocity: [
                    v[0],
                    v[1],
                    v[2],
                    ang_dist.sample(rng),
                ],
                spawn_time,
                size: size_dist.sample(rng),
                tex_id: rng.gen_range(tex_range.0..tex_range.1) as f32,
                z_pos: 0.0,
                time_elapsed: 0.0,
                size_growth: self.size_growth,
                halflife,
                _pad2: 0.0,
            });
            spawn_time += match self.poisson_generation {
                true => time_dist.sample(rng),
                false => 1.0 / spawn_rate,
            };
        }
        return list.len() as u32;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PSRaw {
    lifetime: f32, // in seconds
    num_textures: f32,
}

pub struct ParticleSystem {
    start_time: std::time::Instant,
    particle_lifetime: f32,
    last_particle_death: std::time::Duration,
    particles: Vec<Particle>,
    num_instances: u32,
}

impl ParticleSystem {
    const EPSILON: f32 = 1e-6;

    pub fn new(
        generation_time: std::time::Duration,
        particle_lifetime: f32, // in seconds
        generation_speed: f32,  // measured per second
        color: glm::Vec4,
        gen: impl ParticleGenerator,
        tex_range: (u32, u32),
        device: &wgpu::Device,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Self {
        let mut particles = vec![];
        let num_instances = gen.generate(
            &mut particles,
            generation_time,
            generation_speed,
            particle_lifetime / 2.0,
            tex_range,
            color,
            rng,
        );
        // Time
        let last_particle_death =
            generation_time + std::time::Duration::from_secs_f32(particle_lifetime);
        println!("number of particles: {}", num_instances);
        println!(
            "last particle death: {:?}",
            last_particle_death.as_secs_f32()
        );
        Self {
            start_time: std::time::Instant::now(),
            particle_lifetime,
            last_particle_death,
            particles,
            num_instances,
        }
    }

    pub fn regen(mut self) -> Option<Self> {
        if self.start_time.elapsed() < self.last_particle_death {
            return Some(self);
        }
        //TODO: comtinued generation
        return None;
    }

    pub fn not_done(&self) -> bool {
        return self.start_time.elapsed() < self.last_particle_death;
    }
}

////
/// Should only have one in the entire client
/// This struct stores the buffers and layouts
/// to prevent (more) clutter in State::new()
/// systems: should always
pub struct ParticleDrawer {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    pub tex_bind_group_layout: wgpu::BindGroupLayout,
    tex_bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
    pub systems: Vec<ParticleSystem>,
}

impl ParticleDrawer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_layout: &wgpu::BindGroupLayout,
        texture: texture::Texture,
    ) -> Self {
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(PVertex),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(PInd),
            usage: wgpu::BufferUsages::INDEX,
        });
        let tex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("particle_texture_bind_group_layout"),
            });
        // texture bind group
        let tex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &tex_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: None,
        });
        // set up shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("particle_shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Render Pipeline Layout"),
                bind_group_layouts: &[camera_layout, &tex_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[PV_desc(), Particle::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        Self {
            vbuf,
            ibuf,
            tex_bind_group_layout,
            tex_bind_group,
            texture,
            render_pipeline,
            systems: vec![],
        }
    }

    pub fn get_particles_to_draw(
        &mut self,
        camera: &crate::camera::Camera,
        to_draw: &mut Vec<Particle>,
    ) {
        // remove dead systems
        self.systems = self.systems.drain(..).filter(|x| x.not_done()).collect();
        // Camera info
        let cam_dir: glm::Vec3 = glm::normalize(&(camera.position - camera.target));
        let cpos = &camera.position;
        for ps in &mut self.systems {
            // set elapsed time uniform
            let elapsed = ps.start_time.elapsed().as_secs_f32();

            // order instances first, then set instance buffer
            for p in &mut ps.particles {
                let pos: glm::Vec3 = glm::make_vec3(&p.start_pos[0..3])
                    + (elapsed - p.spawn_time) * glm::make_vec3(&p.velocity[0..3]);
                p.z_pos = glm::dot(&(pos - cpos), &cam_dir);
            }
            let start_ind = ps
                .particles
                .partition_point(|&a| a.spawn_time + ps.particle_lifetime < elapsed);
            ps.particles.drain(0..start_ind);
            let end_ind = ps.particles.partition_point(|&a| a.spawn_time < elapsed);
            for p in &mut ps.particles[..end_ind] {
                p.time_elapsed = elapsed;
            }
            to_draw.extend_from_slice(&ps.particles[..end_ind]);
        }
        // sort by depth
        to_draw.sort_by(|a, b| {
            a.z_pos
                .partial_cmp(&b.z_pos)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn draw<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        inst_num: u32,
        inst_buf: &'a wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &camera_bind_group, &[]);

        //set up constant buffers
        // 1. Vertex buffer
        render_pass.set_vertex_buffer(0, self.vbuf.slice(..));
        // 2. Index buffer
        render_pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint16);
        // 3. texture
        render_pass.set_bind_group(1, &self.tex_bind_group, &[]);

        // Set up instance buffer
        render_pass.set_vertex_buffer(1, inst_buf.slice(..));

        render_pass.draw_indexed(0..6, 0, 0..inst_num);
    }
}
