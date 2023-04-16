use bus::Bus;
use common::core::states::GameState;
use log::{debug, warn};
use server::game_loop::{ClientCommand, GameLoop, ServerEvent};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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

    // some multithreading primitives
    // channel for sending events to the game loop
    let (tx, rx) = mpsc::channel();
    // atomic bool to signal when to stop the game loop
    let running = Arc::new(AtomicBool::new(true));
    // bus for broadcasting events to all clients
    let mut broadcast = Arc::new(Mutex::new(Bus::new(1))); // one event at a time

    // create game state which will be shared among all threads
    let game_state = Arc::new(Mutex::new(GameState::default()));

    // create executor and init game state
    let executor = Executor::new(game_state.clone());
    executor.init();

    // start listening for new clients
    let listener = TcpListener::bind(DEFAULT_SERVER_ADDR).unwrap();
    let pool = ThreadPool::new(4);

    let broadcast_clone = broadcast.clone();
    thread::spawn(move || {
        let mut game_loop = GameLoop::new(rx, &executor, broadcast_clone, running.clone());
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
            // initialize connection
            protocol_clone.send_message(&Message::new(HostRole::Server, Payload::Init(HostRole::Client(1).into()))).expect("send message fails");
            debug!("Sending initialization request");

            let read_handle = thread::spawn(move || {
                // TODO: handle disconnection and errors
                while let Ok(msg) = protocol_clone.read_message::<Message>() {
                    if let Message {
                            host_role: HostRole::Client(client_id),
                            payload,
                            ..
                        } = msg { match payload {
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
                    } }
                }
            });

            let write_handle = thread::spawn(move || {

                // then proceeds on to later tasks
                while let Ok(ServerEvent::Sync) = rx.recv() {
                    debug!("Updating game state to client");
                    let game_state = game_state.lock().unwrap();
                    if let Err(e) = protocol.send_message(&Message::new(
                        HostRole::Server,
                        Payload::StateSync(game_state.clone()),
                    )) {
                        if matches!(e.kind(), std::io::ErrorKind::BrokenPipe) {
                            warn!("Client disconnected");
                            break;
                        }
                    }
                }
            });

            read_handle.join().unwrap();
            write_handle.join().unwrap();
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
