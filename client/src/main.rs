extern crate queues;

use std::env;

use log::{error, info};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use client::event_loop::{PlayerLoop, UserInput};
use client::inputs::InputProcessor;
use common::communication::commons::*;
use common::communication::message::{HostRole, Message, Payload};

use common::core::states::GameState;

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel::<UserInput>();
    let game_state = Arc::new(Mutex::new(GameState::default()));

    let dest: SocketAddr = DEFAULT_SERVER_ADDR
        .parse()
        .expect("server addr parse fails");

    let protocol = Protocol::connect(dest).unwrap();

    // need to clone the protocol to be able to receive events and game states from different threads
    let mut protocol_clone = protocol.try_clone().unwrap();

    let session_data_path = env::current_dir()
        .unwrap()
        .as_path()
        .join("session_data.json");

    let (client_id, session_id) = restore_ids(&session_data_path);

    // send local ids to see if I am a "broken pipe"
    protocol_clone
        .send_message(&Message::new(
            HostRole::Client(client_id),
            Payload::Init((client_id, session_id)),
        ))
        .expect("send message fails");

    // init connection with server and get client id
    let (client_id, session_id) = init_connection(&mut protocol_clone).unwrap();

    // write the client_id, session_id to file
    dump_ids(session_data_path, client_id, session_id);

    let mut player_loop = PlayerLoop::new(tx, game_state.clone(), client_id);

    // spawn a thread to handle user inputs (received from event loop)
    thread::spawn(move || {
        InputProcessor::new(protocol_clone, client_id, rx).run();
    });

    // spawn a thread to handle game state updates
    thread::spawn(move || {
        game_state_update_loop(protocol.try_clone().unwrap(), game_state);
    });

    pollster::block_on(player_loop.run());
}

fn restore_ids(session_data_path: &PathBuf) -> (u8, u64) {
    match File::open(session_data_path) {
        Err(_) => {
            info!("No session data file found");
            (114, 514)
        }
        Ok(file) => serde_json::from_reader(&file).unwrap()
    }
}

fn dump_ids(session_data_path: PathBuf, client_id: u8, session_id: u64) {
    let file = File::create(session_data_path).unwrap();
    let ids = (client_id, session_id);
    serde_json::to_writer(&file, &ids).unwrap();
}

fn init_connection(protocol_clone: &mut Protocol) -> Result<(u8, u64), ()> {
    while let Ok(msg) = protocol_clone.read_message::<Message>() {
        if let Message {
            host_role: HostRole::Server,
            payload: Payload::Init(incoming_ids),
            ..
        } = msg
        {
            info!("Received connection init: {:?}", incoming_ids);
            return Ok(incoming_ids);
        } else {
            error!("Unexpected message before connection init: {:?}", msg);
        }
    }
    Err(())
}

fn game_state_update_loop(mut protocol: Protocol, game_state: Arc<Mutex<GameState>>) {
    loop {
        // check for new state & update local game state
        while let Ok(msg) = protocol.read_message::<Message>() {
            if let Message {
                host_role: HostRole::Server,
                payload: Payload::StateSync(update_game_state),
                ..
            } = msg
            {
                // update state
                let mut game_state = game_state.lock().unwrap();
                *game_state = update_game_state;
                // according to the state, render world
                info!("Received game state: {:?}", game_state);
                break;
            };
        }
    }
}
