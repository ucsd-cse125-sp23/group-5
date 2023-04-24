use wgpu::util::DeviceExt;
use crate::texture;
use crate::resources;

// Vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub texture: [f32; 2],
}


impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct ScreenInstance{
    transform:[[f32; 4] ; 4],
}

impl ScreenInstance{
    pub fn default() -> Self{
        Self { 
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

pub struct ScreenObject{
    pub vbuf : wgpu::Buffer,
    pub ibuf : wgpu::Buffer,
    pub num_indices : u32,
    pub instances : Vec<ScreenInstance>,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl ScreenObject{
    pub async fn new(vtxs : &Vec<Vertex>, idxs : &Vec<u16>, tex_name: &str, 
        layout : &wgpu::BindGroupLayout,
        device : &wgpu::Device, queue: &wgpu::Queue) -> Self{
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Obj Vertex Buffer"),
            contents: bytemuck::cast_slice(vtxs),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(idxs),
            usage: wgpu::BufferUsages::INDEX,
        });

        //Assume there's always a texture
        let diffuse_texture = match resources::load_texture(tex_name, device, queue).await{
            Ok(tex) => tex,
            Err(e) => panic!("Failed to load screen texture {:?}\n", e),
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });
        Self { 
            vbuf,
            ibuf,
            num_indices: idxs.len() as u32,
            diffuse_texture,
            bind_group,
            // TODO!
            instances: vec![ScreenInstance::default()],
        }
    }
}
