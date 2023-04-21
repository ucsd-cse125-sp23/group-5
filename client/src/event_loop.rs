use std::sync::{Arc, Mutex};
use crate::State;
use crate::inputs::Input;
use log::{debug, info, warn};
use std::sync::mpsc::Sender;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use common::core::states::GameState;

#[derive(Debug)]
pub struct UserInput {
    pub(crate) client_id: u8,
    pub input: Input,
}

impl UserInput {
    pub fn new(client_id: u8, input: Input) -> UserInput {
        UserInput { client_id, input }
    }
}

pub struct PlayerLoop {
    // commands is a channel that receives commands from the clients (multi-producer, single-consumer)
    inputs: Sender<UserInput>,

    game_state: Arc<Mutex<GameState>>,

    // current player id
    client_id: u8,
}

impl PlayerLoop {
    /// Creates a new PlayerLoop.
    /// # Arguments
    /// * `commands` - a channel that receives commands from the clients (multi-producer, single-consumer)
    pub fn new(commands: Sender<UserInput>, game_state: Arc<Mutex<GameState>>, id: u8) -> PlayerLoop {
        PlayerLoop {
            inputs: commands,
            game_state,
            client_id: id,
        }
    }

    /// Starts the game loop.
    pub async fn run(&mut self) {
        let mut event_loop = EventLoop::new();
        let window = WindowBuilder::new()
        .with_title("test")
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(Option::None)))
        .build(&event_loop).unwrap();

        let mut state = State::new(window).await;

        //To check
        let mut last_render_time = instant::Instant::now();

        event_loop.run_return(move |event, _, control_flow| match event {
            // event
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                state.camera_state.camera_controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => {
                            info!("Keyboard input: {:?}", input);
                            match self
                                .inputs
                                .send(UserInput::new(self.client_id, Input::Keyboard(*input)))
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!("Error sending input: {:?}", e);
                                }
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::DeviceEvent { ref event, .. } => match event {
                DeviceEvent::MouseMotion { .. }
                | DeviceEvent::MouseWheel { .. }
                | DeviceEvent::Button { .. } => {
                    let output_event = event.clone();
                    match self
                        .inputs
                        .send(UserInput::new(self.client_id, Input::Mouse(output_event)))
                    {
                        Ok(_) => {}
                        Err(e) => {
                            debug!("Error sending input: {:?}", e);
                        }
                    }
                }
                _ => {}
            },
            // graphics
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                // To check 
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.load_game_state(self.game_state.clone());
                state.update(dt);
                
                // send camera position to input processor
                self.inputs.send(UserInput::new(self.client_id, Input::Camera {
                    position: *state.camera_state.camera.position(),
                    spherical_coords: *state.camera_state.camera.spherical_coords()
                })).unwrap();
                
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        });
    }
}

// struct State {
//     surface: wgpu::Surface,
//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     config: wgpu::SurfaceConfiguration,
//     size: winit::dpi::PhysicalSize<u32>,
//     window: Window,
// }

// impl State {
//     // Creating some of the wgpu types requires async code
//     async fn new(window: Window) -> Self {
//         let size = window.inner_size();

//         // The instance is a handle to our GPU
//         // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
//         let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//             backends: wgpu::Backends::all(),
//             dx12_shader_compiler: Default::default(),
//         });

//         // # Safety
//         //
//         // The surface needs to live as long as the window that created it.
//         // State owns the window so this should be safe.
//         let surface = unsafe { instance.create_surface(&window) }.unwrap();

//         let adapter = instance
//             .request_adapter(&wgpu::RequestAdapterOptions {
//                 power_preference: wgpu::PowerPreference::default(),
//                 compatible_surface: Some(&surface),
//                 force_fallback_adapter: false,
//             })
//             .await
//             .unwrap();

//         let (device, queue) = adapter
//             .request_device(
//                 &wgpu::DeviceDescriptor {
//                     features: wgpu::Features::empty(),
//                     // WebGL doesn't support all of wgpu's features, so if
//                     // we're building for the web we'll have to disable some.
//                     limits: if cfg!(target_arch = "wasm32") {
//                         wgpu::Limits::downlevel_webgl2_defaults()
//                     } else {
//                         wgpu::Limits::default()
//                     },
//                     label: None,
//                 },
//                 None, // Trace path
//             )
//             .await
//             .unwrap();

//         let surface_caps = surface.get_capabilities(&adapter);
//         // Shader code in this tutorial assumes an sRGB surface texture. Using a different
//         // one will result all the colors coming out darker. If you want to support non
//         // sRGB surfaces, you'll need to account for that when drawing to the frame.
//         let surface_format = surface_caps
//             .formats
//             .iter()
//             .copied()
//             .filter(|f| f.describe().srgb)
//             .next()
//             .unwrap_or(surface_caps.formats[0]);
//         let config = wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format: surface_format,
//             width: size.width,
//             height: size.height,
//             present_mode: surface_caps.present_modes[0],
//             alpha_mode: surface_caps.alpha_modes[0],
//             view_formats: vec![],
//         };
//         surface.configure(&device, &config);

//         Self {
//             window,
//             surface,
//             device,
//             queue,
//             config,
//             size,
//         }
//     }

//     pub fn window(&self) -> &Window {
//         &self.window
//     }

//     pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
//         if new_size.width > 0 && new_size.height > 0 {
//             self.size = new_size;
//             self.config.width = new_size.width;
//             self.config.height = new_size.height;
//             self.surface.configure(&self.device, &self.config);
//         }
//     }

//     fn input(&mut self, event: &WindowEvent) -> bool {
//         false
//     }

//     fn update(&mut self) {}

//     fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
//         let output = self.surface.get_current_texture()?;
//         let view = output
//             .texture
//             .create_view(&wgpu::TextureViewDescriptor::default());
//         let mut encoder = self
//             .device
//             .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                 label: Some("Render Encoder"),
//             });
//         {
//             let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Render Pass"),
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &view,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color {
//                             r: 0.1,
//                             g: 0.2,
//                             b: 0.3,
//                             a: 1.0,
//                         }),
//                         store: true,
//                     },
//                 })],
//                 depth_stencil_attachment: None,
//             });
//         }

//         // submit will accept anything that implements IntoIter
//         self.queue.submit(std::iter::once(encoder.finish()));
//         output.present();

//         Ok(())
//     }
// }
