use std::sync::{Arc, Mutex};

use winit::event::*;

mod model;

use crate::model::DrawModel;
use model::Vertex;

mod camera;
mod instance;
mod lights;
mod player;
mod resources;
mod scene;
mod texture;

extern crate nalgebra_glm as glm;

pub mod event_loop;
pub mod inputs;

use common::core::states::GameState;
use winit::window::Window;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
    player: player::Player,
    player_controller: player::PlayerController,
    scene: scene::Scene,
    light_state: lights::LightState,
    camera_state: camera::CameraState,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> Self {
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        //Render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let player = player::Player::new(glm::vec3(5.0, 7.0, 5.0));
        let player_controller = player::PlayerController::new(4.0, 0.7);

        let camera_state = camera::CameraState::new(
            &device,
            player.position + glm::vec3(-2.0, 2.0, 0.0),
            player.position,
            glm::vec3(0.0, 1.0, 0.0),
            config.width,
            config.height,
            45.0,
            0.1,
            100.0,
        );

        // Scene
        let obj_model = resources::load_model(
            "assets/island.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();
        let instance_vec = vec![
            instance::Instance {
                #[rustfmt::skip]
                transform: glm::mat4(
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                ),
            },
            instance::Instance {
                #[rustfmt::skip]
                transform: glm::mat4(
                    1.0, 0.0, 0.0, 10.0,
                    0.0, 1.0, 0.0, 2.0,
                    0.0, 0.0, 1.0, 2.0,
                    0.0, 0.0, 0.0, 1.0,
                ),
            },
        ];

        let cube_model = resources::load_model(
            "assets/cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();
        let cube_instance_vec = vec![instance::Instance {
            #[rustfmt::skip]
            transform: glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ),
        }];
        let scene = scene::Scene {
            objects: vec![obj_model, cube_model],
            instance_vectors: vec![instance_vec, cube_instance_vec],
        };

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        #[rustfmt::skip]
            let TEST_LIGHTING: Vec<lights::Light> = Vec::from([
            lights::Light { position: glm::vec4(1.0, 0.0, 0.0, 0.0), color: glm::vec3(1.0, 1.0, 1.0) },
            lights::Light { position: glm::vec4(-10.0, 0.0, 0.0, 3.0), color: glm::vec3(0.0, 0.2, 0.2) },
        ]);
        let light_state = lights::LightState::new(TEST_LIGHTING, &device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
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
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            scene,
            player,
            player_controller,
            camera_state,
            depth_texture,
            light_state,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
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

            self.depth_texture =
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
            } => true,
            _ => false,
        }
    }

    fn update(&mut self, game_state: Arc<Mutex<GameState>>, dt: instant::Duration) {
        let game_state = game_state.lock().unwrap();
        // TODO: game state to scene graph conversion should be done in the scene graph itself
        // like `scene_graph.load_game_state(game_state)`

        // just for testing
        // update the camera target
        if !game_state.players.is_empty() {
            let player_state = &game_state.players[0];
            // update player controller (player, camera, etc) with the latest player state
            self.player_controller.update(
                &mut self.player,
                &mut self.camera_state.camera,
                player_state,
                dt,
            );
        }

        // hard code updating player instance for now
        self.scene.instance_vectors[1][0].transform = self.player.calc_transf_matrix();

        self.camera_state
            .camera_uniform
            .update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.write_buffer(
            &self.camera_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
        );
        self.queue.write_buffer(
            &self.light_state.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_state.light_uniform]),
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
        {
            //placed up here because it needs to be dropped after the render pass
            let mut instanced_objs = Vec::new();

            for i in 0..self.scene.objects.len() {
                let instanced_model = model::InstancedModel::new(
                    &self.scene.objects[i],
                    &self.scene.instance_vectors[i],
                    &self.device,
                );
                instanced_objs.push(instanced_model);
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(2, &self.light_state.light_bind_group, &[]);

            for instanced_obj in instanced_objs.iter() {
                render_pass.draw_model_instanced(
                    instanced_obj,
                    0..instanced_obj.num_instances as u32,
                    &self.camera_state.camera_bind_group,
                );
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
