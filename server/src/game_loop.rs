use crate::executor::Executor;
use crate::outgoing_request::{OutgoingRequest, RequestKind};
use crate::Recipients;
use bus::Bus;
use common::core::command::Command;
use log::debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

const TICK_RATE: u64 = 30; // 30 fps

/// Wrapper around a `Command` that also contains the id of the client that issued the command.
#[derive(Debug)]
pub struct ClientCommand {
    pub(crate) client_id: u32,
    pub command: Command,
}

impl ClientCommand {
    pub fn new(client_id: u32, command: Command) -> ClientCommand {
        ClientCommand { client_id, command }
    }
}

pub struct GameLoop<'a> {
    // commands is a channel that receives commands from the clients (multi-producer, single-consumer)
    commands: Receiver<ClientCommand>,

    // executor is used to execute the commands received from the clients
    executor: &'a Executor,

    // broadcast is used to broadcast events to the clients (single-producer, multi-consumer)
    broadcast: Arc<Mutex<Bus<OutgoingRequest>>>,

    // used to stop the game loop (mostly for testing and debugging purposes)
    running: Arc<AtomicBool>,
}

impl GameLoop<'_> {
    /// Creates a new GameLoop.
    /// # Arguments
    /// * `commands` - a channel that receives commands from the clients (multi-producer, single-consumer)
    /// * `executor` - used to execute the commands received from the clients
    /// * `broadcast` - used to broadcast events to the clients (single-producer, multi-consumer)
    /// * `running` - used to stop the game loop (mostly for testing and debugging purposes)
    pub fn new(
        commands: Receiver<ClientCommand>,
        executor: &Executor,
        broadcast: Arc<Mutex<Bus<OutgoingRequest>>>,
        running: Arc<AtomicBool>,
    ) -> GameLoop {
        GameLoop {
            commands,
            executor,
            broadcast,
            running,
        }
    }

    /// Starts the game loop.
    pub fn run(&mut self) {
        let mut last_instant = Instant::now(); // used to calculate the delta time

        while self.running.load(Ordering::SeqCst) {
            let tick_start = Instant::now();

            // check whether player need to respawn
            let players_to_respawn = self.executor.check_respawn_players();
            if players_to_respawn.is_empty() {
                // consume and collect all messages in the channel
                let commands = self.commands.try_iter().collect::<Vec<_>>();
                // send commands to the executor
                self.executor.plan_and_execute(commands);
            } else {
                for client_id in players_to_respawn {
                    self.executor
                        .plan_and_execute(vec![ClientCommand::new(client_id, Command::Respawn)]);
                }
            }

            // calculate the delta time
            let delta_time = Instant::now().duration_since(last_instant);
            last_instant = Instant::now();

            // executor step physics and sync game state
            self.executor.step(delta_time.as_secs_f32());

            let mut broadcast = self.broadcast.lock().unwrap();

            // broadcast game sync to all clients, in a blocking way
            broadcast.broadcast(OutgoingRequest::new(
                RequestKind::SyncGameState,
                Recipients::All,
            ));

            // broadcast game events collected from the executor to all clients
            let events = self.executor.collect_game_events();
            for (event, recipients) in events {
                broadcast.broadcast(OutgoingRequest::new(
                    RequestKind::SendGameEvent(event),
                    recipients,
                ));
            }

            // wait for the fixed interval tick
            let elapsed = tick_start.elapsed();
            if elapsed < Duration::from_millis(TICK_RATE) {
                sleep(Duration::from_millis(TICK_RATE) - elapsed);
            } else {
                // this should usually not happen unless the server is under heavy load
                debug!("Tick took too long: {:?}", elapsed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::core::command::Command::UpdateCamera;

    use common::core::states::GameState;
    use nalgebra_glm::{vec3, Vec3};
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_game_loop() {
        let (tx, rx) = mpsc::channel();
        let game_state = Arc::new(Mutex::new(GameState::default()));
        let ext = Executor::new(game_state);
        let running = Arc::new(AtomicBool::new(true));

        let broadcast = Arc::new(Mutex::new(Bus::new(1))); // one event at a time

        let mut game_loop = GameLoop::new(rx, &ext, broadcast.clone(), running.clone());

        let tx_clone = tx.clone();

        // client 0 spawns a player at 500ms
        // stop the game loop at 500ms
        let broadcast_clone = broadcast.clone();
        std::thread::spawn(move || {
            let mut rx1 = broadcast_clone.lock().unwrap().add_rx(); // add a receiver for the first client
            tx_clone
                .send(ClientCommand::new(1, Command::Spawn))
                .unwrap();
            tx_clone
                .send(ClientCommand::new(
                    1,
                    UpdateCamera {
                        forward: nalgebra_glm::vec3(2., 0., 1.),
                    },
                ))
                .unwrap();
            sleep(Duration::from_millis(500));

            assert_eq!(
                rx1.try_recv(),
                Ok(OutgoingRequest::new(
                    RequestKind::SyncGameState,
                    Recipients::All
                ))
            ); // the game state should have been synced
            assert!(rx1.try_recv().is_err()); // the message is consumed by the first receiver

            running.store(false, Ordering::SeqCst);
        });

        // client 0 (from another thread) moves the player at 50ms
        let broadcast_clone = broadcast;
        std::thread::spawn(move || {
            let mut rx2 = broadcast_clone.lock().unwrap().add_rx(); // add a receiver for the second client

            sleep(Duration::from_millis(250));

            assert_eq!(
                rx2.try_recv(),
                Ok(OutgoingRequest::new(
                    RequestKind::SyncGameState,
                    Recipients::All
                ))
            ); // the game state should have been synced by now

            tx.send(ClientCommand::new(1, Command::Move(vec3(1., 0., 0.))))
                .unwrap();
        });

        game_loop.run(); // this should block until the game loop is stopped at 500ms

        assert_eq!(ext.game_state().players.len(), 1); // the player should have been spawned
        assert_ne!(
            ext.game_state()
                .players
                .get(&1_u32)
                .unwrap()
                .transform
                .translation,
            Vec3::new(0.0, 0.0, 0.0)
        ); // the player should have moved
    }
}
