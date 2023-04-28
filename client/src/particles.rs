use instant::Duration;
extern crate nalgebra_glm as glm;

#[rustfmt::skip]
const PVertex : [[f32; 2];4] = [
    [0.0, 1.0],
    [0.0, 0.0],
    [1.0, 0.0],
    [1.0, 1.0],
];

#[rustfmt::skip]
const Pind : [u16; 6] =[
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

impl crate::model::Vertex for Particle{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 13]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 14]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 15]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

pub trait ParticleGenerator{
    fn generate(list: &mut Vec<Particle>, n: u32);
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
    fn generate(list: &mut Vec<Particle>, n: u32){
        todo!();
    }
}

pub struct ParticleSystem{
    // start time, end time, lifetime, generation speed
    // current time
    // textures
    start_time: std::time::Instant,
    generation_time: std::time::Duration,
    last_particle_death: std::time::Duration,
    particle_lifetime: std::time::Duration,
    // TODO: poisson process?
    generation_speed: u32, // measured per second
    gen: impl ParticleGenerator,
    particles: Vec<Particle>,
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
    ) -> Self{
        todo!();
    }

    pub fn done(&self) -> bool{
        todo!();
    }

    pub fn draw(){
        todo!();
    }
}