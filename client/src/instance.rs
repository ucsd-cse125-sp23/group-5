extern crate nalgebra_glm as glm;
use common::core::mesh_color::MeshColor;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub type Transform = nalgebra_glm::TMat4<f32>;

// Instances
// Lets us duplicate objects in a scene with less cost
#[derive(Debug, Clone)]
pub struct Instance {
    pub transform: Transform,
    pub mesh_colors: Option<HashMap<String, MeshColor>>,
    pub chosen_materials: Option<HashMap<String, String>>,
}

pub struct InstanceState {
    pub data: Vec<InstanceRaw>,
    pub buffer: wgpu::Buffer,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            transform: glm::identity(),
            mesh_colors: None,
            chosen_materials: None,
        }
    }
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let tmp = glm::inverse_transpose(self.transform);
        let normal = glm::mat3(
            tmp[0], tmp[4], tmp[8], tmp[1], tmp[5], tmp[9], tmp[2], tmp[6], tmp[10],
        );
        InstanceRaw {
            model: self.transform.into(),
            normal: normal.into(),
        }
    }

    pub fn from_transform(transform: &nalgebra_glm::TMat4<f32>) -> Self {
        Self {
            transform: *transform,
            mesh_colors: None,
            chosen_materials: None,
        }
    }

    pub fn make_buffer(instances: &[Instance], device: &wgpu::Device) -> InstanceState {
        let data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        InstanceState { data, buffer }
    }

    pub fn make_buffers(instances: &Vec<Instance>, device: &wgpu::Device) -> Vec<InstanceState> {
        let mut instance_states = Vec::new();
        for instance in instances.iter() {
            let data = instance.to_raw();
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::VERTEX,
            });
            instance_states.push(InstanceState {
                data: vec![data],
                buffer,
            });
        }
        instance_states
    }

    // pub fn add_color(&mut self, colors: HashMap<String, MeshColor>) {
    //     self.mesh_colors = Some(colors);
    // }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl crate::model::Vertex for InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
