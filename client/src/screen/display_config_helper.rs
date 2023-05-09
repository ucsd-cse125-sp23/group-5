use nalgebra_glm as glm;
use wgpu::util::DeviceExt;
use common::configs::display_config::{ConfigButton, ConfigIcon, ConfigScreenBackground, ConfigScreenTransform, ScreenLocation};

use crate::screen::objects;

impl ConfigDisplayGroup {
    pub fn unwrap_config(
        &self,
    ) -> objects::DisplayGroup {
        objects::DisplayGroup {
            id: self.id.clone(),
            screen: self.screen.clone(),
            scene: self.scene.clone(),
        }
    }
}

impl ConfigScreen {
    pub fn unwrap_config(&self, width: u32, height: u32, device: &wgpu::Device) -> objects::Screen {
        let background = match self.background.as_ref() {
            None => None,
            Some(bg) => Some(bg.unwrap_config(device)),
        };
        let mut icons = Vec::new();
        for i in &self.icons {
            icons.push(i.unwrap_config(width, height, device));
        }
        let mut buttons = Vec::new();
        for b in &self.buttons {
            buttons.push(b.unwrap_config(width, height, device));
        }

        objects::Screen {
            id: self.id.clone(),
            background,
            icons,
            buttons,
        }
    }
}

impl ConfigScreenBackground {
    pub fn unwrap_config(&self, device: &wgpu::Device) -> objects::ScreenBackground {
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
        }
    }
}

impl ConfigButton {
    pub fn unwrap_config(&self, width: u32, height: u32, device: &wgpu::Device) -> objects::Button {
        // vertices + buffer
        let mut vertices = objects::TITLE_VERT;
        self.location
            .get_coords(self.aspect, self.height, width, height, &mut vertices);
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
            on_click: self.on_click.clone(),
        }
    }
}

impl ConfigIcon {
    pub fn unwrap_config(&self, width: u32, height: u32, device: &wgpu::Device) -> objects::Icon {
        // vertices + buffer
        let mut vertices = objects::TITLE_VERT;
        self.location
            .get_coords(self.aspect, self.height, width, height, &mut vertices);
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(" Some icon (implement ids for more useful messages!) Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Instances + buffer
        let mut instances = Vec::new();
        for inst in &self.instances {
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
            inst_range,
        }
    }
}

impl ConfigScreenTransform {
    pub fn unwrap_config(&self, width: u32, height: u32) -> objects::ScreenInstance {
        let mut inst: glm::Mat4 = glm::identity();
        glm::scale(&inst, &glm::vec3(self.scale.0, self.scale.1, 1.0));
        glm::rotate_z(&inst, self.rotation);
        let t = self.translation.to_absolute(width, height);
        glm::translate(&inst, &glm::vec3(t[0], t[1], 0.0));

        objects::ScreenInstance {
            transform: inst.into(),
        }
    }
}