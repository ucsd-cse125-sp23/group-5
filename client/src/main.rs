use std::io::{self, prelude::*, BufReader, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::{str, thread};
use log::{debug, info};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use client::event_loop::{PlayerLoop, UserInput};
use common::communication::commons::*;
use common::communication::message::{HostRole, Message, Payload};
use client::user_input::Inputs;
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

    protocol.send_message(
        &Message::new(
            HostRole::Client(1),
            Payload::Command(command.clone().unwrap()),
        )).expect("send message fails");
    info!("Sent command: {:?}", command);
}

fn handle_mouse_input(input: DeviceEvent, protocol: &mut protocol) {
    let mut command: Option<Command> = None;
    match input {
        DeviceEvent::MouseMotion { delta } => {

        }
        DeviceEvent::MouseWheel { delta } => {

        }
        DeviceEvent::Button { button, state } => {

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

    let mut event_loop = PlayerLoop::new(tx);

    thread::spawn(move || {
        loop {
            // get input from event
            while let Ok(user_input) = rx.recv() {
                match user_input.input {
                    Inputs::Keyboard(input) => {
                        handle_keyboard_input(input, &mut protocol);
                        break;
                    },
                    Input::Mouse(input) => {
                        handle_mouse_input(input, &mut protocol);
                        break;
                    },
                    _ => {}
                }
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
                    },
                    _ => {}
                };
            }
            // render world with updated game state
        }
    });

    pollster::block_on(event_loop.run());
}
