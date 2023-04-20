extern crate queues;

use log::{error, info};
use std::net::SocketAddr;
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

    let dest: SocketAddr = DEFAULT_SERVER_ADDR.parse().expect("server addr parse fails");

    let protocol = Protocol::connect(dest).unwrap();

    // need to clone the protocol to be able to receive events and game states from different threads
    let mut protocol_clone = protocol.try_clone().unwrap();

    // init connection with server and get client id
    let client_id = init_connection(&mut protocol_clone).unwrap();

    let mut player_loop = PlayerLoop::new(tx, client_id);

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

fn init_connection(protocol_clone: &mut Protocol) -> Result<u8, ()> {
    while let Ok(msg) = protocol_clone.read_message::<Message>() {
        if let Message {
            host_role: HostRole::Server,
            payload: Payload::Init(incoming_id),
            ..
        } = msg
        {
            info!("Received connection init: {:?}", incoming_id);
            return Ok(incoming_id);
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
        // render world with updated game state
    }
}
