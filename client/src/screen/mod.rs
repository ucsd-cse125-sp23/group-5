use common::configs::ConfigurationManager;
use nalgebra_glm as glm;
use rand::rngs::adapter::ReadError;
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex};
use wgpu::util::DeviceExt;

use common::configs::display_config::{ConfigDisplay, ScreenLocation};
use common::configs::model_config::ModelIndex;
use common::core::choices::CurrentSelections;
use common::core::mesh_color::MeshColor;
use common::core::states::GameState;

use crate::audio::CURR_DISP;
use crate::inputs::Input;
use crate::model::DrawModel;
use crate::other_players::OtherPlayer;
use crate::particles::{self, ParticleDrawer};
use crate::scene::{InstanceBundle, Scene};
use crate::screen::display_helper::{create_display_group, create_screen_map};
use crate::screen::object_transitions::Transition;
use crate::screen::ui_interaction::BUTTON_MAP;
use crate::{camera, lights, model, texture};

use self::objects::Screen;
use crate::skybox;

pub mod display_helper;
pub mod location_helper;
pub mod object_transitions;
pub mod objects;
pub mod texture_helper;
pub mod ui_interaction;

// Should only be one of these in the entire game
pub struct Display {
    pub groups: HashMap<String, objects::DisplayGroup>,
    pub current: String,
    pub game_display: String,
    pub texture_map: HashMap<String, wgpu::BindGroup>,
    pub screen_map: HashMap<String, Screen>,
    pub scene_map: HashMap<String, Scene>,
    pub transition_map: HashMap<String, Transition>,
    // Grandfathered in, we don't really use lights
    pub light_state: lights::LightState,
    // Assumes we only want 1 skybox, it is drawn all the time
    pub skybox: skybox::SkyBoxDrawer,
    pub scene_pipeline: wgpu::RenderPipeline,
    pub ui_pipeline: wgpu::RenderPipeline,
    pub particles: ParticleDrawer,
    pub rect_ibuf: wgpu::Buffer,
    pub depth_texture: texture::Texture,
    pub default_inst_buf: wgpu::Buffer,
    pub customization_choices: CurrentSelections, // TODO: fix later, here for now until the code for sending these updates is finished
    // for sending command
    pub sender: mpsc::Sender<Input>,
    pub game_state: Arc<Mutex<GameState>>,
}

impl Display {
    pub fn from_config(
        config: &ConfigDisplay,
        texture_map: HashMap<String, wgpu::BindGroup>,
        scene_map: HashMap<String, Scene>,
        light_state: lights::LightState,
        scene_pipeline: wgpu::RenderPipeline,
        ui_pipeline: wgpu::RenderPipeline,
        skybox: skybox::SkyBoxDrawer,
        particles: ParticleDrawer,
        rect_ibuf: wgpu::Buffer,
        depth_texture: texture::Texture,
        default_inst_buf: wgpu::Buffer,
        screen_width: u32,
        screen_height: u32,
        device: &wgpu::Device,
        sender: mpsc::Sender<Input>,
        game_state: Arc<Mutex<GameState>>,
    ) -> Self {
        let groups = create_display_group(config);
        let screen_map = create_screen_map(config, device, screen_width, screen_height);
        Self {
            groups,
            current: config.default_display.clone(),
            game_display: config.game_display.clone(),
            texture_map,
            screen_map,
            scene_map,
            transition_map: HashMap::new(),
            light_state,
            skybox,
            scene_pipeline,
            ui_pipeline,
            particles,
            rect_ibuf,
            depth_texture,
            default_inst_buf,
            customization_choices: CurrentSelections::default(),
            sender,
            game_state,
        }
    }

    /// Takes care of any cleanup switching displays might need
    pub fn change_to(&mut self, new: String) {
        self.particles.systems.clear();
        self.current = new;
        *CURR_DISP.get().unwrap().lock().unwrap() = self.current.clone();
    }

