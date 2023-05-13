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
use common::core::states::GameLifeCycleState::{Running, Waiting};

const TICK_RATE: u64 = 30; // 30 fps

/// Wrapper around a `Command` that also contains the id of the client that issued the command.
#[derive(Debug, Clone)]
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

            // consume and collect all messages in the channel
            let mut commands = self.commands.try_iter().collect::<Vec<_>>();

            // automatically spawning the 4 players if gamestate is running now
            let mut commands = self.executor.game_init(commands);

            // update list of dead players and issue die commands
            let dead_players = self.executor.update_dead_players();
            if !dead_players.is_empty() {
                for client_id in dead_players {
                    commands.push(ClientCommand::new(client_id, Command::Die));
                }
            }

            // check whether dead players need to respawn and issue spawn commands
            let players_to_respawn = self.executor.check_respawn_players();
            if !players_to_respawn.is_empty() {
                for client_id in players_to_respawn {
                    commands.push(ClientCommand::new(client_id, Command::Spawn));
                }
            }

            // send commands to the executor
            self.executor.plan_and_execute(commands);

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
mod tests {}
