use bus::Bus;
use common::core::states::GameState;
use log::{debug, warn};
use server::game_loop::{ClientCommand, GameLoop, ServerEvent};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
};

use common::communication::commons::{Protocol, DEFAULT_SERVER_ADDR};
use common::communication::message::{HostRole, Message, Payload};
use server::executor::Executor;
use threadpool::ThreadPool;

fn main() {
    env_logger::init();

    let (tx, rx) = mpsc::channel();
    let game_state = Arc::new(Mutex::new(GameState::default()));
    let ext = Executor::new(game_state.clone());

    let running = Arc::new(AtomicBool::new(true));
    let mut broadcast = Arc::new(Mutex::new(Bus::new(1))); // one event at a time

    let listener = TcpListener::bind(DEFAULT_SERVER_ADDR).unwrap();
    let pool = ThreadPool::new(4);

    let mut broadcast_clone = broadcast.clone();
    thread::spawn(move || {
        let mut game_loop = GameLoop::new(rx, &ext, broadcast_clone, running.clone());
        game_loop.run();
    });
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        debug!("New client connected");
        let tx = tx.clone();
        let broadcast_clone = broadcast.clone();
        let game_state = game_state.clone();
        pool.execute(move || {
            let mut rx = broadcast_clone.lock().unwrap().add_rx(); // add a receiver for the first client
            let mut protocol = Protocol::with_stream(stream).unwrap();

            // need to clone the protocol to be able to read and write from different threads
            let mut protocol_clone = protocol.try_clone().unwrap();
            let read_handle = thread::spawn(move || {
                // TODO: handle disconnection and errors
                while let Ok(msg) = protocol_clone.read_message::<Message>() {
                    match msg {
                        Message {
                            host_role: HostRole::Client(client_id),
                            payload,
                            ..
                        } => match payload {
                            Payload::Command(command) => {
                                tx.send(ClientCommand::new(client_id.into(), command))
                                    .unwrap();
                            }
                            Payload::Ping => {
                                protocol_clone
                                    .send_message(&Message::new(HostRole::Server, Payload::Ping))
                                    .unwrap();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            });

            let write_handle = thread::spawn(move || {
                while let Ok(ServerEvent::Sync) = rx.recv() {
                    let game_state = game_state.lock().unwrap();
                    if let Err(e) = protocol.send_message(&Message::new(
                        HostRole::Server,
                        Payload::StateSync(game_state.clone()),
                    )) {
                        warn!("Failed to send message: {:?}", e);
                    }
                }
            });
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // sleep to simulate a slow request
    std::thread::sleep(std::time::Duration::from_secs(5));

    // print the request
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
