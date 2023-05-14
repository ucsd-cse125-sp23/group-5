use serde::{Deserialize, Serialize};
use nalgebra_glm as glm;
use wgpu::util::DeviceExt;

use crate::mesh_color::{MeshColor, MeshColorInstance};
use crate::screen::location::ScreenLocation;
use crate::screen::objects;

// NOTE: all units are relative to either screen height or screen width
//    with the exception of aspect ratio, which is unitless

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplay{
    pub displays: Vec<ConfigDisplayGroup>,
    pub default_display: String,
    pub game_display: String,
    pub screens: Vec<ConfigScreen>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplayGroup {
    pub id: String,
    pub screen: Option<String>,
    pub scene: Option<String>, // To load the scene graph
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreen {
    pub id: String,
    pub background: Option<ConfigScreenBackground>,
    pub buttons: Vec<ConfigButton>,
    pub icons: Vec<ConfigIcon>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreenBackground{
    pub tex: String,
    pub aspect: f32,
    pub color: Option<[f32; 3]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigButton{
    pub location: ScreenLocation,
    pub aspect: f32,    // both textures must be the same aspect ratio
    pub height: f32,    // relative to screen height
    pub default_tint: [f32; 4],
    pub hover_tint: [f32; 4],
    pub default_tex: String,
    pub hover_tex: String,
    pub color: Option<[f32; 3]>,
    pub on_click: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigIcon{
    pub location: ScreenLocation,
    pub aspect: f32,
    pub height: f32,
    pub tint: [f32; 4],
    pub tex: String,
    pub instances: Vec<ConfigScreenTransform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreenTransform{
    // Scale, then rotate, then translate
    // No shearing...
    pub translation: ScreenLocation,
    pub rotation: f32, // in radians, rotating CCW
    pub scale: (f32, f32),
}

impl ConfigDisplayGroup{
    pub fn unwrap_config(
        &self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
    ) -> objects::DisplayGroup{
        objects::DisplayGroup{
            id: self.id.clone(),
            screen: self.screen.clone(),
            scene: self.scene.clone(),
        }
    }
}

impl ConfigScreen{
    pub fn unwrap_config(
        &self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        color_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> objects::Screen{
        let background = match self.background.as_ref(){
            None => None,
            Some(bg) => Some(bg.unwrap_config(device, color_bind_group_layout))
        };
        let mut icons = Vec::new();
        for i in &self.icons{
            icons.push(i.unwrap_config(width, height, device));
        }
        let mut buttons = Vec::new();
        for b in &self.buttons{
            buttons.push(b.unwrap_config(width, height, device, color_bind_group_layout));
        }

        objects::Screen { 
            id: self.id.clone(),
            background,
            icons,
            buttons,
            default_color:  MeshColorInstance::new(device, color_bind_group_layout, MeshColor::default()),
        }
    }
}

impl ConfigScreenBackground{
    pub fn unwrap_config(
        &self,
        device: &wgpu::Device,
        color_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> objects::ScreenBackground{
        // vertex buffer
        let vertices = objects::TITLE_VERT;
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(" Some icon (implement ids for more useful messages!) Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        objects::ScreenBackground { 
            aspect: self.aspect,
            vbuf,
            texture: self.tex.clone(),
            color: match self.color {None => None, Some(c) => Some(MeshColorInstance::new(device, color_bind_group_layout, MeshColor::new(c)))},
        }
    }
}

impl ConfigButton{
    pub fn unwrap_config(
        &self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        color_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> objects::Button {
        // vertices + buffer
        let mut vertices = objects::TITLE_VERT;
        self.location.get_coords(
            self.aspect,
            self.height,
            width,
            height,
            &mut vertices
        );
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(" Some icon (implement ids for more useful messages!) Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // return
        objects::Button { 
            location: self.location,
            aspect: self.aspect,
            height: self.height,
            vertices,
            vbuf,
            default_tint: glm::make_vec4(&self.default_tint),
            hover_tint: glm::make_vec4(&self.hover_tint),
            default_texture: self.default_tex.clone(),
            hover_texture: self.hover_tex.clone(),
            color: match self.color{None => None, Some(c) => Some(MeshColorInstance::new(device, color_bind_group_layout, MeshColor::new(c)))},
            on_click: self.on_click.clone(),
        }
    } 
}

impl ConfigIcon{
    pub fn unwrap_config(
        &self,
        width: u32,
        height: u32,
        device: &wgpu::Device,
    ) -> objects::Icon {
        // vertices + buffer
        let mut vertices = objects::TITLE_VERT;
        self.location.get_coords(
            self.aspect,
            self.height,
            width,
            height,
            &mut vertices
        );
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(" Some icon (implement ids for more useful messages!) Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Instances + buffer
        let mut instances = Vec::new();
        for inst in &self.instances{
            instances.push(inst.unwrap_config(width, height));
        }
        let inst_range = 0..(instances.len() as u32);
        let inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(" Some icon (implement ids for more useful messages!) Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        objects::Icon { 
            location: self.location,
            aspect: self.aspect,
            height: self.height,
            vertices,
            vbuf,
            tint: glm::make_vec4(&self.tint),
            texture: self.tex.clone(),
            instances,
            inst_buf,
            inst_range
        }
    }
}

impl ConfigScreenTransform{
    pub fn unwrap_config(
        &self,
        width: u32,
        height: u32,
    ) -> objects::ScreenInstance {
        let mut inst: glm::Mat4 = glm::identity();
        glm::scale(&inst, &glm::vec3(self.scale.0, self.scale.1, 1.0));
        glm::rotate_z(&inst, self.rotation);
        let t = self.translation.to_absolute(width, height);
        glm::translate(&inst, &glm::vec3(t[0], t[1], 0.0));

        objects::ScreenInstance { 
            transform: inst.into()
        }
    }
}