use crate::texture;

mod constants;

pub struct SkyBox{
    pub scale: f32,
    pub vbuf: wgpu::Buffer,
    pub ibuf: wgpu::Buffer,
    pub texture: texture::Texture,
    pub tex_bind_group: wgpu::BindGroup,
    pub tex_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl SkyBox{
    pub fn from_texture(texture: wgpu::Texture) -> Self{
        todo!();
    }
}