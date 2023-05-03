use crate::inputs::Input;
use crate::State;
use common::core::states::GameState;
use log::{debug, warn};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct PlayerLoop {
    // commands is a channel that receives commands from the clients (multi-producer, single-consumer)
    inputs: Sender<Input>,

    game_state: Arc<Mutex<GameState>>,

    // current player id
    client_id: u8,
}

impl PlayerLoop {
    /// Creates a new PlayerLoop.
    /// # Arguments
    /// * `commands` - a channel that receives commands from the clients (multi-producer, single-consumer)
    pub fn new(commands: Sender<Input>, game_state: Arc<Mutex<GameState>>, id: u8) -> PlayerLoop {
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
            .with_title("As The Wind Blows")
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(Option::None)))
            .build(&event_loop)
            .unwrap();

        let mut state = State::new(window, self.client_id).await;

        //To check
        let mut last_render_time = instant::Instant::now();

        event_loop.run_return(move |event, _, control_flow| {
            control_flow.set_poll();
            match event {
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
                                // to toggle on/off the background track because it got annoying
                                // match input {
                                //     KeyboardInput {virtual_keycode: Some(VirtualKeyCode::M), ..} => {
                                //         audio.toggle_background_track();
                                //     },
                                //      _ => {},
                                // }
                                
                                match self
                                    .inputs
                                    .send(Input::Keyboard(*input))
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
                            .send(Input::Mouse(output_event))
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

                    state.update(self.game_state.clone(), dt);

                    // send camera position to input processor
                    self.inputs.send(Input::Camera {
                        forward: state.camera_state.camera.forward()
                    }).unwrap();

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
            }
        });
    }
}
