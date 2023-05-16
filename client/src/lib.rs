use glm::vec3;

use std::collections::{HashMap, HashSet};
use std::sync::MutexGuard;
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

use winit::event::*;

mod model;

use model::Vertex;

mod camera;
mod instance;
mod lights;
mod particles;
mod player;
mod resources;
mod scene;
// mod screen_objects;
mod screen;
mod texture;
use nalgebra_glm as glm;

use common::configs::*;

pub mod audio;
pub mod event_loop;
pub mod inputs;

use common::configs::*;
use common::core::command::Command;
use common::core::events;
use common::core::powerup_system::StatusEffect;
use common::core::states::{GameState, ParticleQueue};
use wgpu::util::DeviceExt;
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text};
use winit::window::Window;

const MODELS_CONFIG_PATH: &str = "models.json";
const SCENE_CONFIG_PATH: &str = "scene.json";

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    player: player::Player,
    player_controller: player::PlayerController,
    player_loc: Vec<(u32, glm::Vec4)>,
    invisible_players: HashSet<u32>,
    existing_powerups: HashSet<u32>,
    camera_state: camera::CameraState,
    display: screen::Display,
    pub mouse_position: [f32; 2],
    pub window_size: [f32; 2],
    rng: rand::rngs::ThreadRng,
    client_id: u8,
    staging_belt: wgpu::util::StagingBelt,
    glyph_brush: GlyphBrush<()>,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window, client_id: u8) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.describe().srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 10,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 11,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_bind_group_layout_2d =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("2d_texture_bind_group_layout"),
            });

        //Render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("3d_shader.wgsl"));
        let shader_2d = device.create_shader_module(wgpu::include_wgsl!("2d_shader.wgsl"));

        let config_instance = ConfigurationManager::get_configuration();
        // Scene
        let scene_config = config_instance.scene.clone();
        let models_config = config_instance.models.clone();

        let mut models = HashMap::new();

        for model_config in models_config.models {
            let model = resources::load_model(
                &model_config.path,
                &device,
                &queue,
                &texture_bind_group_layout,
            )
            .await
            .unwrap();
            models.insert(model_config.name, model);
        }

        let mut scene = scene::Scene::from_config(&scene_config);
        scene.objects = models;

        // placeholder position, will get overriden by server
        let player = player::Player::new(vec3(0.0, 0.0, 0.0));
        let player_controller = player::PlayerController::new(4.0, 0.7, 0.1);

        let camera_state = camera::CameraState::new(
            &device,
            player.position + vec3(-2.0, 2.0, 0.0),
            player.position,
            vec3(0.0, 1.0, 0.0),
            config.width,
            config.height,
            45.0,
            0.1,
            100.0,
        );

        scene.draw_scene_dfs();

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        #[rustfmt::skip]
            let TEST_LIGHTING: Vec<lights::Light> = Vec::from([
            lights::Light { position: glm::vec4(1.0, 1.0, 1.0, 0.0), color: glm::vec3(1.0, 1.0, 1.0) },
            lights::Light { position: glm::vec4(-1.0, -1.0, -1.0, 0.0), color: glm::vec3(1.0, 0.7, 0.4) },
            lights::Light { position: glm::vec4(-10.0, 0.0, 0.0, 3.0), color: glm::vec3(0.0, 0.2, 0.2) },
        ]);
        let light_state = lights::LightState::new(TEST_LIGHTING, &device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_state.camera_bind_group_layout,
                    &light_state.light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_layout_2d =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("2D Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout_2d],
                push_constant_ranges: &[],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_state.camera_bind_group_layout,
                    &light_state.light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D World Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                //buffers: &[Vertex::desc()],
                buffers: &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual, // 1.
                stencil: wgpu::StencilState::default(),          // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let render_pipeline_2d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2d Render Pipeline"),
            layout: Some(&render_pipeline_layout_2d),
            vertex: wgpu::VertexState {
                module: &shader_2d,
                entry_point: "vs_main",
                buffers: &[
                    screen::objects::Vertex::desc(),
                    screen::objects::ScreenInstance::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_2d,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // text
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../../assets/Inconsolata-Regular.ttf"
        ))
        .unwrap();

        let glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&device, surface_format);

        let mut rng = rand::thread_rng();
        let particle_tex = resources::load_texture("test_particle.png", &device, &queue)
            .await
            .unwrap();
        let mut particle_renderer = particles::ParticleDrawer::new(
            &device,
            &config,
            &camera_state.camera_bind_group_layout,
            particle_tex,
        );

        //TODO: for debugging -----
        // let mut groups: HashMap<String, DisplayGroup> = HashMap::new();
        // screen::objects::get_display_groups(&device, scene, &mut groups);
        let default_display_id = String::from("display:title");
        let game_display_id = String::from("display:game");

        let mut scene_map = HashMap::new();
        scene_map.insert(String::from("scene:game"), scene);
        // end debug code that needs to be replaced

        let mut texture_map: HashMap<String, wgpu::BindGroup> = HashMap::new();
        screen::texture_config_helper::load_screen_tex_config(
            &device,
            &queue,
            &texture_bind_group_layout_2d,
            screen::TEX_CONFIG_PATH,
            &mut texture_map,
        )
        .await;

        let rect_ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Const Rect Index Buffer"),
            contents: bytemuck::cast_slice(&screen::objects::RECT_IND),
            usage: wgpu::BufferUsages::INDEX,
        });
        let default_inst = screen::objects::ScreenInstance::default();
        let default_inst_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Default Instance Buffer"),
            contents: bytemuck::cast_slice(&[default_inst]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let config_instance = ConfigurationManager::get_configuration();
        let display_config = config_instance.display.clone();
        let display = screen::Display::from_config(
            &display_config,
            texture_map,
            scene_map,
            light_state,
            render_pipeline,
            render_pipeline_2d,
            particle_renderer,
            rect_ibuf,
            depth_texture,
            default_inst_buf,
            // width and height not too important as they will be resized
            // just need to maks sure they're not zero
            1920,
            1080,
            &device,
        );

        // let screens =
        //     screen_objects::get_screens(&texture_bind_group_layout_2d, &device, &queue).await;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            player,
            player_controller,
            player_loc: Vec::new(),
            invisible_players: HashSet::default(),
            existing_powerups: HashSet::default(),
            camera_state,
            display,
            mouse_position: [0.0, 0.0],
            window_size: [1.0, 1.0],
            rng,
            client_id,
            staging_belt,
            glyph_brush,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size[0] = new_size.width as f32;
            self.window_size[1] = new_size.height as f32;

            self.camera_state
                .projection
                .resize(new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.camera_state
                .camera_uniform
                .update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
            self.queue.write_buffer(
                &self.camera_state.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
            );

            for screen in self.display.screen_map.values_mut() {
                screen.resize(new_size.width, new_size.height, &self.device, &self.queue);
            }

            self.display.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel { delta, .. } => {
                self.player_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: _,
                ..
            } => {
                self.display.click(&self.mouse_position);
                return true;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position[0] = 2.0 * (position.x as f32) / self.window_size[0] - 1.0;
                self.mouse_position[1] = -2.0 * (position.y as f32) / self.window_size[1] + 1.0;
                return true;
            }
            _ => false,
        }
    }

    fn update(
        &mut self,
        game_state: Arc<Mutex<GameState>>,
        particle_queue: Arc<Mutex<ParticleQueue>>,
        dt: instant::Duration,
    ) {
        let game_state = game_state.lock().unwrap();
        let game_state_clone = game_state.clone();
        // game state to scene graph conversion and update
        {
            // new block because we need to drop scene_id before continuing
            // it borrows self
            let scene_id = self
                .display
                .groups
                .get(&self.display.game_display)
                .unwrap()
                .scene
                .as_ref()
                .unwrap();

            self.display
                .scene_map
                .get_mut(scene_id)
                .unwrap()
                .load_game_state(
                    game_state,
                    &mut self.player_controller,
                    &mut self.player,
                    &mut self.camera_state,
                    dt,
                    self.client_id,
                );

            self.display
                .scene_map
                .get_mut(scene_id)
                .unwrap()
                .draw_scene_dfs();

            self.player_loc = self
                .display
                .scene_map
                .get(scene_id)
                .unwrap()
                .get_player_positions();

            self.invisible_players = game_state_clone.get_affected_players(StatusEffect::Invisible);
            self.existing_powerups = game_state_clone.get_existing_powerups();
        }

        let particle_queue = particle_queue.lock().unwrap();
        self.load_particles(particle_queue);

        // camera update
        self.camera_state
            .camera_uniform
            .update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.write_buffer(
            &self.camera_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.display.render(
            &self.mouse_position,
            &self.camera_state,
            &self.player_loc,
            &self.invisible_players,
            &self.existing_powerups,
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &output,
            self.client_id as u32,
        );

        let size = &self.window.inner_size();

        // TODO: maybe refactor later?
        // if player is alive
        if !self.player.is_dead {
            // render ammo remaining
            self.glyph_brush.queue(Section {
                screen_position: (30.0, 20.0),
                bounds: (size.width as f32, size.height as f32),
                text: vec![Text::new(
                    &format!("Wind Charge remaining: {:.1}\n", self.player.wind_charge).as_str(),
                )
                .with_color([0.0, 0.0, 0.0, 1.0])
                .with_scale(40.0)],
                ..Section::default()
            });
            // render ability cooldowns
            if self.player.on_cooldown.contains_key(&Command::Attack) {
                let attack_cooldown = self.player.on_cooldown.get(&Command::Attack).unwrap();
                self.glyph_brush.queue(Section {
                    screen_position: (30.0, 60.0),
                    bounds: (size.width as f32, size.height as f32),
                    text: vec![Text::new(
                        &format!("Attack cooldown: {:.1}\n", attack_cooldown).as_str(),
                    )
                    .with_color([0.0, 0.0, 0.0, 1.0])
                    .with_scale(40.0)],
                    ..Section::default()
                });
            }
        } else {
            // render respawn cooldown
            if self.player.on_cooldown.contains_key(&Command::Spawn) {
                let spawn_cooldown = self.player.on_cooldown.get(&Command::Spawn).unwrap();
                self.glyph_brush.queue(Section {
                    screen_position: (size.width as f32 * 0.5, size.height as f32 * 0.4),
                    bounds: (size.width as f32, size.height as f32),
                    text: vec![
                        Text::new("You died!\n")
                            .with_color([1.0, 1.0, 0.0, 1.0])
                            .with_scale(100.0),
                        Text::new("Respawning in ")
                            .with_color([1.0, 1.0, 0.0, 1.0])
                            .with_scale(60.0),
                        Text::new(&format!("{:.1}", spawn_cooldown).as_str())
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(60.0),
                        Text::new(" seconds")
                            .with_color([1.0, 1.0, 0.0, 1.0])
                            .with_scale(60.0),
                    ],
                    layout: Layout::default().h_align(HorizontalAlign::Center),
                    ..Section::default()
                });
            }
        }

        // Draw the text!
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                size.width,
                size.height,
            )
            .expect("Draw queued");

        // Submit the work!
        self.staging_belt.finish();

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Recall unused staging buffers
        self.staging_belt.recall();
        Ok(())
    }

    fn load_particles(&mut self, mut particle_queue: MutexGuard<ParticleQueue>) {
        for p in &particle_queue.particles {
            println!("Handling particle of type: {:?}", p.p_type);
            match p.p_type {
                //TODO: move to config
                // generator
                events::ParticleType::ATTACK => {
                    println!("adding particle: {:?}", p);
                    let atk_gen = particles::gen::ConeGenerator::new(
                        p.position,
                        p.direction,
                        p.up,
                        std::f32::consts::FRAC_PI_3,
                        10.0,
                        0.3,
                        PI,
                        0.5,
                        75.0,
                        10.0,
                        7.0,
                        false,
                    );
                    // System
                    let atk = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.2),
                        0.5,
                        2000.0,
                        p.color,
                        atk_gen,
                        (1, 4),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(atk);
                }
                _ => {}
            }
        }
        particle_queue.particles.clear();
    }
}
