use crate::instance::Instance;
use common::core::mesh_color::{MeshColor, MeshColorInstance};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::sync::Arc;
use wgpu::Device;

use crate::instance;
use crate::resources::{load_model, ModelLoadingResources};
use crate::texture;

pub trait Model: Any + Debug {
    fn meshes(&self) -> &[Mesh];
    fn materials(&self) -> &[Material];
    fn mat_ind(&self) -> Option<&ahash::AHashMap<String, usize>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn Model>;
}

#[derive(Clone)]
pub struct StaticModel {
    pub path: String,
    pub meshes: Arc<Vec<Mesh>>,
    pub materials: Arc<Vec<Material>>,
    pub mat_ind: Option<Arc<ahash::AHashMap<String, usize>>>
}

impl Debug for StaticModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticModel")
            .field("path", &self.path)
            .field("meshes_len", &self.meshes.len())
            .field("materials", &self.materials)
            .finish()
    }
}

impl Model for StaticModel {
    fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    fn materials(&self) -> &[Material] {
        &self.materials
    }

    fn mat_ind(&self) -> Option<&ahash::AHashMap<String, usize>> {
        match &self.mat_ind {
            None => None,
            Some(s) =>  Some(&*s)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Model> {
        Box::new(self.clone())
    }
}

impl StaticModel {
    pub async fn load(file_path: &str, res: ModelLoadingResources<'_>) -> anyhow::Result<Self> {
        load_model(file_path, res).await
    }
}

pub struct InstancedModel {
    // want instances and instanceState to always be synced
    // can only be created, cannot be edited
    // TODO: enforce that somehow?
    pub model: Box<dyn Model>,
    pub num_instances: usize,
    // pub instance_state: instance::InstanceState,
    pub instance_states: Vec<instance::InstanceState>,
    pub mesh_colors: Vec<HashMap<String, MeshColorInstance>>,
    pub chosen_mats: Vec<Option<HashMap<String, String>>>,
    pub default_color: MeshColorInstance,
}

impl InstancedModel {
    pub fn new(
        model: Box<dyn Model>,
        instances: &Vec<Instance>,
        device: &Device,
        color_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            model,
            num_instances: instances.len(),
            // instance_state: Instance::make_buffer(instances, device),
            instance_states: instance::Instance::make_buffers(instances, device),
            mesh_colors: instances
                .iter()
                .map(|x| x.mesh_colors.clone())
                .collect::<Vec<_>>()
                .iter()
                .map(|x| to_mesh_color_inst(x, device, color_bind_group_layout))
                .collect::<Vec<_>>(),
            chosen_mats:instances
                .iter()
                .map(|x| x.chosen_materials.clone())
                .collect::<Vec<_>>(),
            default_color: MeshColorInstance::new(
                device,
                color_bind_group_layout,
                MeshColor::default(),
            ),
        }
    }
}

pub fn to_mesh_color_inst(
    map: &Option<HashMap<String, MeshColor>>,
    device: &wgpu::Device,
    color_bind_group_layout: &wgpu::BindGroupLayout,
) -> HashMap<String, MeshColorInstance> {
    let mut mci = HashMap::new();
    match map {
        Some(colors) => {
            for (mesh_name, mesh_color) in colors.iter() {
                mci.insert(
                    mesh_name.clone(),
                    MeshColorInstance::new(device, color_bind_group_layout, *mesh_color),
                );
            }
        }
        None => {}
    }
    mci
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
    pub const HAS_AMBIENT_TEXTURE: u32 = 2;
    pub const HAS_SPECULAR_TEXTURE: u32 = 4;
    pub const HAS_NORMAL_TEXTURE: u32 = 8;
    pub const HAS_SHININESS_TEXTURE: u32 = 16;

    pub fn new(flags: u32) -> Self {
        Self { flags }
    }
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub specular_texture: texture::Texture,
    pub ambient_texture: texture::Texture,
    pub shininess_texture: texture::Texture,
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
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl ModelVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32x2, 2 => Float32x3,
        3 => Float32x3, 4 => Float32x3,
    ];
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait DrawModel<'a, 'b> {
    fn draw_model(
        &mut self,
        instanced_model: &'a InstancedModel,
        camera_bind_group: &'b wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        instanced_model: &'a InstancedModel,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a>
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
        _instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        for j in 0..instanced_model.instance_states.len() {
            let instance_state = &instanced_model.instance_states[j];
            self.set_vertex_buffer(1, instance_state.buffer.slice(..));
            for i in 0..instanced_model.model.meshes().len() {
                let mesh_name = &instanced_model.model.meshes()[i].name;
                
                // assume each mesh has a material
                let mut mat_id = instanced_model.model.meshes()[i].material;
                if let Some(mtls) = &instanced_model.chosen_mats[j] {
                    if let Some(m) = mtls.get(mesh_name) {
                        if let Some(hm) = instanced_model.model.mat_ind() {
                            mat_id = match hm.get(m) {
                                None => mat_id,
                                Some(i) => *i,
                            };
                        }
                    }
                }
               
                self.set_vertex_buffer(
                    0,
                    instanced_model.model.meshes()[i].vertex_buffer.slice(..),
                );
                self.set_index_buffer(
                    instanced_model.model.meshes()[i].index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                match instanced_model.mesh_colors[j].get(mesh_name) {
                    Some(color) => {
                        self.set_bind_group(3, &color.color_bind_group, &[]);
                    }
                    None => {
                        self.set_bind_group(
                            3,
                            &instanced_model.default_color.color_bind_group,
                            &[],
                        );
                    }
                }

                self.set_bind_group(
                    0,
                    &instanced_model.model.materials()[mat_id].bind_group,
                    &[],
                );
                // print!("model:154 {:?}\n", &instanced_model.model.materials[mat_id]);
                self.set_bind_group(1, camera_bind_group, &[]);
                self.draw_indexed(0..instanced_model.model.meshes()[i].num_elements, 0, 0..1);
            }
        }
    }
}
