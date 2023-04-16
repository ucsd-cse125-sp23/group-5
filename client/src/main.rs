extern crate queues;
use log::{debug, error, info};
use queues::*;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use winit::event::*;

use client::event_loop::{PlayerLoop, UserInput};
use client::user_input::Inputs;
use common::communication::commons::*;
use common::communication::message::{HostRole, Message, Payload};
use common::core::command::{Command, MoveDirection};
use common::core::states::GameState;

fn handle_keyboard_input(input: KeyboardInput, protocol: &mut Protocol) {
    let mut command: Option<Command> = None;
    match input.virtual_keycode {
        Some(VirtualKeyCode::W) => {
            command = Some(Command::Move(MoveDirection::Forward));
        }
        Some(VirtualKeyCode::A) => {
            command = Some(Command::Move(MoveDirection::Left));
        }
        Some(VirtualKeyCode::S) => {
            command = Some(Command::Move(MoveDirection::Backward));
        }
        Some(VirtualKeyCode::D) => {
            command = Some(Command::Move(MoveDirection::Right));
        }
        Some(VirtualKeyCode::Space) => {
            command = Some(Command::Spawn); // TODO: Place it somewhere else
        }
        _ => {}
    }

    if command.is_none() {
        return;
    }

    protocol
        .send_message(&Message::new(
            HostRole::Client(1),
            Payload::Command(command.clone().unwrap()),
        ))
        .expect("send message fails");
    info!("Sent command: {:?}", command);
}

// mw for mouse wheel, mm for mouse motion
fn handle_mouse_input(
    input: DeviceEvent,
    protocol: &mut Protocol,
    mm_buffer: &mut Queue<DeviceEvent>,
    mw_buffer: &mut Queue<DeviceEvent>,
) {
    match input {
        DeviceEvent::MouseMotion { .. } => {
            mm_buffer.add(input).expect("adding to mm_buffer failed \n");
        }
        DeviceEvent::MouseWheel { .. } => {
            mw_buffer.add(input).expect("adding to mw_buffer failed \n");
        }
        // what's that possibly for?
        DeviceEvent::Button { .. } => {
            // if we receive those button events, then should send right away with protocol
            let mut command: Option<Command> = None;
        }
        _ => {}
    }
}

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel::<UserInput>();
    let game_state = Arc::new(Mutex::new(GameState::default()));

    let server_details = DEFAULT_SERVER_ADDR;
    let dest: SocketAddr = server_details.parse().expect("server details parse fails");

    let mut protocol = Protocol::connect(dest).unwrap();

    let mut client_id: u32 = 0;

    let mut event_loop = PlayerLoop::new(tx, client_id);

    thread::spawn(move || {
        // TODO: Initial Connection to get current client id
        while let Ok(msg) = protocol.read_message::<Message>() {
            match msg {
                Message {
                    host_role: HostRole::Server,
                    payload,
                    ..
                } => match payload {
                    Payload::Init(incoming_id) => {
                        client_id = incoming_id;
                        info!("Connection established ");
                        break;
                    }
                    _ => {
                        error!("first contact with server is not init \n");
                    }
                },
                _ => {
                    error!("first contact with server is not init \n");
                }
            }
        }

        let mut mm_tot_dx;
        let mut mm_tot_dy;
        let mut mw_tot_line_dx;
        let mut mw_tot_line_dy;
        let mut mw_tot_pixel_dx;
        let mut mw_tot_pixel_dy;

        let mut mouse_motion_buf = Queue::new();
        let mut mouse_wheel_buf = Queue::new();
        let mut sample_start_time = Instant::now();
        loop {
            // get input from event
            while let Ok(user_input) = rx.recv() {
                match user_input.input {
                    Inputs::Keyboard(input) => {
                        handle_keyboard_input(input, &mut protocol);
                        break;
                    }
                    Inputs::Mouse(input) => {
                        handle_mouse_input(
                            input,
                            &mut protocol,
                            &mut mouse_motion_buf,
                            &mut mouse_wheel_buf,
                        );
                        break;
                    }
                }
            }

            // also have mouse moved data in parameters (depending on what type of input)
            //      mm_tot_dx mm_tot_dy for mouse motion delta
            //      mw_tot_line_dx mw_tot_line_dy for mouse wheel delta
            //      mw_tot_pixel_dx mw_tot_pixel_dy for track delta
            // sampling: check if mouse movement durations has passed
            let elapsed = sample_start_time.elapsed();
            if elapsed > Duration::from_millis(DEFAULT_MOUSE_MOVEMENT_INTERVAL) {
                mm_tot_dx = 0.0;
                mm_tot_dy = 0.0;
                mw_tot_line_dx = 0.0;
                mw_tot_line_dy = 0.0;
                mw_tot_pixel_dx = 0.0;
                mw_tot_pixel_dy = 0.0;
                let n = mouse_motion_buf.size();
                for _ in 1..n {
                    let mm_event = mouse_motion_buf.remove().unwrap();
                    match mm_event {
                        DeviceEvent::MouseMotion { delta } => {
                            let (dx, dy) = delta;
                            mm_tot_dx += dx;
                            mm_tot_dy += dy;
                        }
                        _ => {
                            error!("non-mouse-motion in mouse motion buffer \n")
                        }
                    }
                    let mw_event = mouse_wheel_buf.remove().unwrap();
                    match mw_event {
                        // more on here
                        // http://who-t.blogspot.com/2015/01/providing-physical-movement-of-wheel.html
                        DeviceEvent::MouseWheel { delta } => match delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                mw_tot_line_dx += dx;
                                mw_tot_line_dy += dy;
                            }
                            MouseScrollDelta::PixelDelta(pixel_delta) => {
                                mw_tot_pixel_dx += pixel_delta.x;
                                mw_tot_pixel_dy += pixel_delta.y;
                            }
                        }
                        _ => {}
                    }
                }
                sample_start_time = Instant::now();
            }

            // check for new state & update local game state
            while let Ok(msg) = protocol.read_message::<Message>() {
                match msg {
                    Message {
                        host_role: HostRole::Server,
                        payload,
                        ..
                    } => {
                        match payload {
                            // what should we do with the logic of Ping

                            // statesync
                            Payload::StateSync(update_game_state) => {
                                // update state
                                let mut game_state = game_state.lock().unwrap();
                                *game_state = update_game_state;
                                // according to the state, render world
                                info!("Received game state: {:?}", game_state);
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                };
            }
            // render world with updated game state

        }
    });

    pollster::block_on(event_loop.run());
}
