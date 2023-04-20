
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod model;
use model::Vertex;
use crate::model::DrawModel;
mod camera;
mod texture;
mod resources;
mod lights;
mod instance;
mod scene;
extern crate nalgebra_glm as glm;
pub mod event_loop;
pub mod inputs;

// const NUM_INSTANCES_PER_ROW: u32 = 1;

// pub async fn run() {
//     env_logger::init();
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new()
//         .with_title("test")
//         .with_fullscreen(Some(winit::window::Fullscreen::Borderless(Option::None)))
//         .build(&event_loop)
//         .unwrap();

//     let mut state = State::new(window).await; 
//     let mut last_render_time = instant::Instant::now();

//     event_loop.run(move |event, _, control_flow| match event {
//         Event::DeviceEvent {
//             event: DeviceEvent::MouseMotion{ delta, },
//             .. // We're not using device_id currently
//         } => {
//             state.camera_state.camera_controller.process_mouse(delta.0, delta.1)
//         }
//         Event::WindowEvent {
//             ref event,
//             window_id,
//         } if window_id == state.window.id() => {
//             if !state.input(event) {
//                 match event {
//                     #[cfg(not(target_arch="wasm32"))]
//                     WindowEvent::CloseRequested
//                     | WindowEvent::KeyboardInput {
//                         input:
//                             KeyboardInput {
//                                 state: ElementState::Pressed,
//                                 virtual_keycode: Some(VirtualKeyCode::Escape),
//                                 ..
//                             },
//                         ..
//                     } => *control_flow = ControlFlow::Exit,
//                     WindowEvent::Resized(physical_size) => {
//                         state.resize(*physical_size);
//                     }
//                     WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                         // new_inner_size is &&mut so we have to dereference it twice
//                         state.resize(**new_inner_size);
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         Event::RedrawRequested(window_id) if window_id == state.window().id() => {
//             let now = instant::Instant::now();
//             let dt = now - last_render_time;
//             last_render_time = now;
//             state.update(dt);
//             match state.render() {
//                 Ok(_) => {}
//                 // Reconfigure the surface if lost
//                 Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
//                 // The system is out of memory, we should probably quit
//                 Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
//                 // All other errors (Outdated, Timeout) should be resolved by the next frame
//                 Err(e) => eprintln!("{:?}", e),
//             }
//         }
//         Event::MainEventsCleared => {
//             // RedrawRequested will only trigger once, unless we manually
//             // request it.
//             state.window().request_redraw();
//         }
//         _ => {}
//     });
// }

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
    scene : scene::Scene,
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
            .filter(|f| f.describe().srgb)
            .next()
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
                }
            ],
            label: Some("texture_bind_group_layout"),
        });

        //Render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });
        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(INDICES),
        //     usage: wgpu::BufferUsages::INDEX,
        // });
        // let num_indices = INDICES.len() as u32;

        let camera_state = camera::CameraState::new(
            &device,
        glm::vec3(5.0, 10.0, 15.0), 
        glm::vec3(0.0, 0.0, 0.0), 
        glm::vec3(0.0, 1.0, 0.0),
        config.width, config.height, 45.0, 0.1, 100.0,
        4.0, 1.0, 0.7,
        );

        // Scene
        let obj_model =
        resources::load_model("island.obj", &device, &queue, &texture_bind_group_layout)
        .await
        .unwrap();
        let instance_vec = vec![
            instance::Instance{transform: glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 
                0.0, 0.0, 0.0, 1.0
            )},
            instance::Instance{transform: glm::mat4(
                1.0, 0.0, 0.0, 10.0,
                0.0, 1.0, 0.0, 2.0,
                0.0, 0.0, 1.0, 2.0, 
                0.0, 0.0, 0.0, 1.0
            )},
        ];
        let scene = scene::Scene{objects: vec![obj_model], instance_vectors: vec![instance_vec]};

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        #[rustfmt::skip]
        let TEST_LIGHTING : Vec<lights::Light> = Vec::from([
            lights::Light{ position: glm::vec4(1.0, 0.0, 0.0, 0.0), color: glm::vec3(1.0, 1.0, 1.0)},
            lights::Light{ position: glm::vec4(-10.0, 0.0, 0.0, 3.0), color: glm::vec3(0.0, 0.2, 0.2)},
        ]);
        let light_state = lights::LightState::new(TEST_LIGHTING, &device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_state.camera_bind_group_layout, &light_state.light_bind_group_layout],
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
            self.camera_state.projection.resize(new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.camera_state.camera_uniform.update_view_proj(&self.camera_state.camera, &self.camera_state.projection);
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
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_state.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_state.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: instant::Duration) {
        self.camera_state.camera_controller.update_camera(&mut self.camera_state.camera, dt);
        self.camera_state.camera_uniform
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
            let instanced_obj = model::InstancedModel::new(
                &self.scene.objects[0], &self.scene.instance_vectors[0], &self.device);

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
            
            let count = self.scene.instance_vectors[0].len();
            render_pass.draw_model_instanced(&instanced_obj, 0..count as u32, &self.camera_state.camera_bind_group);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
