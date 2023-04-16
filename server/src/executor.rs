use crate::game_loop::ClientCommand;
use common::core::command::{Command, MoveDirection};
use common::core::states::{GameState, PlayerState};
use glam::Vec2;
use log::debug;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// Executor is a struct that is used to execute a command issued by a client.
/// It maintains the state of the game and is responsible for updating it.
pub struct Executor {
    /// state of the game, owned by the executor, encapsulated in a Arc and protected by Mutex
    game_state: Arc<Mutex<GameState>>,
}

impl Executor {
    /// Creates a new Executor with default game state.
    /// # Examples
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use common::core::states::GameState;
    /// use server::executor::Executor;
    /// let game_state = Arc::new(Mutex::new(GameState::default()));
    /// let executor = Executor::new(game_state);
    pub fn new(game_state: Arc<Mutex<GameState>>) -> Executor {
        Executor { game_state }
    }

    /// Executes a command issued by a client.
    pub(crate) fn execute(&self, client_command: ClientCommand) {
        debug!("Executing command: {:?}", client_command);

        // this is a very simple example of how to update the game state
        // based on the command received from the client; maybe we can do something fancier such as
        // dispatch the command to worker threads
        let mut game_state = self.game_state.lock().unwrap();
        match client_command.command {
            Command::Spawn => {
                game_state.players.push(PlayerState {
                    id: client_command.client_id as usize,
                    ..Default::default()
                });
            }
            Command::Move(dir) => {
                let delta_vec = match dir {
                    MoveDirection::Forward => Vec2::new(0.0, 1.0),
                    MoveDirection::Backward => Vec2::new(0.0, -1.0),
                    MoveDirection::Left => Vec2::new(-1.0, 0.0),
                    MoveDirection::Right => Vec2::new(1.0, 0.0),
                };

                game_state.players[client_command.client_id as usize - 1]
                    .transform
                    .translation += delta_vec.extend(0.0)
            }
            _ => {}
        }
    }

    /// get a clone of the game state
    pub fn game_state(&self) -> GameState {
        self.game_state.lock().unwrap().clone()
    }
}
