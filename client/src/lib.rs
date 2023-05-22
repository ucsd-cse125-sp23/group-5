use glm::vec3;
use other_players::OtherPlayer;
use std::collections::{HashMap, HashSet};

use std::sync::{mpsc, MutexGuard};
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

use common::configs::*;
use common::core::powerup_system::StatusEffect;
use common::core::states::GameLifeCycleState::Ended;
use model::Vertex;
use winit::event::*;

mod camera;
mod instance;
mod lights;
mod model;
mod other_players;
mod particles;
mod player;
mod resources;
mod scene;
mod screen;
mod texture;

use nalgebra_glm as glm;

mod animation;
pub mod audio;
pub mod event_loop;
pub mod inputs;

use crate::animation::AnimatedModel;
use crate::inputs::Input;
use crate::model::{Model, StaticModel};

use common::configs;
use common::configs::parameters::{DEFAULT_CAMERA_POS, DEFAULT_CAMERA_TARGET, DEFAULT_PLAYER_POS};
use common::core::command::Command;
use common::core::events;
use common::core::states::GameLifeCycleState::Running;
use common::core::states::{GameState, ParticleQueue};
use wgpu::util::DeviceExt;
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text};
use winit::window::Window;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    player: player::Player,
    player_controller: player::PlayerController,
    other_players: Vec<OtherPlayer>,
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
    color_bind_group_layout: wgpu::BindGroupLayout,
    animation_controller: animation::AnimationController,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(
        window: Window,
        client_id: u8,
        sender: mpsc::Sender<Input>,
        game_state: Arc<Mutex<GameState>>,
    ) -> Self {
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

        let color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("color_bind_group_layout"),
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

        let mask_texture_bind_group_layout_2d =
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
                label: Some("2d_mask_texture_bind_group_layout"),
            });

        //Render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("3d_shader.wgsl"));
        let shader_2d = device.create_shader_module(wgpu::include_wgsl!("2d_shader.wgsl"));

        let config_instance = ConfigurationManager::get_configuration();
        // Scene
        let model_configs = config_instance.models.clone();

        let model_loading_resources = (&device, &queue, &texture_bind_group_layout);

        let mut models: HashMap<String, Box<dyn Model>> = HashMap::new();

        for model_config in model_configs.models {
            let model: Box<dyn Model> = if model_config.animated() {
                Box::new(
                    AnimatedModel::load(&model_config.path, model_loading_resources)
                        .await
                        .unwrap(),
                )
            } else {
                Box::new(
                    StaticModel::load(&model_config.path, model_loading_resources)
                        .await
                        .unwrap(),
                )
            };
            models.insert(model_config.name, model);
        }

        let scene_config = config_instance.scene.clone();

        let mut scene = scene::Scene::from_config(&scene_config);
        scene.objects = models;

        // placeholder position, will get overriden by server
        let player = player::Player::new(vec3(
            DEFAULT_PLAYER_POS.0,
            DEFAULT_PLAYER_POS.1,
            DEFAULT_PLAYER_POS.2,
        ));
        let player_controller = player::PlayerController::new(4.0, 0.7, 0.1);

        let camera_state = camera::CameraState::new(
            &device,
            glm::vec3(
                DEFAULT_CAMERA_POS.0,
                DEFAULT_CAMERA_POS.1,
                DEFAULT_CAMERA_POS.2,
            ),
            glm::vec3(
                DEFAULT_CAMERA_TARGET.0,
                DEFAULT_CAMERA_TARGET.1,
                DEFAULT_CAMERA_TARGET.2,
            ),
            vec3(0.0, 1.0, 0.0),
            config.width,
            config.height,
            45.0,
            0.1,
            100.0,
        );

        scene.draw_scene_dfs();

        let animation_controller = animation::AnimationController::default();

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        #[rustfmt::skip]
            let TEST_LIGHTING: Vec<lights::Light> = Vec::from([
            lights::Light { position: glm::vec4(1.0, 1.0, 1.0, 0.0), color: glm::vec3(1.0, 1.0, 1.0) },
            lights::Light { position: glm::vec4(-1.0, -1.0, -1.0, 0.0), color: glm::vec3(1.0, 0.7, 0.4) },
            lights::Light { position: glm::vec4(-10.0, 0.0, 0.0, 3.0), color: glm::vec3(0.0, 0.2, 0.2) },
        ]);
        let light_state = lights::LightState::new(TEST_LIGHTING, &device);

        let _render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_state.camera_bind_group_layout,
                    &light_state.light_bind_group_layout,
                    &color_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_layout_2d =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("2D Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout_2d,
                    &color_bind_group_layout,
                    &mask_texture_bind_group_layout_2d,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_state.camera_bind_group_layout,
                    &light_state.light_bind_group_layout,
                    &color_bind_group_layout,
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

        let rng = rand::thread_rng();
        let particle_tex =
            texture::Texture::from_images(&config_instance.texture.particles, &device, &queue)
                .await
                .unwrap();
        let particle_renderer = particles::ParticleDrawer::new(
            &device,
            &config,
            &camera_state.camera_bind_group_layout,
            particle_tex,
        );

        // TODO: fix later -> currently loading all models again for new scene and couldn't figure out lifetime errors if we were to use references
        let model_configs = config_instance.models.clone();
        let model_loading_resources = (&device, &queue, &texture_bind_group_layout);
        let mut models: HashMap<String, Box<dyn Model>> = HashMap::new();

        for model_config in model_configs.models {
            if model_config.animated() {
                // TODO: skipping animated models for now for lobby scene
                continue;
            } else {
                let model: Box<dyn Model> = Box::new(
                    StaticModel::load(&model_config.path, model_loading_resources)
                        .await
                        .unwrap(),
                );
                models.insert(model_config.name, model);
            }
        }

        let lobby_scene_config = config_instance.lobby_scene.clone();

        let mut lobby_scene = scene::Scene::from_config(&lobby_scene_config);
        lobby_scene.objects = models;
        lobby_scene.draw_scene_dfs();
        // lobby_scene.objects.insert("player".to_owned(), models.get("player").unwrap());
        // lobby_scene.objects.insert("ferris".to_owned(), *models.get("ferris").unwrap());

        let mut scene_map = HashMap::new();
        scene_map.insert(String::from("scene:game"), scene);
        scene_map.insert(String::from("scene:lobby"), lobby_scene);

        // end debug code that needs to be replaced

        let mut texture_map: HashMap<String, wgpu::BindGroup> = HashMap::new();
        screen::texture_helper::load_screen_tex_config(
            &device,
            &queue,
            &texture_bind_group_layout_2d,
            configs::TEXTURE_CONFIG_PATH,
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
            &color_bind_group_layout,
            sender,
            game_state,
        );

        let other_players: Vec<OtherPlayer> = (1..5)
            .map(|ind| OtherPlayer {
                id: ind,
                visible: false,
                location: glm::vec4(0.0, 0.0, 0.0, 0.0),
                score: 0.0,
            })
            .collect();

        let other_players: Vec<OtherPlayer> = (1..5)
            .map(|ind| OtherPlayer {
                id: ind,
                visible: false,
                location: glm::vec4(0.0, 0.0, 0.0, 0.0),
                score: 0.0,
            })
            .collect();

        let other_players: Vec<OtherPlayer> = (1..5)
            .map(|ind| OtherPlayer {
                id: ind,
                visible: false,
                location: glm::vec4(0.0, 0.0, 0.0, 0.0),
                score: 0.0,
            })
            .collect();

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            player,
            player_controller,
            other_players,
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
            color_bind_group_layout,
            animation_controller,
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
                state: crate::ElementState::Released,
                ..
            } => {
                self.display.click(&self.mouse_position);
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position[0] = 2.0 * (position.x as f32) / self.window_size[0] - 1.0;
                self.mouse_position[1] = -2.0 * (position.y as f32) / self.window_size[1] + 1.0;
                true
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
        // config setup
        let config_instance = ConfigurationManager::get_configuration();
        let physics_config = config_instance.physics.clone();
        let game_config = config_instance.game.clone();

        // Only update if we're in game/lobby
        if self.display.current != self.display.game_display.clone()
            && self.display.current != "display:lobby"
        {
            return;
        }

        let game_state_clone = game_state.lock().unwrap().clone();

        // check if the game has ended and set corresponding end screen
        if game_state_clone.life_cycle_state == Ended {
            if game_state_clone.game_winner.unwrap() == self.client_id as u32 {
                self.display.current = "display:victory".to_owned();
            } else {
                self.display.current = "display:defeat".to_owned();
            }

            // Reset camera and player for lobby
            self.camera_state.camera.position = glm::vec3(
                DEFAULT_CAMERA_POS.0,
                DEFAULT_CAMERA_POS.1,
                DEFAULT_CAMERA_POS.2,
            );
            self.camera_state.camera.target = glm::vec3(
                DEFAULT_CAMERA_TARGET.0,
                DEFAULT_CAMERA_TARGET.1,
                DEFAULT_CAMERA_TARGET.2,
            );
            return;
        }

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
                    game_state.lock().unwrap(),
                    &mut self.player_controller,
                    &mut self.player,
                    &mut self.camera_state,
                    dt,
                    self.client_id,
                );

            other_players::load_game_state(
                &mut self.other_players,
                game_state.lock().unwrap(),
                game_config.clone(),
            );

            // update player scores
            {
                let screen_id = self
                    .display
                    .groups
                    .get(&self.display.game_display)
                    .unwrap()
                    .screen
                    .as_ref()
                    .unwrap();

                let screen = self.display.screen_map.get_mut(screen_id).unwrap();
                for i in 1..5 {
                    let ind = screen
                        .icon_id_map
                        .get(&format!("icon:score_p{}", i))
                        .unwrap()
                        .clone();
                    let score: f32 = self.other_players[i as usize - 1].score;
                    let mut location = screen.icons[ind].location.clone();
                    location.horz_disp = (
                        0.0,
                        game_config.score_lower_x
                            + score * (game_config.score_upper_x - game_config.score_lower_x),
                    );
                    screen.icons[ind].relocate(
                        location,
                        self.config.width,
                        self.config.height,
                        &self.queue,
                    );
                }
            }

            // update player number of charges
            {
                let screen_id = self
                    .display
                    .groups
                    .get(&self.display.game_display)
                    .unwrap()
                    .screen
                    .as_ref()
                    .unwrap();

                let screen = self.display.screen_map.get_mut(screen_id).unwrap();
                let ind = screen.icon_id_map.get("icon:charge").unwrap().clone();
                screen.icons[ind].inst_range = 0..self.player.wind_charge;
            }

            // update cooldowns
            // hard coded for now... TODO: separate function
            // is it necessary? would need to pass around lots of references
            // might be better to create dedicated function in screen/mod.rs
            {
                let screen_id = self
                    .display
                    .groups
                    .get(&self.display.game_display)
                    .unwrap()
                    .screen
                    .as_ref()
                    .unwrap();

                // TODO: Magic constants here seem a little unavoidable?
                let atk_load = String::from("icon:atk_forward_overlay");
                let atk_area_load = String::from("icon:atk_wave_overlay");

                if self.player.on_cooldown.contains_key(&Command::Attack) {
                    let cd_left = self.player.on_cooldown.get(&Command::Attack).unwrap()
                        / physics_config.attack_config.attack_cooldown;
                    self.display.transition_map.insert(
                        atk_load.clone(),
                        screen::object_transitions::Transition::SqueezeDown(cd_left),
                    );
                } else {
                    self.display.transition_map.remove(&atk_load);
                }

                if self.player.on_cooldown.contains_key(&Command::AreaAttack) {
                    let cd_left = self.player.on_cooldown.get(&Command::AreaAttack).unwrap()
                        / physics_config.attack_config.attack_cooldown;
                    self.display.transition_map.insert(
                        atk_area_load.clone(),
                        screen::object_transitions::Transition::SqueezeDown(cd_left),
                    );
                } else {
                    self.display.transition_map.remove(&atk_area_load);
                }
            }

            self.display
                .scene_map
                .get_mut(scene_id)
                .unwrap()
                .draw_scene_dfs();

            let player_loc = self
                .display
                .scene_map
                .get(scene_id)
                .unwrap()
                .get_player_positions();

            // ASSUME: Ids should always be 1-4
            for p in &mut self.other_players {
                p.visible = false;
            }
            for (i, loc) in player_loc {
                self.other_players[i as usize - 1].location = loc;
                self.other_players[i as usize - 1].visible = true;
            }

            self.invisible_players = game_state_clone.get_affected_players(StatusEffect::Invisible);
            self.existing_powerups = game_state_clone.get_existing_powerups();
        }

        let particle_queue = particle_queue.lock().unwrap();
        self.load_particles(particle_queue);

        // animation update
        self.animation_controller.update(dt);
        self.animation_controller
            .load_game_state(game_state.lock().unwrap());

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
            // &self.player,
            &self.other_players,
            &self.invisible_players,
            &self.existing_powerups,
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &mut self.animation_controller,
            &self.color_bind_group_layout,
            &output,
            self.client_id as u32,
        );

        let size = &self.window.inner_size();

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

        // render status effect and powerup held
        self.glyph_brush.queue(Section {
            screen_position: (600.0, 20.0),
            bounds: (size.width as f32, size.height as f32),
            text: vec![Text::new(
                format!("Active Status Effects: {:?}\n", self.player.status_effects).as_str(),
            )
            .with_color([0.0, 0.0, 0.0, 1.0])
            .with_scale(40.0)],
            ..Section::default()
        });
        self.glyph_brush.queue(Section {
            screen_position: (600.0, 60.0),
            bounds: (size.width as f32, size.height as f32),
            text: vec![
                Text::new(format!("PowerUp Held: {:?}\n", self.player.power_up).as_str())
                    .with_color([0.0, 0.0, 0.0, 1.0])
                    .with_scale(40.0),
            ],
            ..Section::default()
        });

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
        let config_instance = ConfigurationManager::get_configuration();
        let physics_config = config_instance.physics.clone();
        let particle_config = config_instance.particles.clone();
        // attack consts
        let attack_cd = physics_config.attack_config.attack_cooldown;
        let max_attack_angle = physics_config.attack_config.max_attack_angle;
        let max_attack_dist = physics_config.attack_config.max_attack_dist;
        let area_attack_cd = physics_config.attack_config.area_attack_cooldown;
        let max_area_attack_dist = physics_config.attack_config.max_area_attack_dist;
        // particle consts
        let time_divider = particle_config.time_divider;

        for p in &particle_queue.particles {
            println!("Handling particle of type: {:?}", p.p_type);
            match p.p_type {
                // generator
                events::ParticleType::ATTACK => {
                    let time = attack_cd / time_divider;
                    println!("adding particle: {:?}", p);
                    let atk_gen = particles::gen::ConeGenerator::new(
                        p.position,
                        p.direction,
                        p.up,
                        max_attack_angle,
                        max_attack_dist / time,
                        particle_config.attack_particle_config.linear_variance,
                        PI,
                        particle_config.attack_particle_config.angular_variance,
                        particle_config.attack_particle_config.size,
                        particle_config.attack_particle_config.size_variance,
                        particle_config.attack_particle_config.size_growth,
                        false,
                    );
                    // System
                    let atk = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.2),
                        time,
                        particle_config.attack_particle_config.gen_speed,
                        p.color,
                        atk_gen,
                        (1, 4),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(atk);
                }
                events::ParticleType::AREA_ATTACK => {
                    // in this case, only position matters
                    let time = area_attack_cd / time_divider;
                    println!("adding particle: {:?}", p);
                    let atk_gen = particles::gen::SphereGenerator::new(
                        p.position,
                        max_area_attack_dist / time,
                        particle_config.area_attack_particle_config.linear_variance,
                        PI,
                        particle_config.area_attack_particle_config.angular_variance,
                        particle_config.area_attack_particle_config.size,
                        particle_config.area_attack_particle_config.size_variance,
                        particle_config.area_attack_particle_config.size_growth,
                        false,
                    );
                    // System
                    let atk = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.2),
                        time,
                        particle_config.area_attack_particle_config.gen_speed,
                        p.color,
                        atk_gen,
                        (1, 4),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(atk);
                }
            }
        }
        particle_queue.particles.clear();
    }
}
