use std::collections::{HashMap, HashSet};
use std::default;
use std::sync::{mpsc, MutexGuard};
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

use futures::future::join_all;
use futures::{join, TryFutureExt};
use glm::vec3;
use nalgebra_glm as glm;
use nalgebra_glm::{TVec3, Vec3};
use wgpu::util::DeviceExt;
use wgpu_glyph::{ab_glyph, FontId, GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, Text};
use winit::event::*;
use winit::window::Window;

use audio::CURR_DISP;
use common::configs;
use common::configs::game_config::ConfigWeather;
use common::configs::parameters::{
    DEFAULT_CAMERA_FOV, DEFAULT_CAMERA_POS, DEFAULT_CAMERA_TARGET, DEFAULT_PLAYER_POS,
};
use common::configs::*;
use common::core::choices::OBJECT_PLAYER_MODEL;
use common::core::command::Command;
use common::core::events;
use common::core::powerup_system::StatusEffect::Power;
use common::core::powerup_system::{
    PowerUp, PowerUpEffects, PowerUpStatus, StatusEffect, POWER_UP_TO_EFFECT_MAP,
};
use common::core::states::GameLifeCycleState::Ended;
use common::core::states::GameLifeCycleState::Running;
use common::core::states::{GameLifeCycleState, GameState, ParticleQueue};
use common::core::weather::Weather;
use model::Vertex;
use other_players::OtherPlayer;
use resources::{KOROK_MTL_LIB, KOROK_MTL_LIBRARY_PATH};

use crate::animation::AnimatedModel;
use crate::inputs::Input;
use crate::model::{Model, StaticModel};

mod animation;
pub mod audio;
mod camera;
pub mod event_loop;
pub mod inputs;
mod instance;
mod lights;
mod model;
mod other_players;
mod particles;
mod player;
mod resources;
mod scene;
mod screen;
mod skybox;
mod texture;

const DEFAULT_AMBIENT_MULTIPLIER: f32 = 1.0;
const RAINY_AMBIENT_MULTIPLIER: f32 = 0.5;

