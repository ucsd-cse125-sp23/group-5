use std::ops::Range;

use crate::instance;
use crate::resources::load_model;
use crate::texture;

pub struct Model {
    pub path: String,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub struct InstancedModel<'a> {
    // want instances and instanceState to always be synced
    // can only be created, cannot be edited
    // TODO: enforce that somehow?
    pub model: &'a Model,
    pub num_instances: usize,
    pub instance_state: instance::InstanceState,
}

impl<'a> InstancedModel<'a> {
    pub fn new(
        model: &'a Model,
        instances: &Vec<instance::Instance>,
        device: &wgpu::Device,
    ) -> Self {
        Self {
            model,
            num_instances: instances.len(),
            instance_state: instance::Instance::make_buffer(instances, device),
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Phong {
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub shininess: [f32; 4],
}

impl Phong {
    pub fn default() -> Self {
        Self {
            ambient: [0.0; 4],
            diffuse: [0.0; 4],
            specular: [0.0; 4],
            shininess: [0.0; 4],
        }
    }

    pub fn new(m: &tobj::Material) -> Self {
        Self {
            ambient: [m.ambient[0], m.ambient[1], m.ambient[2], 1.0],
            diffuse: [m.diffuse[0], m.diffuse[1], m.diffuse[2], 1.0],
            specular: [m.specular[0], m.specular[1], m.specular[2], 1.0],
            shininess: [m.shininess, 0.0, 0.0, 0.0],
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderFlags {
    flags: u32,
}

impl ShaderFlags {
    pub const HAS_DIFFUSE_TEXTURE: u32 = 1;
    pub fn new(flags: u32) -> Self {
        Self { flags }
    }
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub phong_mtl: Phong,
    pub flags: ShaderFlags,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub trait DrawModel<'a> {
    fn draw_model(
        &mut self,
        instanced_model: &'a InstancedModel,
        camera_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        instanced_model: &'a InstancedModel,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_model(
        &mut self,
        instanced_model: &'a InstancedModel,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(instanced_model, 0..1, camera_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        instanced_model: &'a InstancedModel,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(1, instanced_model.instance_state.buffer.slice(..));
        for mesh in &instanced_model.model.meshes {
            // assume each mesh has a material
            let mat_id = mesh.material;
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.set_bind_group(0, &instanced_model.model.materials[mat_id].bind_group, &[]);
            // print!("model:154 {:?}\n", &instanced_model.model.materials[mat_id]);
            self.set_bind_group(1, camera_bind_group, &[]);
            self.draw_indexed(0..mesh.num_elements, 0, instances.clone());
        }
    }
}
