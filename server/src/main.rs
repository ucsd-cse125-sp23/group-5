use bus::Bus;
use common::core::states::GameState;

use std::sync::atomic::AtomicU8;

use clap::__derive_refs::once_cell::sync::Lazy;
use log::info;
use server::game_loop::GameLoop;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::{net::TcpListener, thread};

#[allow(unused_imports)]
use common::communication::commons::CSE125_SERVER_ADDR;
#[allow(unused_imports)]
use common::communication::commons::DEFAULT_SERVER_ADDR;

use server::executor::Executor;
use threadpool::ThreadPool;

mod client_handler;

use client_handler::ClientHandler;
use common::communication::commons::DEMO_SERVER_ADDR;

pub static CLIENT_ID_ASSIGNER: AtomicU8 = AtomicU8::new(1);
pub static SESSION_ID: Lazy<u64> = Lazy::new(rand::random::<u64>);

fn main() {
    env_logger::init();

    // shared resources between threads for message passing
    let (tx, rx) = mpsc::channel();
    let running = Arc::new(AtomicBool::new(true));
    let broadcast = Arc::new(Mutex::new(Bus::new(1)));

    // game state
    let game_state = Arc::new(Mutex::new(GameState::new()));

    // executor
    let executor = Executor::new(game_state.clone());
    executor.world_init();

    info!("World initialized");

    // start of server listening
    let listener = if cfg!(feature = "prod") {
        TcpListener::bind(DEMO_SERVER_ADDR).unwrap()
    } else if cfg!(feature = "debug-remote") {
        TcpListener::bind(CSE125_SERVER_ADDR).unwrap()
    } else if cfg!(feature = "debug-addr") {
        TcpListener::bind(DEFAULT_SERVER_ADDR).unwrap()
    } else {
        panic!("No appropriate feature flag set!");
    };

    let pool = ThreadPool::new(4);

    // starting game loop
    let broadcast_clone = broadcast.clone();
    thread::spawn(move || {
        GameLoop::new(rx, &executor, broadcast_clone, running.clone()).run();
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // cloning pointers to shared resources for each client
        let tx = tx.clone();
        let broadcast_clone = broadcast.clone();
        let game_state = game_state.clone();

        pool.execute(move || {
            ClientHandler::new(stream, tx, broadcast_clone, game_state).run();
        });
    }
}
