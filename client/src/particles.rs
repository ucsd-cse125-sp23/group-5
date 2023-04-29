use wgpu::util::DeviceExt;
use crate::texture;
use crate::model::Vertex;
extern crate nalgebra_glm as glm;
use std::f32::consts::FRAC_PI_2;
use std::ops::Not;
// use rand_distr::Distribution;
// use rand::Rng;

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
/// spawn_time: relative to the time which the system was created
/// size: radius in cm
pub struct Particle{ 
    // use last f32 as angluar position/velocity
    start_pos: [f32; 4],
    velocity: [f32; 4],
    color: [f32; 4],
    spawn_time: u32,
    size: f32,
    tex_id: f32,
    _pad0: u32,
}

impl Particle{
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        1 => Float32x4, 2 => Float32x4, 3 => Float32x4,
        4 => Uint32 ,   5 => Float32,   6 => Float32,
        7 => Uint32,
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
        spawn_rate: u32,
        num_textures: u32,
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
        spawn_rate: u32, // per second
        num_textures: u32,
    ) -> u32{
        todo!();
    }
}
pub struct LineGenerator{
    source: glm::Vec3,
    dir: glm::Vec3,
}

impl LineGenerator{
    //// Line particle generator
    pub fn new(source: glm::Vec3, dir: glm::Vec3) -> Self{
        Self{
            source,
            dir,
        }
    }
}

impl ParticleGenerator for LineGenerator{
    fn generate(
            &self,
            list: &mut Vec<Particle>,
            spawning_time: std::time::Duration,
            spawn_rate: u32,
            num_textures: u32,
        ) -> u32 {
        let n : u32 = (spawning_time.as_millis() * (spawn_rate as u128) / 1000) as u32;
        // let dist = rand_distr::Normal::new(1.0, 0.2).unwrap();
        // let mut rng = rand::thread_rng();
        for i in 0..n{
            let v = self.dir; //* dist.sample(&mut rng);
            list.push(
                Particle {
                    start_pos: [self.source[0], self.source[1], self.source[2], 0.0],
                    color: [1.0, 1.0, 1.0, 1.0], //TODO
                    velocity:  [v[0], v[1], v[2], 0.0],
                    spawn_time: (i * ((1000.0 / (spawn_rate as f32)).floor() as u32)),
                    size: 3.0,
                    tex_id: 0.0,
                    _pad0: 0,
                }
            );
        }
        return n;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PSRaw{
    lifetime: u32, // in ms
    num_textures: f32,
}

pub struct ParticleSystem{
    // textures - texture, bind group, number of particle types
    start_time: std::time::Instant,
    last_particle_death: std::time::Duration,
    // TODO: poisson process?
    particles: Vec<Particle>,
    num_instances: u32,
    inst_buf: wgpu::Buffer,
    tex_bind_group: wgpu::BindGroup,
}

impl ParticleSystem{
    pub fn new(
        generation_time: std::time::Duration,
        particle_lifetime: u32, // in milliseconds
        generation_speed: u32, // measured per second
        gen: impl ParticleGenerator,
        texture: &texture::Texture,
        tex_layout: &wgpu::BindGroupLayout,
        num_textures: u32,
        device: &wgpu::Device,
    ) -> Self{
        let mut particles = vec![];
        let num_instances = gen.generate(&mut particles, generation_time, generation_speed, num_textures);
        // instances don't change over time
        let inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::VERTEX,
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
        let last_particle_death = std::time::Duration::from_millis((particles[particles.len() - 1].spawn_time + particle_lifetime) as u64);
        println!("number of particles: {}", num_instances);
        println!("last particle death: {:?}", last_particle_death.as_millis());
        Self{
            start_time: std::time::Instant::now(),
            last_particle_death,
            particles,
            num_instances,
            inst_buf,
            tex_bind_group,
        }
    }

    pub fn done(&self) -> bool{
        return self.start_time.elapsed() >= self.last_particle_death;
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
        let elapsed = 0;
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ){
        // remove dead systems
        self.systems = self.systems.drain(..).filter(|x| x.done().not()).collect();
        render_pass.set_pipeline(&self.render_pipeline);

        for ps in &self.systems{
            render_pass.set_vertex_buffer(0, self.vbuf.slice(..));
            render_pass.set_vertex_buffer(1, ps.inst_buf.slice(..)); 
            render_pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(1, &ps.tex_bind_group, &[]);

            let elapsed = ps.start_time.elapsed().as_millis() as u32;
            queue.write_buffer(&self.tbuf, 0, bytemuck::cast_slice(&[elapsed]));
            render_pass.set_bind_group(2, &self.t_bind_group, &[]);

            render_pass.draw_indexed(0..6, 0, 0..ps.num_instances);
        }
    }
}