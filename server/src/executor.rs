use crate::game_loop::ClientCommand;
use common::core::command::Command;
use common::core::states::{GameState, PlayerState};
use log::debug;
use std::cell::RefCell;

/// Executor is a struct that is used to execute a command issued by a client.
/// It maintains the state of the game and is responsible for updating it.
pub struct Executor {
    /// state of the game, owned by the executor, encapsulated in a RefCell to allow for interior
    /// mutability
    game_state: RefCell<GameState>,
}

impl Executor {
    /// Creates a new Executor with default game state.
    /// # Examples
    /// ```
    /// use server::executor::Executor;
    /// let executor = Executor::new();
    pub fn new() -> Executor {
        Executor {
            game_state: RefCell::new(GameState::default()),
        }
    }

    /// Executes a command issued by a client.
    pub(crate) fn execute(&self, client_command: ClientCommand) {
        debug!("Executing command: {:?}", client_command);

        // this is a very simple example of how to update the game state
        // based on the command received from the client; maybe we can do something fancier such as
        // dispatch the command to worker threads
        match client_command.command {
            Command::Spawn => {
                self.game_state
                    .borrow_mut()
                    .players
                    .push(PlayerState::default());
            }
            Command::Move(_) => {
                self.game_state.borrow_mut().players[0]
                    .transform
                    .translation
                    .x += 1.0;
            }
            _ => {}
        }
    }

    /// get a clone of the game state
    pub(crate) fn game_state(&self) -> GameState {
        self.game_state.borrow().clone()
    }
}