// TVec3<f32>

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
    previous_game_life_cycle_state: GameLifeCycleState,
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

        let model_loading_resources = (&device, &queue, &texture_bind_group_layout);

        // load korok material library
        KOROK_MTL_LIB
            .set(
                resources::load_material_library(KOROK_MTL_LIBRARY_PATH, model_loading_resources)
                    .await
                    .unwrap(),
            )
            .expect("failed to set KOROK_MTL_LIB");

        // Scene
        // creates a series of long lived for life time issues
        let long_lived_models = config_instance.models.clone().models.clone();
        let mut long_lived_model_configs = vec![];
        for model_config in long_lived_models.into_iter() {
            long_lived_model_configs.push(model_config);
        }

        // vector of futures and names
        let mut animated_model_futures = vec![];
        let mut static_model_futures = vec![];
        let mut animated_futures_names = vec![];
        let mut static_futures_names = vec![];

        let model_configs = config_instance.models.clone();

        let mut models_scene: HashMap<String, Box<dyn Model>> = HashMap::new();
        let mut models_lobby_scene: HashMap<String, Box<dyn Model>> = HashMap::new();
        let mut models_end_scene: HashMap<String, Box<dyn Model>> = HashMap::new();

        // load all models once and clone for scenes
        // why so many loops and redundant hashmaps here?
        for (index, model_config) in model_configs.models.clone().into_iter().enumerate() {
            if model_config.animated() {
                let model = AnimatedModel::load(
                    &(long_lived_model_configs.get(index).unwrap().path),
                    model_loading_resources,
                );
                animated_model_futures.push(model);
                animated_futures_names.push(model_config.name);
            } else {
                let model = StaticModel::load(
                    &(long_lived_model_configs.get(index).unwrap().path),
                    model_loading_resources,
                );
                static_model_futures.push(model);
                static_futures_names.push(model_config.name);
            }
        }

        let game_config = config_instance.game.clone();

        // placeholder position, will get overriden by server
        let player = player::Player::new(vec3(
            DEFAULT_PLAYER_POS.0,
            DEFAULT_PLAYER_POS.1,
            DEFAULT_PLAYER_POS.2,
        ));
        let player_controller = player::PlayerController::new(
            game_config.camera_config.x_sensitivity,
            game_config.camera_config.y_sensitivity,
            game_config.camera_config.scroll_sensitivity,
        );

        let mut camera_state = camera::CameraState::new(
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
            DEFAULT_CAMERA_FOV,
            0.1,
            100.0,
        );

        // to demonstrate changing global illumination
        camera_state.camera.ambient_multiplier = glm::vec3(1.0, 1.0, 1.0).into();

        let animation_controller = animation::AnimationController::default();

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        #[rustfmt::skip]
            let TEST_LIGHTING: Vec<lights::Light> = Vec::from([
            // point light example
            // lights::Light { position: glm::vec4(10.0, -9.0, 0.0, 1.0), position_2: glm::vec4(0.0, 0.0, 0.0, 0.0), color: glm::vec3(10.0, 10.0, 10.0) },
            // sun
            lights::Light::sun(),
            // line segment light example
            // lights::Light{
            //     position: glm::vec4(0.0, -9.1, 0.0, 2.0),
            //     position_2: glm::vec4(0.0, 15.0, 0.0, 10.0),
            //     color: glm::vec3(1.0, 1.0, 1.0),
            // }
        ]);
        let light_state = lights::LightState::new(TEST_LIGHTING, &device);

        let render_pipeline_layout_2d =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("2D Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout_2d,
                    &mask_texture_bind_group_layout_2d,
                    &camera_state.camera_bind_group_layout,
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
                depth_write_enabled: false, // TODO: change back to true if anything breaks
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
        )).unwrap();
        let zqf = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../../assets/ZuiQingFeng.ttf"
        ))
        .unwrap();

        let glyph_brush = GlyphBrushBuilder::using_font(zqf).build(&device, surface_format);

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

        let skybox_tex = texture::Texture::cube(&config_instance.texture.skybox, &device, &queue)
            .await
            .unwrap();
        let skybox = skybox::SkyBoxDrawer::from_texture(
            skybox_tex,
            parameters::SKYBOX_SCALE,
            &device,
            &config,
            &camera_state.camera_bind_group_layout,
        );

        //
        // <<<<<<< HEAD
        // =======
        //         let mut lobby_scene = scene::Scene::from_config(&lobby_scene_config);
        //         lobby_scene.objects = models;
        //         lobby_scene.draw_scene_dfs();
        //
        //         let mut models: HashMap<String, Box<dyn Model>> = HashMap::new();
        //         for model_config in model_configs.models.clone() {
        //             let model: Box<dyn Model> = if model_config.animated() {
        //                 let anim_model = anim_loaded_models.get(model_config.name.as_str()).unwrap();
        //                 Box::new(anim_model.clone())
        //             } else {
        //                 let static_model = static_loaded_models
        //                     .get(model_config.name.as_str())
        //                     .unwrap();
        //                 Box::new(static_model.clone())
        //             };
        //             models.insert(model_config.name, model);
        //         }
        //
        //         let end_scene_config = config_instance.end_screen_scene.clone();
        //
        //         let mut end_scene = scene::Scene::from_config(&end_scene_config);
        //         end_scene.objects = models;
        //         end_scene.draw_scene_dfs();
        //
        //         let mut scene_map = HashMap::new();
        //         scene_map.insert(String::from("scene:game"), scene);
        //         scene_map.insert(String::from("scene:lobby"), lobby_scene);
        //         scene_map.insert(String::from("scene:end_screen_scene"), end_scene);
        //
        // >>>>>>> main
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

        // join_all
        let (animated_future_done, static_future_done) = join!(
            join_all(animated_model_futures),
            join_all(static_model_futures)
        );

        for (index, model) in animated_future_done.into_iter().enumerate() {
            let name = animated_futures_names.get(index).unwrap();
            let _model = model.unwrap();
            models_scene.insert(String::from(name.clone()), Box::new(_model.clone()));
            models_lobby_scene.insert(String::from(name), Box::new(_model.clone()));
            models_end_scene.insert(String::from(name), Box::new(_model));
        }

        for (index, model) in static_future_done.into_iter().enumerate() {
            let name = static_futures_names.get(index).unwrap();
            let _model = model.unwrap();
            models_scene.insert(String::from(name.clone()), Box::new(_model.clone()));
            models_lobby_scene.insert(String::from(name), Box::new(_model.clone()));
            models_end_scene.insert(String::from(name), Box::new(_model));
        }

        // moved to here to give time for async
        let scene_config = config_instance.scene.clone();
        let mut scene = scene::Scene::from_config(&scene_config);
        scene.objects = models_scene;
        scene.draw_scene_dfs();

        let lobby_scene_config = config_instance.lobby_scene.clone();
        let mut lobby_scene = scene::Scene::from_config(&lobby_scene_config);
        lobby_scene.objects = models_lobby_scene;
        lobby_scene.draw_scene_dfs();

        let end_scene_config = config_instance.end_screen_scene.clone();
        let mut end_scene = scene::Scene::from_config(&end_scene_config);
        end_scene.objects = models_end_scene;
        end_scene.draw_scene_dfs();

        let mut scene_map = HashMap::new();
        scene_map.insert(String::from("scene:game"), scene);
        scene_map.insert(String::from("scene:lobby"), lobby_scene);
        scene_map.insert(String::from("scene:end_screen_scene"), end_scene);

        let display = screen::Display::from_config(
            &display_config,
            texture_map,
            scene_map,
            light_state,
            render_pipeline,
            render_pipeline_2d,
            skybox,
            particle_renderer,
            rect_ibuf,
            depth_texture,
            default_inst_buf,
            // width and height not too important as they will be resized
            // just need to maks sure they're not zero
            1920,
            1080,
            &device,
            sender,
            game_state,
        );

        let _other_players: Vec<OtherPlayer> = (1..5)
            .map(|ind| OtherPlayer {
                id: ind,
                visible: false,
                location: glm::vec4(0.0, 0.0, 0.0, 0.0),
                score: 0.0,
            })
            .collect();

        let _other_players: Vec<OtherPlayer> = (1..5)
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

        let previous_game_life_cycle_state = GameLifeCycleState::Waiting;

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
            previous_game_life_cycle_state,
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
                self.relocate_selectors();
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

    fn relocate_selectors(&mut self) {
        if self.display.current == "display:lobby".to_string() {
            let screen = self.display.screen_map.get_mut("screen:lobby").unwrap();
            for s in vec![
                "leaf_type_selector",
                "leaf_color_selector",
                "wood_color_selector",
            ] {
                let ind = screen.icon_id_map.get(s).unwrap().clone();
                let loc = screen.icons[ind].location.clone();
                screen.icons[ind].relocate(loc, self.config.width, self.config.height, &self.queue);
            }
        }
    }

    fn gradual_convert_lighting(&mut self, (r, g, b): (f32, f32, f32), change_rate_coef: f32) {
        let ambient_multiplier = &mut self.camera_state.camera.ambient_multiplier;

        let targets = [r, g, b];
        for (val, &target) in ambient_multiplier.iter_mut().zip(targets.iter()) {
            if *val != target {
                let sign = (target - *val).signum();
                *val += sign * change_rate_coef;
                if (target - *val).signum() != sign {
                    *val = target;
                }
            }
        }
    }

    fn update(
        &mut self,
        game_state: Arc<Mutex<GameState>>,
        particle_queue: Arc<Mutex<ParticleQueue>>,
        dt: instant::Duration,
        weather_config: ConfigWeather,
    ) {
        // Only update if we're in game/lobby
        if self.display.current != self.display.game_display.clone()
            && self.display.current != "display:lobby"
        {
            self.animation_controller.update(dt);
            return;
        }
        // config setup
        let config_instance = ConfigurationManager::get_configuration();
        let physics_config = config_instance.physics.clone();
        let game_config = config_instance.game.clone();

        let game_state_clone = game_state.lock().unwrap().clone();

        // check whether all players are ready, if so launch the game
        match game_state_clone.life_cycle_state {
            GameLifeCycleState::Running(..) => {
                if self.previous_game_life_cycle_state == GameLifeCycleState::Waiting {
                    // should only run once
                    //println!("once");
                    self.display.change_to(self.display.game_display.clone());
                    self.add_game_particles();
                    /*
                    self.display.current = self.display.game_display.clone();
                    *CURR_DISP.get().unwrap().lock().unwrap() = self.display.current.clone();
                    */
                }

                let change_rate_coef = weather_config.environment_lighting_rate_change_coefficient;
                // update lighting based on weather
                match game_state_clone.world.weather {
                    Some(Weather::Rainy) => {
                        let rainy_ambient_multiplier =
                            weather_config.rainy_weather_ambient_multiplier;
                        self.gradual_convert_lighting(
                            (
                                rainy_ambient_multiplier,
                                rainy_ambient_multiplier,
                                rainy_ambient_multiplier,
                            ),
                            change_rate_coef,
                        );
                    }
                    _ => {
                        let default_ambient_multiplier =
                            weather_config.default_weather_ambient_multiplier;
                        self.gradual_convert_lighting(
                            (
                                default_ambient_multiplier,
                                default_ambient_multiplier,
                                default_ambient_multiplier,
                            ),
                            change_rate_coef,
                        );
                    }
                }
            }
            // check if the game has ended and set corresponding end screen
            GameLifeCycleState::Ended => {
                //println!("{:?}", game_state_clone.life_cycle_state);
                if game_state_clone.game_winner.unwrap() == self.client_id as u32 {
                    self.display.change_to("display:victory".to_owned());
                } else {
                    self.display.change_to("display:defeat".to_owned());
                }
                //*CURR_DISP.get().unwrap().lock().unwrap() = self.display.current.clone();

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
                self.camera_state.projection.fovy = DEFAULT_CAMERA_FOV.to_radians();

                let reset_ambient_multiplier = weather_config.default_weather_ambient_multiplier;
                self.camera_state.camera.ambient_multiplier = glm::vec3(
                    reset_ambient_multiplier,
                    reset_ambient_multiplier,
                    reset_ambient_multiplier,
                );

                self.camera_state
                    .camera_uniform
                    .update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
                self.queue.write_buffer(
                    &self.camera_state.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
                );

                let winner = game_state_clone.game_winner.unwrap();
                let winner_custom = game_state_clone.players_customization.get(&winner).unwrap();
                if let Some(scene) = self.display.scene_map.get_mut("scene:end_screen_scene") {
                    if let Some(node) = scene.scene_graph.get_mut("object:winner_model") {
                        node.model = Some(winner_custom.model.clone());
                        node.colors = Some(winner_custom.color.clone());
                        node.materials = Some(winner_custom.materials.clone());
                    }
                    scene.draw_scene_dfs();
                }

                let loser_screen = self.display.screen_map.get_mut("screen:loser").unwrap();
                let winner_icon_index =
                    *loser_screen.icon_id_map.get("icon:winner_number").unwrap();
                loser_screen.icons[winner_icon_index].texture = format!("icon:player_{winner}");

                return;
            }
            _ => {}
        }

        self.previous_game_life_cycle_state = game_state_clone.life_cycle_state.clone();

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

            // only update game-related info if we're in game
            if let GameLifeCycleState::Running(timestamp) = game_state_clone.life_cycle_state {
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
                        let score_ind = *screen
                            .icon_id_map
                            .get(&format!("icon:score_p{}", i))
                            .unwrap();
                        let profile_body_ind = *screen
                            .icon_id_map
                            .get(&format!("icon:profile_body_p{}", i))
                            .unwrap();
                        let profile_leaf_ind = *screen
                            .icon_id_map
                            .get(&format!("icon:profile_leaf_p{}", i))
                            .unwrap();
                        let score: f32 = self.other_players[i as usize - 1].score;

                        // Set colors/textures
                        if let Some(player_customization) =
                            game_state_clone.players_customization.get(&i)
                        {
                            let leaf_type =
                                format!("icon:profile_leaf_{}", player_customization.model);
                            screen.icons[profile_leaf_ind].texture = leaf_type;

                            let leaf_color = player_customization
                                .color
                                .get(common::core::choices::LEAF_MESH)
                                .unwrap()
                                .rgb_color;
                            screen.icons[profile_leaf_ind].tint =
                                glm::vec4(leaf_color[0], leaf_color[1], leaf_color[2], 1.0);

                            let body_color = player_customization
                                .color
                                .get(common::core::choices::BODY_MESH)
                                .unwrap()
                                .rgb_color;
                            screen.icons[profile_body_ind].tint =
                                glm::vec4(body_color[0], body_color[1], body_color[2], 1.0);
                        }

                        for ind in [score_ind, profile_body_ind, profile_leaf_ind] {
                            let mut location = screen.icons[ind].location;
                            location.horz_disp = (
                                0.0,
                                game_config.score_lower_x
                                    + score
                                        * (game_config.score_upper_x - game_config.score_lower_x),
                            );
                            screen.icons[ind].relocate(
                                location,
                                self.config.width,
                                self.config.height,
                                &self.queue,
                            );
                        }
                    }
                }

                // update the wind charge
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

                    // update player ammo barhead
                    let ind_barhead = *screen.icon_id_map.get("icon:barhead").unwrap();
                    screen.icons[ind_barhead].inst_range = 0..1;

                    // update player number of charges
                    let ind_charge = *screen.icon_id_map.get("icon:charge").unwrap();
                    screen.icons[ind_charge].inst_range = 0..self.player.wind_charge;

                    // Update empty ammo
                    let ind_empty_ammo = *screen.icon_id_map.get("icon:empty_charge").unwrap();
                    screen.icons[ind_empty_ammo].inst_range = self.player.wind_charge..10;
                    // Adjust the range as per your total ammos
                }

                // update weather icon
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
                    let wind_ind = *screen.icon_id_map.get("icon:windy").unwrap();

                    screen.icons[wind_ind].inst_range = 0..{
                        if matches!(
                            game_state.lock().unwrap().world.weather,
                            Some(Weather::Windy(_))
                        ) {
                            1
                        } else {
                            0
                        }
                    };

                    let rain_ind = *screen.icon_id_map.get("icon:rainy").unwrap();
                    screen.icons[rain_ind].inst_range = 0..{
                        if matches!(
                            game_state.lock().unwrap().world.weather,
                            Some(Weather::Rainy)
                        ) {
                            1
                        } else {
                            0
                        }
                    };
                }

                // update cooldowns
                // hard coded for now... TODO: separate function
                // is it necessary? would need to pass around lots of references
                // might be better to create dedicated function in screen/command_handlers
                {
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
                            / physics_config.attack_config.area_attack_cooldown;
                        self.display.transition_map.insert(
                            atk_area_load.clone(),
                            screen::object_transitions::Transition::SqueezeDown(cd_left),
                        );
                    } else {
                        self.display.transition_map.remove(&atk_area_load);
                    }
                }

                // update for powerup
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
                    let ind_atk_ult = *screen.icon_id_map.get("icon:atk_ult_specific").unwrap();

                    let prev_transp = screen.icons[ind_atk_ult].tint[3];
                    if let Some(power_up) = self.player.power_up.as_ref() {
                        // Adjust the properties for both icons
                        match power_up.0 {
                            PowerUp::Blizzard => {
                                screen.icons[ind_atk_ult].texture =
                                    String::from("icon:power_blizzard");
                            }
                            PowerUp::WindEnhancement => {
                                screen.icons[ind_atk_ult].texture = String::from("icon:power_wind");
                            }
                            PowerUp::Dash => {
                                screen.icons[ind_atk_ult].texture = String::from("icon:power_dash");
                            }
                            PowerUp::Flash => {
                                screen.icons[ind_atk_ult].texture =
                                    String::from("icon:power_flash");
                            }
                            PowerUp::Invisible => {
                                screen.icons[ind_atk_ult].texture =
                                    String::from("icon:power_invisible");
                            }
                            PowerUp::TripleJump => {
                                screen.icons[ind_atk_ult].texture =
                                    String::from("icon:power_triple_jump");
                            }
                            PowerUp::Invincible => {
                                screen.icons[ind_atk_ult].texture =
                                    String::from("icon:power_invincible");
                            }
                        }
                        screen.icons[ind_atk_ult].tint[3] = 1.0;
                    } else {
                        // Reset the properties for both icons to their default values
                        screen.icons[ind_atk_ult].tint[3] = 0.0;
                    }
                    if (prev_transp != screen.icons[ind_atk_ult].tint[3]) {
                        let tint = screen.icons[ind_atk_ult].tint;
                        for v in &mut screen.icons[ind_atk_ult].vertices {
                            v.color = tint.into();
                        }
                        self.queue.write_buffer(
                            &screen.icons[ind_atk_ult].vbuf,
                            0,
                            bytemuck::cast_slice(&screen.icons[ind_atk_ult].vertices),
                        );
                    }
                }

                // update cooldown for powerup
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

                    let power_up_overlay_id = "icon:atk_powerup_overlay";
                    let ind_atk_powerup_overlay =
                        *screen.icon_id_map.get(power_up_overlay_id).unwrap();

                    if let Some((power_up, status)) = self.player.power_up.as_ref() {
                        if *status == PowerUpStatus::Active {
                            // Set the overlay icon texture to the corresponding "pure" power-up icon
                            let power_up_overlay_texture = match power_up {
                                PowerUp::Blizzard => "icon:power_blizzard_overlay",
                                PowerUp::WindEnhancement => "icon:power_wind_overlay",
                                PowerUp::Dash => "icon:power_dash_overlay",
                                PowerUp::Flash => "icon:power_flash_overlay",
                                PowerUp::Invisible => "icon:power_invisible_overlay",
                                PowerUp::TripleJump => "icon:power_triple_jump_overlay",
                                PowerUp::Invincible => "icon:power_invincible_overlay",
                            };
                            screen.icons[ind_atk_powerup_overlay].texture =
                                String::from(power_up_overlay_texture);

                            let power_up_status = POWER_UP_TO_EFFECT_MAP
                                .get(&power_up.value())
                                .unwrap_or(&StatusEffect::None);
                            // Calculate the fraction of the power-up time left
                            if let Some(time_left) = self.player.status_effects.get(power_up_status)
                            {
                                //TODO: store that const into config
                                let fraction_left = *time_left / 10.0;

                                // Apply the SqueezeDown transition
                                self.display.transition_map.insert(
                                    String::from(power_up_overlay_id),
                                    screen::object_transitions::Transition::SqueezeDown(
                                        fraction_left,
                                    ),
                                );
                            }
                        } else {
                            // Remove the overlay icon if the power-up is not active
                            self.display.transition_map.remove(power_up_overlay_id);
                            screen.icons[ind_atk_powerup_overlay].texture =
                                String::from("icon:empty");
                        }
                    } else {
                        // in case you died holding a powerup
                        screen.icons[ind_atk_powerup_overlay].texture = String::from("icon:empty");
                    }
                }

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

                self.invisible_players = game_state_clone
                    .get_affected_players(StatusEffect::Power(PowerUpEffects::Invisible));
                self.existing_powerups = game_state_clone.get_existing_powerups();
            }

            self.display
                .scene_map
                .get_mut(scene_id)
                .unwrap()
                .draw_scene_dfs();

            {
                let particle_queue = particle_queue.lock().unwrap();
                self.add_powerup_particles(game_state_clone, particle_queue, dt);
            }

            {
                let particle_queue = particle_queue.lock().unwrap();
                self.load_particles(particle_queue);
            }
        }

        // animation update
        self.animation_controller.update(dt);
        self.animation_controller
            .load_game_state(game_state.lock().unwrap());

        // camera update
        self.camera_state
            .camera_uniform
            .update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
        if  self.display.current == self.display.game_display.clone()  &&
            self.player.on_cooldown.contains_key(&Command::Spawn) {
                self.camera_state.camera_uniform.ambient_multiplier[3] = 0.0;
        } else {
                self.camera_state.camera_uniform.ambient_multiplier[3] = 1.0;
        }
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
            &self.config,
        );

        let size = &self.window.inner_size();

        // TODO: ONLY DISPLAY ONCE THE PLAYER CLICKS "GO" BUTTON
        if self.display.current == "display:lobby" && self.display.customization_choices.ready {
            // TODO: update duration or delete this animation from the animaton_controller after animation is done playing
            self.animation_controller
                .play_animation("idle".to_string(), "object:player_model".to_string());
            let text_size = 0.07 * size.height as f32;
            self.glyph_brush.queue(Section {
                screen_position: (size.width as f32 * 0.5, size.height as f32 * 0.9),
                bounds: (size.width as f32, size.height as f32),
                text: vec![
                    Text::new(format!("Ready! Waiting for other players...").as_str())
                        .with_color([0.0, 0.0, 0.0, 1.0])
                        .with_scale(text_size),
                ],
                layout: Layout::default().h_align(HorizontalAlign::Center),
                ..Section::default()
            });
        } else if self.display.current == "display:victory"
            || self.display.current == "display:defeat"
        {
            self.animation_controller
                .play_animation("dance".to_string(), "object:winner_model".to_string());
        }
        // temporary fix
        else {
            self.animation_controller
                .play_animation("idle".to_string(), "object:player_model".to_string());
        }

        if self.display.current == self.display.game_display.clone() {
            // render respawn cooldown
            if self.player.on_cooldown.contains_key(&Command::Spawn) {
                let spawn_cooldown = self.player.on_cooldown.get(&Command::Spawn).unwrap();
                // stroke attempt
                let size_top = 0.18 * size.height as f32;
                let size_second = 0.1 * size.height as f32;
                self.glyph_brush.queue(Section {
                    screen_position: (size.width as f32 * 0.5, size.height as f32 * 0.3),
                    bounds: (size.width as f32, size.height as f32),
                    text: vec![
                        Text::new("You died!\n")
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(size_top),
                        Text::new("Respawning in ")
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(size_second),
                        Text::new(format!("{:.1}", spawn_cooldown).as_str())
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(size_second),
                        Text::new(" seconds")
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(size_second),
                    ],
                    layout: Layout::default().h_align(HorizontalAlign::Center),
                    ..Section::default()
                });
                // main text
                self.glyph_brush.queue(Section {
                    screen_position: (size.width as f32 * 0.5, size.height as f32 * 0.3),
                    bounds: (size.width as f32, size.height as f32),
                    text: vec![
                        Text::new("You died!\n")
                            .with_color([0.722, 0.525, 0.043, 1.0])
                            .with_scale(size_top),
                        Text::new("Respawning in ")
                            .with_color([0.0, 0.0, 0.0, 1.0])
                            .with_scale(size_second),
                        Text::new(format!("{:.1}", spawn_cooldown).as_str())
                            .with_color([0.0, 0.0, 0.0, 1.0])
                            .with_scale(size_second),
                        Text::new(" seconds")
                            .with_color([0.0, 0.0, 0.0, 1.0])
                            .with_scale(size_second),
                    ],
                    layout: Layout::default().h_align(HorizontalAlign::Center),
                    ..Section::default()
                });
            }
            // render status effect and powerup held
            // self.glyph_brush.queue(Section {
            //     screen_position: (600.0, 20.0),
            //     bounds: (size.width as f32, size.height as f32),
            //     text: vec![Text::new(
            //         format!("Active Status Effects: {:?}\n", self.player.status_effects).as_str(),
            //     )
            //     .with_color([0.0, 0.0, 0.0, 1.0])
            //     .with_scale(40.0)],
            //     ..Section::default()
            // });
            // self.glyph_brush.queue(Section {
            //     screen_position: (600.0, 60.0),
            //     bounds: (size.width as f32, size.height as f32),
            //     text: vec![Text::new(
            //         format!("PowerUp Held: {:?}\n", self.player.power_up).as_str(),
            //     )
            //     .with_color([0.0, 0.0, 0.0, 1.0])
            //     .with_scale(40.0)],
            //     ..Section::default()
            // });
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

    fn add_game_particles(&mut self) {
        let config_instance = ConfigurationManager::get_configuration();
        let particle_config = config_instance.particles.clone();

        let bounds_min = particle_config
            .winning_area_ribbon_particle_config
            .bounds_min;
        let bounds_max = particle_config
            .winning_area_ribbon_particle_config
            .bounds_max;
        let v_dir = particle_config.winning_area_ribbon_particle_config.v_dir;
        let gen = particles::ribbon::LineRibbonGenerator::new(
            glm::vec3(bounds_min.0, bounds_min.1, bounds_min.2),
            glm::vec3(bounds_max.0, bounds_max.1, bounds_max.2),
            glm::vec3(v_dir.0, v_dir.1, v_dir.2),
            particle_config
                .winning_area_ribbon_particle_config
                .linear_speed,
            particle_config
                .winning_area_ribbon_particle_config
                .linear_variance,
            particle_config
                .winning_area_ribbon_particle_config
                .visible_time,
            particle_config.winning_area_ribbon_particle_config.size,
            particle_config
                .winning_area_ribbon_particle_config
                .size_variance,
            particle_config
                .winning_area_ribbon_particle_config
                .subdivisions,
            false,
        );
        let ribbon = particles::ParticleSystem::new(
            std::time::Duration::from_secs_f32(
                particle_config.winning_area_ribbon_particle_config.gen_time,
            ),
            particle_config.winning_area_ribbon_particle_config.time,
            particle_config
                .winning_area_ribbon_particle_config
                .gen_speed,
            glm::vec4(0.4, 0.9, 0.7, 1.0),
            gen,
            (43, 44),
            &self.device,
            &mut self.rng,
        );
        self.display.particles.systems.push(ribbon);
    }

    fn add_powerup_particles(
        &mut self,
        game_state: GameState,
        mut particle_queue: MutexGuard<ParticleQueue>,
        dt: instant::Duration,
    ) {
        let config_instance = ConfigurationManager::get_configuration();
        let particle_config = config_instance.particles.clone();

        let powerup_players = game_state
            .players
            .iter()
            .filter(|(_, player)| !player.power_up.is_none())
            .map(|(&id, _)| id)
            .collect::<Vec<_>>();

        for player_id in powerup_players {
            if player_id != self.client_id as u32 && self.invisible_players.contains(&player_id) {
                continue;
            }

            let player_state = game_state.player(player_id).unwrap();
            let player_pos = player_state.transform.translation;
            let player_vel = player_state.physics.velocity;

            let (player_power_up, player_power_up_status) = player_state.power_up.clone().unwrap();
            let mut aura_color_string = match player_power_up {
                PowerUp::Blizzard => "blizzard",
                _ => "default",
            };

            if player_power_up_status == PowerUpStatus::Active {
                aura_color_string = match player_power_up {
                    PowerUp::WindEnhancement => "wind_enhancement",
                    PowerUp::Dash => "dash",
                    PowerUp::Flash => "flash",
                    PowerUp::Invisible => "invisibility",
                    PowerUp::Invincible => "invincibility",
                    PowerUp::TripleJump => "triple_jump",
                    _ => "default",
                }
            }

            let aura_color = *particle_config
                .powerup_aura_particle_config
                .aura_colors
                .get(aura_color_string)
                .unwrap();
            particle_queue.add_particle(events::ParticleSpec::new(
                events::ParticleType::POWERUP_AURA,
                player_pos + player_vel * (dt.as_secs_f32()),
                glm::vec3(0.0, 1.0, 0.0),
                //TODO: placeholder for player color
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(aura_color.0, aura_color.1, aura_color.2, aura_color.3),
                format!("Power Up Aura from player {}", player_id),
            ));
        }
    }

    fn load_particles(&mut self, mut particle_queue: MutexGuard<ParticleQueue>) {
        let config_instance = ConfigurationManager::get_configuration();
        let game_config = config_instance.game.clone();
        let physics_config = config_instance.physics.clone();
        let particle_config = config_instance.particles.clone();

        // attack consts
        let attack_cd = physics_config.attack_config.attack_cooldown;
        let max_attack_angle = physics_config.attack_config.max_attack_angle;
        let max_attack_dist = physics_config.attack_config.max_attack_dist;
        let blizzard_max_attack_angle = game_config.powerup_config.blizzard_max_attack_angle;
        let blizzard_max_attack_dist = game_config.powerup_config.blizzard_max_attack_dist;
        let area_attack_cd = physics_config.attack_config.area_attack_cooldown;
        let max_area_attack_dist = physics_config.attack_config.max_area_attack_dist;
        // particle consts
        let time_divider = particle_config.time_divider;

        for p in &particle_queue.particles {
            // println!("Handling particle of type: {:?}", p.p_type);
            match p.p_type {
                // generator
                events::ParticleType::ATTACK => {
                    let leaf_type = match &p.particle_id[..] {
                        common::configs::particle_config::MODEL_1 => 0,
                        common::configs::particle_config::MODEL_2 => 1,
                        common::configs::particle_config::MODEL_3 => 2,
                        common::configs::particle_config::MODEL_4 => 3,
                        _ => 0,
                    };

                    // ORIGINAL
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
                        (
                            leaf_type * particles::constants::ATK_NUM_TEX_TYPES
                                + particles::constants::ATK_BASE_IND,
                            (leaf_type + 1) * particles::constants::ATK_NUM_TEX_TYPES
                                + particles::constants::ATK_BASE_IND,
                        ),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(atk);
                }
                events::ParticleType::AREA_ATTACK => {
                    // in this case, only position matters
                    let leaf_type = match &p.particle_id[..] {
                        common::configs::particle_config::MODEL_1 => 0,
                        common::configs::particle_config::MODEL_2 => 1,
                        common::configs::particle_config::MODEL_3 => 2,
                        common::configs::particle_config::MODEL_4 => 3,
                        _ => 0,
                    };
                    let time = area_attack_cd / time_divider;
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
                        (
                            leaf_type * particles::constants::ATK_NUM_TEX_TYPES
                                + particles::constants::ATK_BASE_IND,
                            (leaf_type + 1) * particles::constants::ATK_NUM_TEX_TYPES
                                + particles::constants::ATK_BASE_IND,
                        ),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(atk);
                }
                events::ParticleType::BLIZZARD => {
                    let time = particle_config.blizzard_particle_config.time / time_divider;
                    let blizz_gen = particles::gen::ConeGenerator::new(
                        p.position,
                        p.direction,
                        p.up,
                        blizzard_max_attack_angle,
                        blizzard_max_attack_dist / time,
                        particle_config.blizzard_particle_config.linear_variance,
                        PI,
                        particle_config.blizzard_particle_config.angular_variance,
                        particle_config.blizzard_particle_config.size,
                        particle_config.blizzard_particle_config.size_variance,
                        particle_config.blizzard_particle_config.size_growth,
                        false,
                    );
                    // System
                    let blizzard = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.2),
                        time,
                        particle_config.blizzard_particle_config.gen_speed,
                        p.color,
                        blizz_gen,
                        (
                            particles::constants::SNOW_BASE_IND,
                            particles::constants::SNOW_BASE_IND
                                + particles::constants::SNOW_NUM_TEX_TYPES,
                        ),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(blizzard);
                }
                events::ParticleType::POWERUP => {
                    // in this case, only position matters
                    let time = particle_config.powerup_particle_config.time / time_divider;
                    let powerup_gen = particles::gen::SphereGenerator::new(
                        p.position,
                        particle_config.powerup_particle_config.max_dist / time,
                        particle_config.powerup_particle_config.linear_variance,
                        PI,
                        particle_config.powerup_particle_config.angular_variance,
                        particle_config.powerup_particle_config.size,
                        particle_config.powerup_particle_config.size_variance,
                        particle_config.powerup_particle_config.size_growth,
                        false,
                    );
                    // System
                    let powerup = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.2),
                        time,
                        particle_config.powerup_particle_config.gen_speed,
                        p.color,
                        powerup_gen,
                        (
                            particles::constants::SOFT_CIRCLE_IND,
                            particles::constants::SOFT_CIRCLE_IND + 1,
                        ),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(powerup);
                }
                events::ParticleType::POWERUP_AURA => {
                    // in this case, only position matters
                    let time = particle_config.powerup_aura_particle_config.time / time_divider;
                    let powerup_aura_gen = particles::gen::CylinderGenerator::new(
                        p.position,
                        p.direction,
                        p.up,
                        particle_config.powerup_aura_particle_config.r,
                        particle_config.powerup_aura_particle_config.half_height,
                        particle_config.powerup_aura_particle_config.linear_speed,
                        particle_config.powerup_aura_particle_config.linear_variance,
                        PI,
                        particle_config
                            .powerup_aura_particle_config
                            .angular_variance,
                        particle_config.powerup_aura_particle_config.size,
                        particle_config.powerup_aura_particle_config.size_variance,
                        particle_config.powerup_aura_particle_config.size_growth,
                        false,
                    );
                    // System
                    let powerup_aura = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.05),
                        time,
                        particle_config.powerup_aura_particle_config.gen_speed,
                        p.color,
                        powerup_aura_gen,
                        (
                            particles::constants::SOFT_CIRCLE_IND,
                            particles::constants::SOFT_CIRCLE_IND + 1,
                        ),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(powerup_aura);
                }
                events::ParticleType::RAIN => {
                    let time = 2.;
                    println!("adding particle: {:?}", p);
                    let atk_gen = particles::gen::RainGenerator::new(
                        p.position + Vec3::new(0., 20., 0.),
                        (20.0, 20.0, 20.0),
                        p.direction,
                        3.0,
                        0.3,
                        25.0,
                        PI,
                        0.0,
                        false,
                    );
                    // System
                    let atk = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(0.2),
                        time,
                        2500.0,
                        p.color,
                        atk_gen,
                        (
                            particles::constants::RAIN_IND,
                            particles::constants::RAIN_IND + 1,
                        ),
                        &self.device,
                        &mut self.rng,
                    );
                    self.display.particles.systems.push(atk);
                }
                events::ParticleType::WIND => {
                    let time = 1.2;
                    let gen = particles::ribbon::LineRibbonGenerator::new(
                        glm::vec3(-25., -6., -25.),
                        glm::vec3(25., 0., 25.),
                        p.direction,
                        40.0,
                        0.0,
                        0.1,
                        15.,
                        0.0,
                        1,
                        false,
                    );
                    let atk = particles::ParticleSystem::new(
                        std::time::Duration::from_secs_f32(time),
                        0.25,
                        10.0,
                        p.color,
                        gen,
                        (
                            particles::constants::STREAK_IND,
                            particles::constants::STREAK_IND + 1,
                        ),
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
