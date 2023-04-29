use instant::Duration;
use crate::texture;
extern crate nalgebra_glm as glm;

#[rustfmt::skip]
pub const PVertex : [[f32; 2];4] = [
    [0.0, 1.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
];

#[rustfmt::skip]
pub const Pind : [u16; 6] =[
    0, 2, 1,
    0, 3, 2,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
//// For particle instances
/// start_pos[3]: angle at the start
/// velocity[3]: angular velocity
/// color: color to tint the particle
/// spawn_time: relative to the time which the system was created
pub struct Particle{ 
    // use last f32 as angluar position/velocity
    start_pos: [f32; 4],
    velocity: [f32; 4],
    color: [f32; 4],
    spawn_time: f32,
    size: f32,
    tex_id: u32,
    _pad0: u32,
}

impl Particle{
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        1 => Float32x4, 2 => Float32x4, 3 => Float32x4,
        4 => Float32,   5 => Float32,   6 => Uint32,
        7 => Uint32,
    ];
}

impl crate::model::Vertex for Particle{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
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
    pub fn generate(
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
            angle: glm::radians(angle),
        }
    }
}

impl ParticleGenerator for ConeGenerator{
    fn generate(
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: u32, // per second
        num_textures: u32,
    ) -> u32{
        todo!();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PSRaw{
    lifetime: u32, // in ms
    num_textures: u32,
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
        particles = vec![];
        let num_instances = gen::generate(particles, generation_time, generation_speed);
        // instances don't change over time
        let inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // Uniform
        let meta = PSRaw{
            lifetime: particle_lifetime,
            num_textures,
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
                    binding: 0,
                    resource: meta_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        // Time
        let last_particle_death = std::time::Duration::from_millis(particles[particles.len() - 1].spawn_time + particle_lifetime);
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

    pub fn draw(){
        todo!();
    }
}