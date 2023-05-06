use std::collections::HashMap;
use crate::particles::ParticleDrawer;
use crate::texture;

pub mod objects;
pub mod location;
// pub mod config; // TODO later

#[rustfmt::skip]
const RECT_IND : Vec<u16> = vec![
    0, 2, 1,
    0, 3, 2,
];

// Should only be one of these int he entire game
pub struct Display{
    pub groups: HashMap<String, objects::DisplayGroup>,
    pub current: String,
    pub scene_pipeline: wgpu::RenderPipeline,
    pub ui_pipeline: wgpu::RenderPipeline,
    pub particles: ParticleDrawer,
    pub rect_ibuf: wgpu::Buffer,
}

impl Display{
    pub fn new() -> Self{
        todo!();
    }

    pub fn render(
        &mut self,
        texture_map: &HashMap<String, texture::Texture>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &wgpu::RenderPass,
    ){
        todo!();
    }
}

// Place click events here
