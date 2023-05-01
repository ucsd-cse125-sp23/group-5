use wgpu::util::DeviceExt;
use crate::texture;
use crate::model::Vertex;
extern crate nalgebra_glm as glm;
use std::{f32::consts::{FRAC_PI_2, PI}, ops::IndexMut};
// use rand_distr::Distribution;
use rand::Rng;
use rand_distr::{Normal, Distribution, Poisson};

#[rustfmt::skip]
pub const PVertex : &[[f32; 2];4] = &[
    [0.0, 1.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
];

const PV_ATTRIBS : [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
    0 => Float32x2,
];

fn PV_desc<'a>() -> wgpu::VertexBufferLayout<'a>{
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
pub struct Particle{ 
    // use last f32 as angluar position/velocity
    start_pos: [f32; 4],
    velocity: [f32; 4],
    color: [f32; 4],
    spawn_time: f32,
    size: f32,
    tex_id: f32,
    z_pos: f32,
}

impl Particle{
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        1 => Float32x4, 2 => Float32x4, 3 => Float32x4,
        4 => Float32 ,  5 => Float32,   6 => Float32,
        7 => Float32,
    ];
}

impl crate::model::Vertex for Particle{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait ParticleGenerator{
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
        num_textures: u32,
        rng: &mut rand::rngs::ThreadRng,
    ) -> u32;
}

pub struct ConeGenerator{
    source: glm::Vec3,
    dir: glm::Vec3,
    //angle should be in radians
    angle: f32,
}

impl ConeGenerator{
    //// Cone particle generator
    /// Arguments: angle: in degrees, the spread for the cone
    pub fn new(source: glm::Vec3, dir: glm::Vec3, angle: f32) -> Self{
        Self{
            source,
            dir,
            angle: angle * FRAC_PI_2 / 180.0,
        }
    }
}

impl ParticleGenerator for ConeGenerator{
    fn generate(
        &self,
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: f32, // per second
        num_textures: u32,
        rng: &mut rand::rngs::ThreadRng,
    ) -> u32{
        todo!();
    }
}
pub struct LineGenerator{
    source: glm::Vec3,
    dir: glm::Vec3,
    linear_variance: f32,
    angular_velocity: f32,
    angular_variance: f32,
    size: f32,
    size_variance: f32,
    poisson_generation: bool,
}

impl LineGenerator{
    //// Line particle generator
    pub fn new(
        source: glm::Vec3,
        dir: glm::Vec3,
        linear_variance: f32,
        angular_velocity: f32,
        angular_variance: f32,
        size: f32,
        size_variance: f32,
        poisson_generation: bool,
    ) -> Self{
        Self{
            source,
            dir,
            linear_variance,
            angular_velocity,
            angular_variance,
            size,
            size_variance,
            poisson_generation,
        }
    }
}

impl ParticleGenerator for LineGenerator{
    fn generate(
            &self,
            list: &mut Vec<Particle>,
            spawning_time: std::time::Duration,
            spawn_rate: f32,
            num_textures: u32,
            rng: &mut rand::rngs::ThreadRng,
        ) -> u32 {
        // let n : u32 = (spawning_time.as_secs_f32() * spawn_rate).floor() as u32;
        let lin_dist = Normal::new(1.0, self.linear_variance).unwrap();
        let ang_dist = Normal::new(1.0, self.angular_variance).unwrap();
        let size_dist = Normal::new(1.0, self.size_variance).unwrap();
        let time_dist = Poisson::new(1.0/spawn_rate).unwrap();
        let v = self.dir;
        let mut spawn_time = 0.0;
        while std::time::Duration::from_secs_f32(spawn_time) < spawning_time{
            let linear_scale = lin_dist.sample(rng);
            let angular_scale = ang_dist.sample(rng);
            let size_scale = size_dist.sample(rng);
            // want velocity to be nonzero
            glm::clamp_scalar(linear_scale, ParticleSystem::EPSILON, f32::INFINITY);
            glm::clamp_scalar(angular_scale, ParticleSystem::EPSILON, f32::INFINITY);
            glm::clamp_scalar(size_scale, ParticleSystem::EPSILON, f32::INFINITY);
            list.push(
                Particle {
                    start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                    color: [1.0, 1.0, 1.0, 1.0], //TODO
                    velocity:  [v[0] * linear_scale, v[1] * linear_scale, v[2] * linear_scale, self.angular_velocity * angular_scale],
                    spawn_time,
                    size: self.size * size_scale,
                    tex_id: rng.gen_range(0..num_textures) as f32,
                    z_pos: 0.0,
                }
            );
            spawn_time += match self.poisson_generation{
                true => time_dist.sample(rng),
                false => 1.0/spawn_rate,
            };
        }
        return list.len() as u32;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PSRaw{
    lifetime: f32, // in seconds
    num_textures: f32,
}

pub struct ParticleSystem{
    start_time: std::time::Instant,
    particle_lifetime: f32,
    last_particle_death: std::time::Duration,
    particles: Vec<Particle>,
    num_instances: u32,
    inst_buf: wgpu::Buffer,
    tex_bind_group: wgpu::BindGroup,
}

impl ParticleSystem{
    const EPSILON: f32 = 1e-6;

    pub fn new(
        generation_time: std::time::Duration,
        particle_lifetime: f32, // in seconds
        generation_speed: f32, // measured per second
        gen: impl ParticleGenerator,
        texture: &texture::Texture,
        tex_layout: &wgpu::BindGroupLayout,
        num_textures: u32,
        device: &wgpu::Device,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Self{
        let mut particles = vec![];
        let num_instances = gen.generate(&mut particles, generation_time, generation_speed, num_textures, rng);
        // instances don't change over time
        let inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        // Uniform
        let meta = PSRaw{
            lifetime: particle_lifetime,
            num_textures: num_textures as f32,
        };
        let meta_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Meta Buffer"),
            contents: bytemuck::cast_slice(&[meta]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // texture bind group
        let tex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: meta_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        // Time
        let last_particle_death = generation_time + std::time::Duration::from_secs_f32(particle_lifetime);
        println!("number of particles: {}", num_instances);
        println!("last particle death: {:?}", last_particle_death.as_secs_f32());
        Self{
            start_time: std::time::Instant::now(),
            particle_lifetime,
            last_particle_death,
            particles,
            num_instances,
            inst_buf,
            tex_bind_group,
        }
    }

    pub fn regen(mut self) -> Option<Self>{
        if self.start_time.elapsed() < self.last_particle_death {
            return Some(self);
        }
        //TODO: comtinued generation
        return None;
    }

    pub fn not_done(&self) -> bool{
        return self.start_time.elapsed() < self.last_particle_death;
    }
}

////
/// Should only have one in the entire client
/// This struct stores the buffers and layouts 
/// to prevent (more) clutter in State::new()
/// systems: should always
pub struct ParticleDrawer{
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    tbuf: wgpu::Buffer,
    t_bind_group_layout: wgpu::BindGroupLayout,
    t_bind_group: wgpu::BindGroup,
    pub tex_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    pub systems: Vec<ParticleSystem>,
}

impl ParticleDrawer{
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_layout: &wgpu::BindGroupLayout) -> Self{
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
        let elapsed: f32  = 0.0;
        let tbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Elapsed Buffer"),
            contents: bytemuck::cast_slice(&[elapsed]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let t_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("time_elapsed_bind_group_layout"),
            });
        let t_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &t_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: tbuf.as_entire_binding(),
                }],
                label: Some("time_elapsed_bind_group"),
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("particle_texture_bind_group_layout"),
            });
        // set up shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("particle_shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera_layout,
                    &tex_bind_group_layout,
                    &t_bind_group_layout,
                ],
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
            tbuf,
            t_bind_group_layout,
            t_bind_group,
            tex_bind_group_layout,
            render_pipeline,
            systems: vec![],
        }
    }

    pub fn draw<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        camera: &crate::camera::Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ){
        // remove dead systems
        self.systems = self.systems.drain(..).filter_map(|x| x.regen()).collect();
        render_pass.set_pipeline(&self.render_pipeline);
        let cam_dir: glm::Vec3 = glm::normalize(&(camera.position - camera.target));
        let cpos = &camera.position;

        for ps in &mut self.systems{
            render_pass.set_vertex_buffer(0, self.vbuf.slice(..));
            // set elapsed time uniform
            let elapsed = ps.start_time.elapsed().as_secs_f32();
            queue.write_buffer(&self.tbuf, 0, bytemuck::cast_slice(&[elapsed]));
            render_pass.set_bind_group(2, &self.t_bind_group, &[]);

            // order instances first, then set instance buffer
            for p in &mut ps.particles{
                let pos: glm::Vec3 = glm::make_vec3(&p.start_pos[0..3]) 
                    + (elapsed - p.spawn_time) * glm::make_vec3(&p.velocity[0..3]);
                p.z_pos = glm::dot(&(pos - cpos), &cam_dir);
            }
            let start_ind = match ps.particles.binary_search_by(
                    |&a| (a.spawn_time + ps.particle_lifetime).partial_cmp(&elapsed).expect("Unexpected Nan")
                ) {
                Ok(ind) => ind + 1,
                Err(ind) => ind,
            };
            ps.particles.drain(0..start_ind);
            let end_ind = match ps.particles.binary_search_by(
                    |&a| a.spawn_time.partial_cmp(&elapsed).expect("Unexpected Nan")
                ) {
                Ok(ind) => ind + 1,
                Err(ind) => ind,
            };
            let mut tmp = ps.particles[0..end_ind].to_vec();
            tmp.sort_by(|a, b | {
                a.z_pos.partial_cmp(&b.z_pos).unwrap_or(std::cmp::Ordering::Equal)
            });
            queue.write_buffer(&ps.inst_buf, 0, bytemuck::cast_slice(&tmp));
            render_pass.set_vertex_buffer(1, ps.inst_buf.slice(..)); 
            // set rest of the buffers 
            render_pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(1, &ps.tex_bind_group, &[]);

            render_pass.draw_indexed(0..6, 0, 0..(end_ind as u32));
            // println!("Num particles to draw: {}", tmp.len());
        }
    }
}