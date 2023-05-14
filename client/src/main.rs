extern crate queues;

use std::env;

use client::audio::{Audio, ConfigAudioAssets, SoundQueue};
use common::configs::from_file;
use log::{debug, error, info};
use std::fs::File;

use bus::Bus;
use env_logger::Builder;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use client::event_loop::PlayerLoop;
use client::inputs::{Input, InputEventProcessor};
use common::communication::commons::*;
use common::communication::message::{HostRole, Message, Payload};
use common::core::events::GameEvent;

use common::core::states::{GameState, ParticleQueue};

const AUDIO_CONFIG_PATH: &str = "audio.json";

fn main() {
    // env::set_var("RUST_BACKTRACE", "1");
    Builder::from_default_env().format_timestamp_micros().init();

    // input channel for communicating between event loop and input processor
    let (tx, rx) = mpsc::channel::<Input>();
    let game_state = Arc::new(Mutex::new(GameState::default()));
    let particle_queue = Arc::new(Mutex::new(ParticleQueue::default()));
    let sound_queue = Arc::new(Mutex::new(SoundQueue::default()));

    let game_events_bus = Bus::new(1);
    // let mut particle_rcvr = game_events_bus.add_rx();

    #[cfg(not(feature = "debug-addr"))]
    let dest: SocketAddr = CSE125_SERVER_ADDR.parse().expect("server addr parse fails");

    #[cfg(feature = "debug-addr")]
    let dest: SocketAddr = DEFAULT_SERVER_ADDR
        .parse()
        .expect("server addr parse fails");

    let protocol = Protocol::connect(dest).unwrap();

    // need to clone the protocol to be able to receive events and game states from different threads
    let mut protocol_clone = protocol.try_clone().unwrap();

    // TODO: make the string path a Const/Static, debug mode for reconnection
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

    // prod
    // write the client_id, session_id to file
    #[cfg(not(feature = "debug-recon"))]
    dump_ids(session_data_path, client_id, session_id);

    // for debug
    #[cfg(feature = "debug-recon")]
    dump_ids(session_data_path, client_id + 1, session_id);

    let mut player_loop =
        PlayerLoop::new(tx, game_state.clone(), particle_queue.clone(), client_id);

    // spawn a thread to handle user inputs (received from event loop)
    thread::spawn(move || {
        let mut input_processor = InputEventProcessor::new(protocol_clone, client_id, rx);

        input_processor.start_poller();

        input_processor.listen();
    });

    // spawn a thread to handle game state updates and events
    let game_state_clone = game_state.clone();
    let sound_queue_clone = sound_queue.clone();
    thread::spawn(move || {
        recv_server_updates(
            protocol.try_clone().unwrap(),
            game_state.clone(),
            particle_queue,
            sound_queue.clone(),
            game_events_bus,
        );
    });

    // thread for audio
    // let game_state_clone1 = game_state.clone();
    thread::spawn(move || {
        let audio_config = from_file::<_, ConfigAudioAssets>(AUDIO_CONFIG_PATH).unwrap();
        let mut audio = Audio::from_config(&audio_config, sound_queue_clone);
        
        audio.play_background_track([0.0, 100.0, 0.0]); // add position of background track to config
        audio.handle_audio_updates(game_state_clone, client_id);
    });      

    pollster::block_on(player_loop.run());
}

fn restore_ids(session_data_path: &PathBuf) -> (u8, u64) {
    match File::open(session_data_path) {
        Err(_) => {
            info!("No session data file found");
            (114, 514)
        }
        Ok(file) => serde_json::from_reader(&file).unwrap(),
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

fn recv_server_updates(
    mut protocol: Protocol,
    game_state: Arc<Mutex<GameState>>,
    particle_queue: Arc<Mutex<ParticleQueue>>,
    sound_queue: Arc<Mutex<SoundQueue>>,
    mut game_events: Bus<GameEvent>,
) {
    // check for new state & update local game state
    while let Ok(msg) = protocol.read_message::<Message>() {
        match msg {
            Message {
                host_role: HostRole::Server,
                payload: Payload::StateSync(update_game_state),
                ..
            } => {
                // update state
                let mut game_state = game_state.lock().unwrap();
                *game_state = update_game_state;
                // according to the state, render world
                debug!("Received game state: {:?}", game_state);
            }
            Message {
                host_role: HostRole::Server,
                payload: Payload::ServerEvent(GameEvent::ParticleEvent(p)),
                ..
            } => {
                print!("Receiving PARTICLE: {:?}...", p);
                particle_queue.lock().unwrap().add_particle(p);
                println!("Done!");
            }
            Message {
                host_role: HostRole::Server,
                payload: Payload::ServerEvent(GameEvent::SoundEvent(s)),
                ..
            } => {
                sound_queue.lock().unwrap().add_sound(s);
            }
            // Message {
            //     host_role: HostRole::Server,
            //     payload: Payload::ServerEvent(update_game_event),
            //     ..
            // } => {
            //     debug!("Received game events: {:?}", update_game_event);

            //     // update state
            //     game_events
            //         .broadcast(update_game_event);
            //         // .try_broadcast(update_game_event)
            //         // .map_err(|e| {
            //         //     error!("Failed to broadcast game event: {:?}", e);
            //         // })
            //         // .unwrap();
            // }
            _ => {}
        }
    }
}
