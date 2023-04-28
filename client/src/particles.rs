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
    pub fn generate(
        list: &mut Vec<Particle>,
        spawning_time: std::time::Duration,
        spawn_rate: u32,
        num_textures: u32,
    );
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
        spawn_rate: u32,
        num_textures: u32,
    ){
        todo!();
    }
}

pub struct ParticleSystem{
    // textures - texture, bind group, number of particle types
    start_time: std::time::Instant,
    generation_time: std::time::Duration,
    last_particle_death: std::time::Duration,
    particle_lifetime: std::time::Duration,
    // TODO: poisson process?
    generation_speed: u32, // measured per second
    gen: impl ParticleGenerator,
    particles: Vec<Particle>,
    texture: texture::Texture,
    num_textures: u32,
    bufs: Vec<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
}

impl ParticleSystem{
    pub fn new(
        start_time: std::time::Instant,
        generation_time: std::time::Duration,
        last_particle_death: std::time::Duration,
        particle_lifetime: std::time::Duration,
        generation_speed: u32, // measured per second
        gen: impl ParticleGenerator,
        texture: texture::Texture,
        num_textures: u32,
    ) -> Self{
        particles = vec![];
        gen::generate(particles, generation_time, generation_speed);

    }

    pub fn done(&self) -> bool{
        todo!();
    }

    pub fn draw(){
        todo!();
    }
}