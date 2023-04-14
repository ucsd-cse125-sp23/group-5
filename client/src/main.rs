use std::io::{self, prelude::*, BufReader, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::{str, thread};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use client::event_loop::{PlayerLoop, UserInput};
use common::communication::commons::*;
use common::communication::message::Message;
use client::user_input::Inputs;

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel();

    let server_details = DEFAULT_SERVER_ADDR;
    let dest: SocketAddr = server_details.parse().expect("server details parse fails");

    let mut conn = Protocol::connect(dest).unwrap();

    let mut input: UserInput;
    let mut event_loop = PlayerLoop::new(tx, &input);


    thread::spawn(move || {
        pollster::block_on(event_loop.run());
    });

    loop {
        // get input from event
        while let Ok(keyboard_in) = rx.recv() {
            let mut keystr= String::new();
            // let tx_clone = tx.clone();
            // 1. parser from event to String
            if let Some(keycode) = keyboard_in.virtual_keycode {
                if keyboard_in.state == ElementState::Pressed {
                    // if let Some(keychar) = keycode. {
                    //     let keystr = keychar.to_string();
                    // }
                }
            }
            keyboard_in.virtual_keycode;
            // 2. construct string
            // let mut req = Message::new(input);
            // 3. send to server
            // conn.send_message(&req).expect("send to server fails");
        }
        // check for new state
        // let new = conn.read_message::<Response>().unwrap();
        // update local game state

        // render world
    }
}
