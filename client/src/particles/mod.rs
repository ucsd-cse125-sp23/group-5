use crate::model::Vertex;
use crate::texture;

use nalgebra_glm as glm;
use wgpu::util::DeviceExt;

//exports
pub mod constants;
pub mod gen;
pub mod ribbon;
pub mod trail;

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
    pub start_pos: [f32; 4],
    pub velocity: [f32; 4],
    pub color: [f32; 4],
    pub normal_1: [f32; 4],
    pub normal_2: [f32; 4],
    pub spawn_time: f32,
    pub size: f32,
    pub tex_id: i32,
    pub z_pos: f32,
    pub time_elapsed: f32,
    pub size_growth: f32,
    pub halflife: f32,
    pub FLAG: u32,
}

impl Particle {
    const ATTRIBS: [wgpu::VertexAttribute; 13] = wgpu::vertex_attr_array![
        1 => Float32x4, 2 => Float32x4, 3 => Float32x4,
        4 => Float32x4, 5 => Float32x4, 6 => Float32 ,
        7 => Float32,   8 => Sint32,    9 => Float32,
        10 => Float32,   11 => Float32,  12 => Float32,
        13 => Uint32,
    ];

    pub fn dead_partition_pred(&self, lifetime: f32, elapsed: f32) -> bool {
        if self.FLAG == constants::POINT_PARTICLE {
            return self.spawn_time + lifetime < elapsed;
        } else {
            // since time 2 is stored in size
            return self.size + lifetime < elapsed;
        }
    }

    pub fn spawned_partition_pred(&self, lifetime: f32, elapsed: f32) -> bool {
        // return true;
        if self.FLAG == constants::POINT_PARTICLE {
            return self.spawn_time < elapsed;
        } else {
            return self.spawn_time < elapsed + lifetime;
        }
    }

    pub fn calc_z(&mut self, cam_dir: &glm::Vec3, cpos: &glm::Vec3, elapsed: f32) {
        if self.FLAG == constants::POINT_PARTICLE {
            let pos: glm::Vec3 = glm::make_vec3(&self.start_pos[0..3])
                + (elapsed - self.spawn_time) * glm::make_vec3(&self.velocity[0..3]);
            self.z_pos = glm::dot(&(pos - cpos), cam_dir);
        } else {
            let p1 = glm::make_vec3(&self.start_pos[0..3]);
            let p2 = glm::make_vec3(&self.velocity[0..3]);
            let z1 = glm::dot(&(p1 - cpos), cam_dir);
            let z2 = glm::dot(&(p2 - cpos), cam_dir);
            if z1 > z2 {
                self.z_pos = z1;
            } else {
                self.z_pos = z2;
            }
        }
    }
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
        gen: impl gen::ParticleGenerator,
        tex_range: (u32, u32),
        _device: &wgpu::Device,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Self {
        let mut particles = vec![];
        let last_particle_death = gen.generate(
            &mut particles,
            generation_time,
            generation_speed,
            particle_lifetime / 2.0,
            tex_range,
            color,
            rng,
        );
        // // Time
        // let mut last_particle_death =
        //     generation_time + std::time::Duration::from_secs_f32(particle_lifetime);
        // if particles[0].FLAG != constants::POINT_PARTICLE {
        //     last_particle_death =
        //         std::time::Duration::from_secs_f32(particle_lifetime + particles[particles.len()-1].size);
        // }
        // println!("number of particles: {}", num_instances);
        // println!(
        //     "last particle death: {:?}",
        //     last_particle_death.as_secs_f32()
        // );
        let num_instances = particles.len() as u32;
        Self {
            start_time: std::time::Instant::now(),
            particle_lifetime,
            last_particle_death: std::time::Duration::from_secs_f32(last_particle_death),
            particles,
            num_instances,
        }
    }

    pub fn regen(self) -> Option<Self> {
        if self.start_time.elapsed() < self.last_particle_death {
            return Some(self);
        }
        //TODO if necessary: continued generation
        None
    }

    pub fn not_done(&self) -> bool {
        self.start_time.elapsed() < self.last_particle_death
    }
}

////
/// Should only have one in the entire client
/// This struct stores the buffers and layouts
/// to prevent (more) clutter in State::new()
/// command_handlers: should always
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
            contents: bytemuck::cast_slice(constants::VTXS),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(constants::INDS),
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
                            view_dimension: wgpu::TextureViewDimension::D2Array,
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
                buffers: &[constants::vtx_desc(), Particle::desc()],
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
        // remove dead command_handlers
        self.systems = self.systems.drain(..).filter(|x| x.not_done()).collect();
        // Camera info
        let cam_dir: glm::Vec3 = glm::normalize(&(camera.position - camera.target));
        let cpos = &camera.position;
        for ps in &mut self.systems {
            // set elapsed time uniform
            let elapsed = ps.start_time.elapsed().as_secs_f32();

            // order instances first, then set instance buffer
            for p in &mut ps.particles {
                p.calc_z(&cam_dir, cpos, elapsed);
            }
            let start_ind = ps
                .particles
                .partition_point(|&a| a.dead_partition_pred(ps.particle_lifetime, elapsed));
            ps.particles.drain(0..start_ind);
            let end_ind = ps
                .particles
                .partition_point(|&a| a.spawned_partition_pred(ps.particle_lifetime, elapsed));
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
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);

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
