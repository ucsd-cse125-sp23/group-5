use instant::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use log::{debug, warn};
use nalgebra_glm as glm;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use common::core::states::{GameState, ParticleQueue};

use crate::inputs::Input;
use crate::State;

pub struct PlayerLoop {
    // commands is a channel that receives commands from the clients (multi-producer, single-consumer)
    inputs: Sender<Input>,
    game_state: Arc<Mutex<GameState>>,
    particle_queue: Arc<Mutex<ParticleQueue>>,
    // current player id
    client_id: u8,
    // audio flag
    audio_flag: Arc<AtomicBool>,
    audio_thread_handle: JoinHandle<()>,
}

impl PlayerLoop {
    /// Creates a new PlayerLoop.
    /// # Arguments
    /// * `commands` - a channel that receives commands from the clients (multi-producer, single-consumer)
    pub fn new(
        commands: Sender<Input>,
        game_state: Arc<Mutex<GameState>>,
        particle_queue: Arc<Mutex<ParticleQueue>>,
        id: u8,
        audio_flag: Arc<AtomicBool>,
        audio_thread_handle: JoinHandle<()>,
    ) -> PlayerLoop {
        PlayerLoop {
            inputs: commands,
            game_state,
            particle_queue,
            client_id: id,
            audio_flag,
            audio_thread_handle,
        }
    }

    /// Starts the game loop.
    pub async fn run(&mut self) {
        let mut event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("As The Wind Blows")
            // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(Option::None)))
            .with_window_icon(Some(
                winit::window::Icon::from_rgba(
                    vec![
                        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
                    ],
                    2,
                    2,
                )
                .unwrap(),
            ))
            .build(&event_loop)
            .unwrap();

        // let start = Instant::now();
        let mut state = State::new(
            window,
            self.client_id,
            self.inputs.clone(),
            self.game_state.clone(),
        )
        .await;

        // let duration = start.elapsed();
        // println!("state construction took: {:?}", duration);

        // notify audio thread to play bg track
        self.audio_flag.store(true, Ordering::Release);
        self.audio_thread_handle.thread().unpark();

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
                                // FOR TESTING TO FIND CORRECT ORIENTATION OF CHARACTERS
                                if state.display.current == "display:lobby" {
                                    let scene = state.display.scene_map.get_mut("scene:lobby").unwrap();
                                    let mut node: &mut crate::scene::Node = &mut crate::scene::Node::new("random".to_string());
                                    let mut rot = nalgebra::Quaternion::new(1.0, 0.0, 0.0, 0.0);
                                    match scene.scene_graph.get_mut("object:player_model") {
                                        None => (),
                                        Some(n) => node = n,
                                    }

                                    match input {
                                        KeyboardInput { virtual_keycode: Some(VirtualKeyCode::W), .. } => {
                                            rot = glm::quat_rotate(&rot, -glm::pi::<f32>() / 10.0, &glm::vec3(1.0, 0.0, 0.0));
                                        }
                                        KeyboardInput { virtual_keycode: Some(VirtualKeyCode::A), .. } => {
                                            rot = glm::quat_rotate(&rot, -glm::pi::<f32>() / 10.0, &glm::vec3(0.0, 1.0, 0.0));
                                        }
                                        KeyboardInput { virtual_keycode: Some(VirtualKeyCode::S), .. } => {
                                            rot = glm::quat_rotate(&rot, glm::pi::<f32>() / 10.0, &glm::vec3(1.0, 0.0, 0.0));
                                        }
                                        KeyboardInput { virtual_keycode: Some(VirtualKeyCode::D), .. } => {
                                            rot = glm::quat_rotate(&rot, glm::pi::<f32>() / 10.0, &glm::vec3(0.0, 1.0, 0.0));
                                        }
                                        _ => {}
                                    }
                                    node.transform *= glm::quat_to_mat4(&rot);
                                    println!("QUAT: {:#?}", glm::to_quat(&node.transform.clone()));
                                    scene.draw_scene_dfs();
                                    std::thread::sleep(instant::Duration::new(0, 10));
                                }
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
                    event: DeviceEvent::MouseMotion { delta, },
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

                    state.update(self.game_state.clone(), self.particle_queue.clone(), dt);

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
