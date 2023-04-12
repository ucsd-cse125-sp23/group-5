use std::{io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, thread};
use std::io::BufWriter;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::AtomicBool;
use bus::Bus;
use common::core::command::Command;
use common::core::states::GameState;
use server::executor::Executor;
use server::game_loop::{ClientCommand, GameLoop, ServerEvent};

use server::thread_pool::ThreadPool;

fn main() {
    env_logger::init();

    let (tx, rx) = mpsc::channel();
    let game_state = Arc::new(Mutex::new(GameState::default()));
    let ext = Executor::new(game_state.clone());
    let running = Arc::new(AtomicBool::new(true));

    let mut broadcast = Arc::new(Mutex::new(Bus::new(1))); // one event at a time


    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    let mut broadcast_clone = broadcast.clone();
    thread::spawn(move || {
        let mut game_loop = GameLoop::new(rx, &ext, broadcast_clone, running.clone());
        game_loop.run();
    });
    let mut client_id = 0;
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let tx = tx.clone();
        let broadcast_clone = broadcast.clone();
        let game_state = game_state.clone();
        pool.execute(move || {
            let mut rx = broadcast_clone.lock().unwrap().add_rx(); // add a receiver for the first client

            let read_stream = stream.try_clone().expect("Failed to clone stream");
            let write_stream = stream;

            // Create a BufReader and BufWriter
            let reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(write_stream);

            // Spawn a thread to handle reading from the stream
            let read_handle = thread::spawn(move || {
                for line in reader.lines() {
                    let line = line.unwrap();
                    // Parse command from JSON
                    let command: Command = serde_json::from_str(&line).unwrap();
                    tx.send(ClientCommand::new(client_id, command)).unwrap();
                }
            });

            // Spawn a thread to handle writing to the stream
            let write_handle = thread::spawn(move || {
                while let Ok(ServerEvent::Sync) = rx.recv() {

                    // Serialize the message as JSON and write it to the stream
                    let game_state = game_state.lock().unwrap();
                    let game_state_json = serde_json::to_string(&*game_state).unwrap();
                    writeln!(&mut writer, "{}", game_state_json).unwrap();
                    writer.flush().unwrap();
                }
            });

            // Wait for the read and write threads to finish
            read_handle.join().unwrap();
            write_handle.join().unwrap();
        });
        client_id += 1;
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
