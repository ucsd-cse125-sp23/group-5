use wgpu::util::DeviceExt;

// Vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}


impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

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
    transform:[[f32; 3] ; 3],
}

impl ScreenInstance{
    pub fn default() -> Self{
        Self { 
            transform: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0]
            ],
        }
    }
}

pub struct ScreenObject{
    pub vbuf : wgpu::Buffer,
    pub ibuf : wgpu::Buffer,
    pub num_indices : u32,
    pub instances : Vec<ScreenInstance>
}

impl ScreenObject{
    pub fn new(vtxs : &Vec<Vertex>, idxs : &Vec<u16>, device : &wgpu::Device) -> Self{
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
        Self { 
            vbuf,
            ibuf,
            num_indices: idxs.len() as u32,
            // TODO!
            instances: vec![ScreenInstance::default()],
        }
    }
}
