use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::model::DrawModel;
use crate::particles::{ParticleDrawer, self};
use crate::{texture, model, camera, lights};

pub mod objects;
pub mod location;
pub mod texture_config;
// pub mod config; // TODO later

// Should only be one of these in the entire game
pub struct Display{
    pub groups: HashMap<String, objects::DisplayGroup>,
    pub current: String,
    pub game_display: String,
    pub texture_map: HashMap<String, wgpu::BindGroup>,
    pub light_state: lights::LightState, // Grandfathered in, we don't really use lights
    pub scene_pipeline: wgpu::RenderPipeline,
    pub ui_pipeline: wgpu::RenderPipeline,
    pub particles: ParticleDrawer,
    pub rect_ibuf: wgpu::Buffer,
    pub depth_texture: texture::Texture,
    pub default_inst_buf: wgpu::Buffer,
}

impl Display{
    pub fn new(
        groups: HashMap<String, objects::DisplayGroup>,
        current: String,
        game_display: String,
        texture_map: HashMap<String, wgpu::BindGroup>,
        light_state: lights::LightState,
        scene_pipeline: wgpu::RenderPipeline,
        ui_pipeline: wgpu::RenderPipeline,
        particles: ParticleDrawer,
        rect_ibuf: wgpu::Buffer,
        depth_texture: texture::Texture,
        default_inst_buf: wgpu::Buffer,
    ) -> Self{
        Self {
            groups,
            current,
            game_display,
            texture_map,
            light_state,
            scene_pipeline,
            ui_pipeline,
            particles,rect_ibuf,
            depth_texture,
            default_inst_buf
        }
    }

    pub fn render(
        &mut self,
        camera_state: &camera::CameraState,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        output: &wgpu::SurfaceTexture,
    ){
        // inability to find the scene would be a major bug
        // panicking is fine
        let display_group = self.groups.get(&self.current).unwrap();

        // Instance 3D objects
        let mut instanced_objs = Vec::new();
        
        match &display_group.scene {
            None => {},
            Some(scene) => {
                for (index, instances) in scene.objects_and_instances.iter() {
                    let count = instances.len();
                    let instanced_obj = model::InstancedModel::new(
                        scene.objects.get(index).unwrap(),
                        instances,
                        device,
                    );
                    instanced_objs.push((instanced_obj, count));
                }
            }
        };
        
        // generate particles
        let mut to_draw: Vec<particles::Particle> = Vec::new();
        self.particles
            .get_particles_to_draw(&camera_state.camera, &mut to_draw);
        // write buffer to gpu and set buffer
        let particle_inst_buf = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&to_draw),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            // Make render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.888,
                            g: 0.815,
                            b: 0.745,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Optionally draw scene
            if instanced_objs.len() > 0{
                render_pass.set_pipeline(&self.scene_pipeline);
                render_pass.set_bind_group(2, &self.light_state.light_bind_group, &[]);

                for obj in instanced_objs.iter() {
                    render_pass.draw_model_instanced(
                        &obj.0,
                        0..obj.1 as u32,
                        &camera_state.camera_bind_group,
                    );
                }
            }

            // Draw particles
            self.particles.draw(
                &mut render_pass,
                &camera_state.camera_bind_group,
                to_draw.len() as u32,
                &particle_inst_buf,
                device,
                queue,
            );

            // Optionally draw GUI
            match &display_group.screen{
                None => {},
                Some(screen) => {
                    render_pass.set_pipeline(&self.ui_pipeline);
                    render_pass.set_index_buffer(self.rect_ibuf.slice(..), wgpu::IndexFormat::Uint16);
                    // first optionally draw background
                    if let Some(bkgd) = &screen.background { 
                        render_pass.draw_ui_instanced(
                            &self.texture_map.get(
                                &bkgd.texture
                            ).unwrap(),
                            &bkgd.vbuf,
                            &self.default_inst_buf,
                            0..1
                        );
                    };
                    for button in &screen.buttons{
                        let texture = match button.is_hover {
                            true => &button.hover_texture,
                            false => &button.default_texture,
                        };
                        render_pass.draw_ui_instanced(
                            &self.texture_map.get(
                                texture
                            ).unwrap(),
                            &button.vbuf,
                            &self.default_inst_buf,
                            0..1
                        );
                    }
                    for icon in &screen.icons{
                        render_pass.draw_ui_instanced(
                            &self.texture_map.get(
                                &icon.texture
                            ).unwrap(),
                            &icon.vbuf,
                            &self.default_inst_buf,
                            0..1
                        );
                    }
                }
            };
        }
    }
}

pub trait DrawGUI<'a> {
    fn draw_ui_instanced(
        &mut self,
        tex_bind_group: &'a wgpu::BindGroup,
        vbuf: &'a wgpu::Buffer,
        inst_buf: &'a wgpu::Buffer,
        instances: std::ops::Range<u32>,
    );
}

impl<'a, 'b> DrawGUI<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_ui_instanced(
            &mut self,
            tex_bind_group: &'a wgpu::BindGroup,
            vbuf: &'a wgpu::Buffer,
            inst_buf: &'a wgpu::Buffer,
            instances: std::ops::Range<u32>,
        ) {
        self.set_vertex_buffer(0, vbuf.slice(..));
        self.set_vertex_buffer(1, inst_buf.slice(..));
        self.set_bind_group(0, tex_bind_group, &[]);
        self.draw_indexed(0..6, 0, instances);
    }
}
