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

use client::event_loop::{run, PlayerLoop};
use common::communication::commons::*;
use common::communication::message::Message;
use common::communication::response::Response;

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel();

    let server_details = DEFAULT_SERVER_ADDR;
    let dest: SocketAddr = server_details.parse().expect("server details parse fails");

    let mut conn = Protocol::connect(dest).unwrap();

    thread::spawn(move || {
        let mut event_loop = PlayerLoop::new(tx);
        pollster::block_on(event_loop.run());
    });

    loop {
        // get input from event
        let rx_clone = rx.clone();
        while let Ok(event) = rx_clone.recv() {
            // 1. parser from event to String
            // 2. construct string
            let mut req = Message::new(input);
            // 3. send to server
            conn.send_message(&req).expect("send to server fails");
        }
        // check for new state
        let new = conn.read_message::<Response>().unwrap();
        // update local game state

        // render world
    }
}
