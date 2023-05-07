use std::collections::HashMap;

use crate::resources;

pub async fn load_screen_tex(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    id: String,
    file_name: &str,
    texture_map: &mut HashMap<String, wgpu::BindGroup>,
){
    let texture = match resources::load_texture(
        file_name,
        &device,
        &queue
    ).await {
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