use bus::Bus;
use common::core::states::GameState;

use std::sync::atomic::AtomicU8;

use clap::__derive_refs::once_cell::sync::Lazy;
use server::game_loop::GameLoop;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::{net::TcpListener, thread};

use common::communication::commons::DEFAULT_SERVER_ADDR;

use server::executor::Executor;
use threadpool::ThreadPool;

mod client_handler;

use client_handler::ClientHandler;

pub static CLIENT_ID_ASSIGNER: AtomicU8 = AtomicU8::new(1);
pub static SESSION_ID: Lazy<u64> = Lazy::new(rand::random::<u64>);

fn main() {
    env_logger::init();

    // shared resources between threads for message passing
    let (tx, rx) = mpsc::channel();
    let running = Arc::new(AtomicBool::new(true));
    let broadcast = Arc::new(Mutex::new(Bus::new(1)));

    // game state
    let game_state = Arc::new(Mutex::new(GameState::default()));

    // executor
    let executor = Executor::new(game_state.clone());
    executor.init();

    // start of server listening
    let listener = TcpListener::bind(DEFAULT_SERVER_ADDR).unwrap();
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
