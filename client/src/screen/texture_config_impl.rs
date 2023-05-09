use std::collections::HashMap;


use crate::resources;
use common::configs::*;
use common::configs::texture_config::ConfigTexture;


pub async fn load_screen_tex_config(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    file_name: &str,
    texture_map: &mut HashMap<String, wgpu::BindGroup>,
) {
    let config_instance = ConfigurationManager::get_configuration();
    let screen_tex_config = config_instance.texture.clone();
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