    pub fn render(
        &mut self,
        mouse: &[f32; 2],
        camera_state: &camera::CameraState,
        // player: &crate::player::Player,
        other_players: &Vec<OtherPlayer>,
        _invisible_players: &HashSet<u32>,
        existing_powerups: &HashSet<u32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        animation_controller: &mut crate::animation::AnimationController,
        color_bind_group_layout: &wgpu::BindGroupLayout,
        _output: &wgpu::SurfaceTexture,
        _client_id: u32,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let config_instance = ConfigurationManager::get_configuration();
        let game_config = config_instance.game.clone();

        // inability to find the display group would be a major bug
        // panicking is fine
        let display_group = self.groups.get(&self.current).unwrap();

        // Instance 3D objects
        let mut instanced_objs = Vec::new();

        match &display_group.scene {
            None => {}
            Some(scene_id) => {
                let scene = self.scene_map.get(scene_id).unwrap();
                for (index, instances) in scene.objects_and_instances.iter() {
                    for instance in instances.iter() {
                        let mut model = scene.objects.get(index).unwrap().clone_box();
                        animation_controller
                            .update_animated_model_state(&mut model, &instance.node_id);

                        let instanced_obj = model::InstancedModel::new(
                            model,
                            &vec![InstanceBundle::instance(instance)],
                            device,
                            color_bind_group_layout,
                        );
                        instanced_objs.push(instanced_obj);
                    }
                }
            }
        };

        // generate particles
        let mut to_draw: Vec<particles::Particle> = Vec::new();
        // conditionally add the player labels
        if self.current == self.game_display {
            let cam_dir: glm::Vec3 =
                glm::normalize(&(camera_state.camera.position - camera_state.camera.target));
            let cpos = &camera_state.camera.position;

            for player in other_players {
                let id = player.id;
                let pos = player.location;
                if !player.visible {
                    // skip if player invisible
                    continue;
                }
                // -1 to cancel out 1.0 in pos, 2.5 to place above the player
                let pos = pos + glm::vec4(0.0, 2.5, 0.0, -1.0);
                let vec3pos = glm::vec3(pos[0], pos[1], pos[2]);
                let z_pos = glm::dot(&(vec3pos - cpos), &cam_dir);
                to_draw.push(particles::Particle {
                    start_pos: pos.into(),
                    velocity: glm::vec4(0.0, 0.0, 0.0, 0.0).into(),
                    color: glm::vec4(1.0, 1.0, 1.0, 1.0).into(), // was blue intended to be 0?
                    normal_1: [0., 0., 0., 0.],
                    normal_2: [0., 0., 0., 0.],
                    spawn_time: 0.0,
                    size: 75.0,
                    tex_id: id as i32 + 4,
                    z_pos,
                    time_elapsed: 0.0,
                    size_growth: 0.0,
                    halflife: 1.0,
                    FLAG: particles::constants::POINT_PARTICLE,
                });
            }

            // draw the powerups if they exist
            for id in existing_powerups.iter() {
                let _pos = *game_config
                    .powerup_config
                    .power_up_locations
                    .get(id)
                    .unwrap();
                let pos = glm::vec4(_pos.0, _pos.1, _pos.2, 0.0);
                let vec3pos = glm::vec3(pos[0], pos[1], pos[2]);
                let z_pos = glm::dot(&(vec3pos - cpos), &cam_dir);
                to_draw.push(particles::Particle {
                    start_pos: pos.into(),
                    velocity: glm::vec4(0.0, 0.0, 0.0, 0.0).into(),
                    color: glm::vec4(1.0, 1.0, 1.0, 1.0).into(), // was blue intended to be 0?
                    normal_1: [0., 0., 0., 0.],
                    normal_2: [0., 0., 0., 0.],
                    spawn_time: 0.0,
                    size: 100.0,
                    tex_id: 9, // TODO: Find more icons for powerup
                    // prob need a system to link each powerup to each icon
                    // (Or perhaps we can just use one Icon and show players what they get after they have obtained it, adds a little bit of randomness on top)
                    z_pos,
                    time_elapsed: 0.0,
                    size_growth: 0.0,
                    halflife: 1.0,
                    FLAG: particles::constants::POINT_PARTICLE,
                });
            }
        }
        self.particles
            .get_particles_to_draw(&camera_state.camera, &mut to_draw);
        // write buffer to gpu and set buffer
        let particle_inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&to_draw),
            usage: wgpu::BufferUsages::VERTEX,
        });

        {
            // Make render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.945098, // 0.888,
                            g: 0.909804, // 0.815,
                            b: 0.874509, // 0.745,
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

            // draw skybox
            render_pass.set_pipeline(&self.skybox.render_pipeline);
            render_pass.set_bind_group(0, &self.skybox.tex_bind_group, &[]);
            render_pass.set_bind_group(1, &camera_state.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.skybox.vbuf.slice(..));
            render_pass.set_index_buffer(self.skybox.ibuf.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..36, 0, 0..1);

            // Optionally draw scene
            if !instanced_objs.is_empty() {
                render_pass.set_pipeline(&self.scene_pipeline);
                render_pass.set_bind_group(2, &self.light_state.light_bind_group, &[]);

                for obj in instanced_objs.iter() {
                    render_pass.draw_model_instanced(
                        obj,
                        0..obj.num_instances as u32,
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
            match &display_group.screen {
                None => {}
                Some(screen_id) => {
                    let screen = self.screen_map.get_mut(screen_id).unwrap();
                    render_pass.set_pipeline(&self.ui_pipeline);
                    render_pass
                        .set_index_buffer(self.rect_ibuf.slice(..), wgpu::IndexFormat::Uint16);
                    // first optionally draw background
                    if let Some(bkgd) = &screen.background {
                        render_pass.draw_ui_instanced(
                            self.texture_map.get(&bkgd.texture).unwrap(),
                            self.texture_map.get(&bkgd.mask_texture).unwrap(),
                            &bkgd.vbuf,
                            &self.default_inst_buf,
                            0..1,
                        );
                    };

                    let icons_on_top = vec![
                        "leaf_type_selector",
                        "leaf_color_selector",
                        "wood_color_selector",
                        "hover_icon",
                    ];
                    let mut icons_top = Vec::new();

                    for icon in &mut screen.icons {
                        if icons_on_top.contains(&icon.id.as_str()) {
                            icons_top.push(icon);
                            continue;
                        }
                        match self.transition_map.get(&icon.id) {
                            None => {}
                            Some(x) => x.apply(icon, queue),
                        };
                        render_pass.draw_ui_instanced(
                            self.texture_map.get(&icon.texture).unwrap(),
                            self.texture_map.get(&icon.mask_texture).unwrap(),
                            &icon.vbuf,
                            &icon.inst_buf,
                            icon.inst_range.clone(),
                        );
                    }


                    let mut curr_btn_loc = ScreenLocation {
                        vert_disp: (1000.0, 1000.0),
                        horz_disp: (1000.0, 1000.0),
                    };
                    let mut button_id = None;
                    for button in &mut screen.buttons {
                        let mut texture = &button.default_texture;
                        texture = match button.is_hover(mouse) {
                            true => {
                                if button.id != Some("start_game".to_string()) && !self.customization_choices.ready{
                                    curr_btn_loc = button.location.clone();
                                }
                                button_id = button.id.clone();
                                for v in &mut button.vertices {
                                    v.color = [
                                        button.hover_tint[0],
                                        button.hover_tint[1],
                                        button.hover_tint[2],
                                        button.hover_tint[3],
                                    ];
                                }
                                queue.write_buffer(
                                    &button.vbuf,
                                    0,
                                    bytemuck::cast_slice(&button.vertices),
                                );
                                // if button.selected {
                                //     texture
                                // } else {
                                //     &button.hover_texture
                                // }
                                &button.hover_texture
                            }
                            false => {
                                for v in &mut button.vertices {
                                    v.color = [
                                        button.default_tint[0],
                                        button.default_tint[1],
                                        button.default_tint[2],
                                        button.default_tint[3],
                                    ];
                                }
                                queue.write_buffer(
                                    &button.vbuf,
                                    0,
                                    bytemuck::cast_slice(&button.vertices),
                                );
                                texture
                            }
                        };

                        texture = match button.selected_texture.as_ref() {
                            None => texture,
                            Some(tex) => match button.selected {
                                true => tex,
                                false => texture,
                            },
                        };

                        render_pass.draw_ui_instanced(
                            self.texture_map.get(texture).unwrap(),
                            self.texture_map.get(&button.mask_texture).unwrap(),
                            &button.vbuf,
                            &self.default_inst_buf,
                            0..1,
                        );
                    }

                    // TEMPORARY FIX
                    if screen.id == "screen:lobby" {
                        for icon in icons_top {
                            if icon.id == "hover_icon" {
                                if let Some(b) = &button_id {
                                    icon.height = 0.37;
                                    if b.contains("color"){
                                        icon.height = 0.222;
                                    }
                                }
                                icon.relocate(curr_btn_loc, config.width, config.height, queue);
                            }

                            render_pass.draw_ui_instanced(
                                self.texture_map.get(&icon.texture).unwrap(),
                                self.texture_map.get(&icon.mask_texture).unwrap(),
                                &icon.vbuf,
                                &icon.inst_buf,
                                icon.inst_range.clone(),
                            );
                        }
                    }
                }
            };
        }
    }

    pub fn click(&mut self, mouse: &[f32; 2]) {
        //iterate through buttons
        let display_group = self.groups.get(&self.current).unwrap();

        let screen = match display_group.screen.as_ref() {
            None => return,
            Some(s) => self.screen_map.get(s).unwrap(),
        };
        let mut to_call: Option<&str> = None;
        let mut button_id: Option<String> = None;
        for button in &screen.buttons {
            if button.is_hover(mouse) {
                to_call = Some(&button.on_click[..]);
                button_id = button.id.clone();
            }
        }
        match to_call {
            None => {}
            Some(id) => BUTTON_MAP.get(id).unwrap()(self, button_id),
        };
    }
}

pub trait DrawGUI<'a> {
    fn draw_ui_instanced(
        &mut self,
        tex_bind_group: &'a wgpu::BindGroup,
        mask_bind_group: &'a wgpu::BindGroup,
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
        mask_bind_group: &'a wgpu::BindGroup,
        vbuf: &'a wgpu::Buffer,
        inst_buf: &'a wgpu::Buffer,
        instances: std::ops::Range<u32>,
    ) {
        self.set_vertex_buffer(0, vbuf.slice(..));
        self.set_vertex_buffer(1, inst_buf.slice(..));
        self.set_bind_group(0, tex_bind_group, &[]);
        self.set_bind_group(1, mask_bind_group, &[]);
        self.draw_indexed(0..6, 0, instances);
    }
}
