use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources;
use crate::screen;
use common::configs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTexture {
    pub textures: Vec<ConfigTextureItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTextureItem {
    pub name: String,
    pub path: String,
}

pub async fn load_screen_tex_config(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    file_name: &str,
    texture_map: &mut HashMap<String, wgpu::BindGroup>,
) {
    let screen_tex_config = configs::from_file::<_, ConfigTexture>(file_name).unwrap();
    for tex in &screen_tex_config.textures {
        load_screen_tex(
            device,
            queue,
            layout,
            tex.name.clone(),
            &tex.path,
            texture_map,
        )
        .await;
    }
}

pub async fn load_screen_tex(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    id: String,
    file_name: &str,
    texture_map: &mut HashMap<String, wgpu::BindGroup>,
) {
    let texture = match resources::load_texture(file_name, &device, &queue).await {
        Ok(tex) => tex,
        Err(e) => panic!("Failed to load screen texture {:?}\n", e),
    };
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
        label: None,
    });
    texture_map.insert(id, texture_bind_group);
}
