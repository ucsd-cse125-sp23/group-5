use crate::{CLIENT_ID_ASSIGNER, SESSION_ID};
use bus::{Bus, BusReader};
use common::communication::commons::Protocol;
use common::communication::message::{HostRole, Message, Payload};
use common::core::states::GameState;
use log::{debug, error, info, warn};
use server::game_loop::{ClientCommand, ServerEvent};
use std::net::TcpStream;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ClientHandler {
    protocol: Protocol,
    tx: mpsc::Sender<ClientCommand>,
    rx: BusReader<ServerEvent>,
    game_state: Arc<Mutex<GameState>>,
}

impl ClientHandler {
    pub fn new(
        stream: TcpStream,
        tx: mpsc::Sender<ClientCommand>,
        broadcast: Arc<Mutex<Bus<ServerEvent>>>,
        game_state: Arc<Mutex<GameState>>,
    ) -> Self {
        // add a new rx to the broadcast bus
        let rx = broadcast.lock().unwrap().add_rx();

        // create a new protocol with the stream
        let protocol = Protocol::with_stream(stream).unwrap();

        ClientHandler {
            protocol,
            tx,
            rx,
            game_state,
        }
    }

    pub fn run(mut self) {
        let read_protocol = self.protocol.try_clone().unwrap();
        let write_protocol = self.protocol.try_clone().unwrap();
        let client_id;

        // connect with client
        if let Ok(msg) = self.protocol.read_message::<Message>() {
            if let Message {
                host_role: HostRole::Client(_),
                payload: Payload::Init((incoming_client_id, incoming_session_id)),
                ..
            } = msg
            {
                if !SESSION_ID.cmp(&incoming_session_id).is_eq() {
                    info!("New client connected");
                    client_id = CLIENT_ID_ASSIGNER.fetch_add(1, Ordering::SeqCst);
                } else {
                    info!("Client reconnected");
                    client_id = incoming_client_id;
                }
                self.protocol
                    .send_message(&Message::new(
                        HostRole::Server,
                        Payload::Init((
                            HostRole::Client(client_id).into(),
                            SESSION_ID.to_owned(),
                        )),
                    ))
                    .expect("send message fails");
            } else {
                error!("Unexpected message before connection init: {:?}", msg);
            }
        }

        let read_handler = thread::spawn(move || {
            let mut read_resources = (read_protocol, self.tx);
            Self::read_messages(&mut read_resources);
        });

        let write_handler = thread::spawn(move || {
            let mut write_resources = (write_protocol, self.rx, self.game_state);
            Self::write_messages(&mut write_resources);
        });

        read_handler.join().unwrap();
        write_handler.join().unwrap();
        warn!("Client disconnected");
    }

    fn read_messages(resources: &mut (Protocol, mpsc::Sender<ClientCommand>)) {
        let (protocol, tx) = resources;
        while let Ok(msg) = protocol.read_message::<Message>() {
            if let Message {
                host_role: HostRole::Client(client_id),
                payload,
                ..
            } = msg
            {
                match payload {
                    Payload::Command(command) => {
                        tx.send(ClientCommand::new(client_id.into(), command))
                            .unwrap();
                    }
                    Payload::Ping => {
                        protocol
                            .send_message(&Message::new(HostRole::Server, Payload::Ping))
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    fn write_messages(resources: &mut (Protocol, BusReader<ServerEvent>, Arc<Mutex<GameState>>)) {
        let (protocol, rx, game_state) = resources;
        while let Ok(ServerEvent::Sync) = rx.recv() {
            debug!("Updating game state to client");
            let game_state = game_state.lock().unwrap();
            if let Err(e) = protocol.send_message(&Message::new(
                HostRole::Server,
                Payload::StateSync(game_state.clone()),
            )) {
                match e.kind() {
                    std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset => break,
                    _ => {
                        warn!("Error while sending message to client: {:?}", e);
                    }
                }
            }
        }
    }
}
