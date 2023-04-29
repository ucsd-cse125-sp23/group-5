use std::{sync::{Arc, Mutex}, f32::consts::PI};

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
mod screen_objects;
mod particles;
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
    render_pipeline_2d: wgpu::RenderPipeline,
    player: player::Player,
    player_controller: player::PlayerController,
    scene: scene::Scene,
    light_state: lights::LightState,
    camera_state: camera::CameraState,
    screens: Vec<screen_objects::Screen>,
    screen_ind: usize,
    particle_renderer: particles::ParticleDrawer,
    client_id: u8,
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
        
        let player = player::Player::new(glm::vec3(5.0, 7.0, 5.0));
        // let player_controller = player::PlayerController::new(4.0, 1.0, 0.7); 
        let player_controller = player::PlayerController::new(4.0, 1.0, 0.1); 

        let camera_state = camera::CameraState::new(
            &device,
        player.position + glm::vec3(-5.0, 15.0, 0.0), 
        player.position, 
        glm::vec3(0.0, 1.0, 0.0),
        config.width, config.height, 45.0, 0.1, 100.0,
        );

        // Scene
        let obj_model = resources::load_model(
            "assets/islands_set_remade.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();
        // let instance_vec = vec![
        //     instance::Instance{transform: glm::mat4(
        //         1.0, 0.0, 0.0, 0.0,
        //         0.0, 1.0, 0.0, 0.0,
        //         0.0, 0.0, 1.0, 0.0,
        //         0.0, 0.0, 0.0, 1.0
        //     )},
        //     instance::Instance{transform: glm::mat4(
        //         1.0, 0.0, 0.0, 10.0,
        //         0.0, 1.0, 0.0, 2.0,
        //         0.0, 0.0, 1.0, 2.0,
        //         0.0, 0.0, 0.0, 1.0
        //     )},
        // ];
        // use cube as a placeholder player model for now
        let player_obj = resources::load_model(
            "assets/cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();

        let cube_obj = resources::load_model(
            "assets/cube.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();

        let ferris_obj = resources::load_model(
            "assets/ferris.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();
        // let cube_instance_vec = vec![
        //     instance::Instance{transform: glm::mat4(
        //         1.0, 0.0, 0.0, 5.0,
        //         0.0, 1.0, 0.0, 10.0,
        //         0.0, 0.0, 1.0, 5.0,
        //         0.0, 0.0, 0.0, 1.0
        //     )},
        // ];
        // let scene = scene::Scene{objects: vec![obj_model], instance_vectors: vec![instance_vec]};

        let mut scene = scene::Scene::new(vec![obj_model, player_obj, cube_obj, ferris_obj]);
        scene.init_scene_graph();

        // placeholder position, will get overriden by server
        let player = player::Player::new(glm::vec3(0.0, 0.0, 0.0));
        let player_controller = player::PlayerController::new(4.0, 0.7, 0.1);

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

        scene.draw_scene_dfs(&camera_state.camera);

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

        let render_pipeline_2d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2d Render Pipeline"),
            layout: Some(&render_pipeline_layout_2d),
            vertex: wgpu::VertexState {
                module: &shader_2d,
                entry_point: "vs_main",
                buffers: &[screen_objects::Vertex::desc(), screen_objects::ScreenInstance::desc()],
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

        let mut particle_renderer = particles::ParticleDrawer::new(&device, &config, &camera_state.camera_bind_group_layout);
        let particle_tex = resources::load_texture("test_particle.png", &device, &queue).await.unwrap();
        let test_particle_gen = particles::LineGenerator::new(glm::vec3(0.0, -5.0, 0.0), glm::vec3(0.0, 2.0, 0.0));
        let test_particle = particles::ParticleSystem::new(
            std::time::Duration::from_secs(60),
            2.0,
            5.0,
            test_particle_gen,
            &particle_tex,
            &particle_renderer.tex_bind_group_layout,
            4,
            &device,
        );
        particle_renderer.systems.push(test_particle);

        let screens = 
            screen_objects::get_screens(&texture_bind_group_layout_2d, &device, &queue).await;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            render_pipeline_2d,
            scene,
            player,
            player_controller,
            camera_state,
            depth_texture,
            light_state,
            screens,
            #[cfg(not(feature = "debug-lobby"))]
            screen_ind: 0,
            #[cfg(feature = "debug-lobby")]
            screen_ind: 1,
            particle_renderer,
            client_id,
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

            screen_objects::update_screen(new_size.width, new_size.height, &self.device, &mut self.screens[0].objects[0]);

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
        // game state to scene graph conversion and update
        self.scene.load_game_state(
            game_state,
            &mut self.player_controller,
            &mut self.player,
            &mut self.camera_state,
            dt,
            self.client_id,
        );
        // camera update
        self.camera_state
            .camera_uniform
            .update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
        self.queue.write_buffer(
            &self.camera_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_state.camera_uniform]),
        );
        // light update
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

            for i in self.scene.objects_and_instances.iter() {
                let count = i.1.len();
                let instanced_obj =
                    model::InstancedModel::new(&self.scene.objects[i.0.index], &i.1, &self.device);
                instanced_objs.push((instanced_obj, count));
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            // r: 0.1,
                            // g: 0.2,
                            // b: 0.3,
                            // a: 1.0,
                            r: 0.274,
                            g: 0.698,
                            b: 0.875,
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

            for obj in instanced_objs.iter() {
                render_pass.draw_model_instanced(
                    &obj.0,
                    0..obj.1 as u32,
                    &self.camera_state.camera_bind_group,
                );
            }

            // Particle Drawing
            self.particle_renderer.draw(
                &mut render_pass,
                &self.camera_state.camera_bind_group,
                &self.camera_state.camera,
                &self.device,
                &self.queue
            );

            // GUI drawing
            render_pass.set_pipeline(&self.render_pipeline_2d);

            // TO REMOVE: for testing
            if self.screen_ind == 1{
                self.screens[self.screen_ind].ranges[1] = 0..5;
            }
            
            for i in 0..self.screens[self.screen_ind].objects.len(){
                let obj = &self.screens[self.screen_ind].objects[i];
                let range = &self.screens[self.screen_ind].ranges[i];
                render_pass.set_vertex_buffer(0, obj.vbuf.slice(..));
                render_pass.set_vertex_buffer(1, obj.inst_buf.slice(..)); 
                render_pass.set_index_buffer(obj.ibuf.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &obj.bind_group, &[]);
                render_pass.draw_indexed(0..obj.num_indices, 0, range.clone());
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
