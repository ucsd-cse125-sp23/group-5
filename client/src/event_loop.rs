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

    // current player id
    client_id: u8,
}

impl PlayerLoop {
    /// Creates a new PlayerLoop.
    /// # Arguments
    /// * `commands` - a channel that receives commands from the clients (multi-producer, single-consumer)
    pub fn new(commands: Sender<UserInput>, id: u8) -> PlayerLoop {
        PlayerLoop {
            inputs: commands,
            client_id: id,
        }
    }

    /// Starts the game loop.
    pub async fn run(&mut self) {
        let mut event_loop = EventLoop::new();
        let window = WindowBuilder::new()
        .with_title("As the Wind Blows")
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(Option::None)))
        .build(&event_loop).unwrap();

        let mut state = State::new(window).await;

        //To check
        let mut last_render_time = instant::Instant::now();

        event_loop.run_return(move |event, _, control_flow| match event {
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
            // event
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                state.player_controller.process_mouse(delta.0, delta.1)
            }
            Event::DeviceEvent { ref event, .. } => match event {
                DeviceEvent::MouseWheel { .. }
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
                state.update(dt);
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