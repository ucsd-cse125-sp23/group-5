
use std::collections::btree_map::Range;

use crate::model;

use crate::resources;
use crate::texture;
use wgpu::util::DeviceExt;

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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenInstance {
    pub transform: [[f32; 4]; 4],
}

impl ScreenInstance {
    pub fn default() -> Self {
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

impl crate::model::Vertex for ScreenInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ScreenInstance>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct ScreenObject {
    pub vbuf: wgpu::Buffer,
    pub ibuf: wgpu::Buffer,
    pub num_indices: u32,
    pub num_inst: u32,
    pub instances: Vec<ScreenInstance>,
    pub inst_buf: wgpu::Buffer,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl ScreenObject {
    pub async fn new(
        vtxs: &Vec<Vertex>,
        idxs: &Vec<u16>,
        instances: Vec<ScreenInstance>,
        tex_name: &str,
        layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
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

        let num_inst = instances.len() as u32;
        let inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        //Assume there's always a texture
        let diffuse_texture = match resources::load_texture(tex_name, device, queue).await {
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
            num_inst,
            diffuse_texture,
            bind_group,
            // TODO!
            instances,
            inst_buf,
        }
    }
}

pub struct Screen {
    pub name: String,
    pub objects: Vec<ScreenObject>,
    pub ranges: Vec<std::ops::Range<u32>>,
}

pub async fn get_screens(
    texture_bind_group_layout_2d: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Vec<Screen> {
    #[rustfmt::skip]
    let rect_indices : Vec<u16> = vec![
        0, 2, 1,
        0, 3, 2,
    ];

    let title_vert: Vec<Vertex> = vec![
        Vertex {
            position: [-1.0, -1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.0, 1.0],
        }, // A
        Vertex {
            position: [-1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.0, 0.0],
        }, // B
        Vertex {
            position: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [1.0, 0.0],
        }, // C
        Vertex {
            position: [1.0, -1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [1.0, 1.0],
        }, // D
    ];
    #[rustfmt::skip]
    let title_inst = vec![
        ScreenInstance{
            transform: [[1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],]
        },
    ];
    let title_inst_num = title_inst.len() as u32;
    let title_hover_inst = vec![ScreenInstance {
        transform: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }];

    let _title_hover_inst_num = title_inst.len() as u32;

    #[rustfmt::skip]
    let atk_bx_vert : Vec<Vertex> = vec![
        Vertex { position: [-0.90, 0.75], color: [1.0, 1.0, 1.0, 0.9], texture: [0.0, 1.0] }, // A
        Vertex { position: [-0.90, 0.90], color: [1.0, 1.0, 1.0, 0.9], texture: [0.0, 0.0] }, // B
        Vertex { position: [-0.30, 0.90], color: [1.0, 1.0, 1.0, 0.9], texture: [1.0, 0.0] }, // C
        Vertex { position: [-0.30, 0.75], color: [1.0, 1.0, 1.0, 0.9], texture: [1.0, 1.0] }, // D
    ];

    let atk_itm_vert: Vec<Vertex> = vec![
        Vertex {
            position: [-0.88, 0.75],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.0, 1.0],
        }, // A
        Vertex {
            position: [-0.88, 0.90],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.0, 0.0],
        }, // B
        Vertex {
            position: [-0.78, 0.90],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [1.0, 0.0],
        }, // C
        Vertex {
            position: [-0.78, 0.75],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [1.0, 1.0],
        }, // D
    ];

    #[rustfmt::skip]
    let atk_bx_inst = vec![
        ScreenInstance{
            transform: [[1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],]
        },
    ];
    let atk_bx_inst_num = atk_bx_inst.len() as u32;
    let atk_itm_inst = vec![
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.065, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.130, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.195, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.260, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.325, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.390, 0.0, 0.0, 1.0],
            ],
        },
        ScreenInstance {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.455, 0.0, 0.0, 1.0],
            ],
        },
    ];
    let atk_itm_inst_num = atk_itm_inst.len() as u32;

    let title_obj = ScreenObject::new(
        &title_vert,
        &rect_indices,
        title_inst,
        "start_screen_default.jpg",
        texture_bind_group_layout_2d,
        device,
        queue,
    )
    .await;

    let title_hover_obj = ScreenObject::new(
        &title_vert,
        &rect_indices,
        title_hover_inst,
        "start_screen_hover.jpg",
        texture_bind_group_layout_2d,
        device,
        queue,
    )
    .await;

    let atk_box_obj = ScreenObject::new(
        &atk_bx_vert,
        &rect_indices,
        atk_bx_inst,
        "back1.png",
        texture_bind_group_layout_2d,
        device,
        queue,
    )
    .await;

    let atk_itm_obj = ScreenObject::new(
        &atk_itm_vert,
        &rect_indices,
        atk_itm_inst,
        "wind.png",
        texture_bind_group_layout_2d,
        device,
        queue,
    )
    .await;

    vec![
        Screen {
            name: String::from("Title Screen"),
            objects: vec![title_obj, title_hover_obj],
            ranges: vec![(0..title_inst_num), (0..0)],
        },
        Screen {
            name: String::from("Playing Screen"),
            objects: vec![atk_box_obj, atk_itm_obj],
            ranges: vec![(0..atk_bx_inst_num), (0..atk_itm_inst_num)],
        },
    ]
}

// only for title screen right now
pub fn update_screen(width: u32, height: u32, device: &wgpu::Device, screen: &mut ScreenObject) {
    let aspect: f32 = (width as f32) / (height as f32);
    const title_ar: f32 = 16.0 / 9.0;
    let title_x_span_half = (glm::clamp_scalar(aspect / title_ar, 0.0, 1.0)) / 2.0;
    let title_vert: Vec<Vertex> = vec![
        Vertex {
            position: [-1.0, -1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.5 - title_x_span_half, 1.0],
        }, // A
        Vertex {
            position: [-1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.5 - title_x_span_half, 0.0],
        }, // B
        Vertex {
            position: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.5 + title_x_span_half, 0.0],
        }, // C
        Vertex {
            position: [1.0, -1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            texture: [0.5 + title_x_span_half, 1.0],
        }, // D
    ];
    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Screen Obj Vertex Buffer"),
        contents: bytemuck::cast_slice(&title_vert),
        usage: wgpu::BufferUsages::VERTEX,
    });
    screen.vbuf = vbuf;
}
