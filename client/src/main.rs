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

    protocol.send_message(&Message::new(
        HostRole::Client(1),
        Payload::Command(command.clone().unwrap()),
    ));
    info!("Sent command: {:?}", command);
}

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel::<UserInput>();

    let server_details = DEFAULT_SERVER_ADDR;
    let dest: SocketAddr = server_details.parse().expect("server details parse fails");

    let mut protocol = Protocol::connect(dest).unwrap();

    let mut event_loop = PlayerLoop::new(tx);

    thread::spawn(move || {
        loop {
            // get input from event
            while let Ok(user_input) = rx.recv() {
                match user_input.input {
                    Inputs::Keyboard(input) => handle_keyboard_input(input, &mut protocol),
                    _ => {}
                }
            }
        }
    });


    // check for new state
    // let new = conn.read_message::<Response>().unwrap();
    // update local game state

    // render world

    pollster::block_on(event_loop.run());
}
