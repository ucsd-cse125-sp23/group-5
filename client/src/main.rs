extern crate queues;

use std::env;
use std::fs::File;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use bus::Bus;
use env_logger::Builder;
use log::{debug, error, info};

use client::audio::{Audio, AudioAsset, SoundQueue, AUDIO_POS_AT_CLIENT};
use client::event_loop::PlayerLoop;
use client::inputs::{Input, InputEventProcessor};
use common::communication::commons::*;
use common::communication::message::{HostRole, Message, Payload};
use common::configs::*;
use common::core::events::GameEvent;
use common::core::states::{GameState, ParticleQueue};

use async_std::task;

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

    let dest: SocketAddr = if cfg!(feature = "prod") {
        DEMO_SERVER_ADDR.parse().expect("server addr parse fails")
    } else if cfg!(feature = "debug-remote") {
        CSE125_SERVER_ADDR.parse().expect("server addr parse fails")
    } else if cfg!(feature = "debug-addr") {
        DEFAULT_SERVER_ADDR
            .parse()
            .expect("server addr parse fails")
    } else {
        panic!("No appropriate feature flag set!");
    };

    let protocol = Protocol::connect(dest).unwrap();

    // need to clone the protocol to be able to receive events and game states from different threads
    let mut write_protocol = protocol.try_clone().unwrap();
    let mut read_protocol = protocol.try_clone_into().unwrap();

    // TODO: make the string path a Const/Static, debug mode for reconnection
    let session_data_path = env::current_dir()
        .unwrap()
        .as_path()
        .join("session_data.json");

    let (client_id, session_id) = restore_ids(&session_data_path);

    // send local ids to see if I am a "broken pipe"
    write_protocol
        .send_message(&Message::new(
            HostRole::Client(client_id),
            Payload::Init((client_id, session_id)),
        ))
        .expect("send message fails");

    // init connection with server and get client id
    let (client_id, session_id) = init_connection(&mut read_protocol).unwrap();

    // prod
    // write the client_id, session_id to file
    #[cfg(not(feature = "debug-recon"))]
    dump_ids(session_data_path, client_id, session_id);

    // for debug
    #[cfg(feature = "debug-recon")]
    dump_ids(session_data_path, client_id + 1, session_id);

    // audio blocking flag
    let audio_flag = Arc::new(AtomicBool::new(false));
    let _audio_flag = Arc::clone(&audio_flag);

    // spawn a thread to handle game state updates and events
    let game_state_clone = game_state.clone();
    let sound_queue_clone = sound_queue.clone();
    // thread for audio
    let audio_thread_handle = thread::spawn(move || {
        let config_instance = ConfigurationManager::get_configuration();
        let audio_config = config_instance.audio.clone();

        let mut audio = Audio::from_config(&audio_config, sound_queue_clone);

        // wait till screen is spawned
        while !_audio_flag.load(Ordering::Acquire) {
            thread::park();
        }

        audio.play_background_track(AudioAsset::BKGND_WAIT, AUDIO_POS_AT_CLIENT); // add position of background track to config
        audio.handle_audio_updates(game_state_clone, client_id);
    });

    let mut player_loop = PlayerLoop::new(
        tx,
        game_state.clone(),
        particle_queue.clone(),
        client_id,
        audio_flag,
        audio_thread_handle,
    );

    // spawn a thread to handle user inputs (received from event loop)
    thread::spawn(move || {
        let mut input_processor = InputEventProcessor::new(write_protocol, client_id, rx);

        input_processor.start_poller();

        input_processor.listen();
    });

    thread::spawn(move || {
        recv_server_updates(
            read_protocol,
            game_state.clone(),
            particle_queue,
            sound_queue.clone(),
            game_events_bus,
        );
    });

    task::block_on(player_loop.run());
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

fn init_connection(read_protocol: &mut Protocol) -> Result<(u8, u64), ()> {
    while let Ok(msg) = read_protocol.read_message::<Message>() {
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

#[allow(unreachable_code)]
fn recv_server_updates(
    mut protocol: Protocol,
    game_state: Arc<Mutex<GameState>>,
    particle_queue: Arc<Mutex<ParticleQueue>>,
    sound_queue: Arc<Mutex<SoundQueue>>,
    _game_events: Bus<GameEvent>,
) {
    // check for new state & update local game state
    loop {
        match protocol.read_message::<Message>() {
            Ok(msg) => {
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
                    _ => {}
                }
            }
            Err(e) => {
                error!("Error reading message: {:?}", e);
                exit(1);
                break;
            }
        }
    }
}
