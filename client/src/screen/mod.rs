pub mod screen_objects;
pub mod config;

pub struct Display{
    pub groups: HashMap<String, config::DisplayGroup>,
    pub current: &str,
    pub scene_pipeline: wgpu::RenderPipeline,
    pub ui_pipeline: wgpu::RenderPipeline,
    pub particles: ParticleDrawer,
}