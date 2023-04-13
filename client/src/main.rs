use std::io::{self,prelude::*,BufReader,Write};
use std::{str, thread};
use std::net::SocketAddr;
use std::net::TcpStream;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use client::run;
use common::communication::commons::*;
use common::communication::request::Request;
use common::communication::response::Response;

fn main(){
    env_logger::init();
    let server_details = DEFAULT_SERVER_ADDR;
    let dest: SocketAddr = server_details.parse().expect("server details parse fails");

    let mut conn = Protocol::connect(dest).unwrap();

    thread::spawn(move || {
        pollster::block_on(run());
    });

    loop {
        // send input to server
        let mut input = String::new();
        // how can I get the state update from here????

        let mut req = Request::new(input);
        conn.send_message(&req).unwrap();
        // check for new state
        let new = conn.read_message::<Response>().unwrap();
        // update local game state


        // render world
    }
}
