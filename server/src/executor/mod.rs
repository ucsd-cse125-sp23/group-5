use crate::executor::command_handlers::{CommandHandler, MoveCommandHandler, SpawnCommandHandler};
use crate::game_loop::ClientCommand;
use crate::simulation::physics_state::PhysicsState;
use common::core::command::{Command, MoveDirection};
use common::core::states::{GameState, PlayerState};
use glam::Vec2;
use log::{debug, warn};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

mod command_handlers;

/// Executor is a struct that is used to execute a command issued by a client.
/// It maintains the state of the game and is responsible for updating it.
pub struct Executor {
    /// state of the game, owned by the executor, encapsulated in a Arc and protected by Mutex
    game_state: Arc<Mutex<GameState>>,
    physics_state: RefCell<PhysicsState>,
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
        Executor {
            game_state,
            physics_state: RefCell::new(PhysicsState::new()),
        }
    }

    /// Executes a command issued by a client.
    pub(crate) fn execute(&self, client_command: ClientCommand) {
        debug!("Executing command: {:?}", client_command);

        // this is a very simple example of how to update the game state
        // based on the command received from the client; maybe we can do something fancier such as
        // dispatch the command to worker threads
        let mut game_state = self.game_state.lock().unwrap();
        let handler: Box<dyn CommandHandler> = match client_command.command {
            Command::Spawn => Box::new(SpawnCommandHandler::new(client_command.client_id)),
            Command::Move(dir) => Box::new(MoveCommandHandler::new(client_command.client_id, dir)),
            _ => panic!("Unsupported command: {:?}", client_command.command),
        };

        if let Err(e) = handler.handle(&mut game_state, &mut self.physics_state.borrow_mut()) {
            warn!("Failed to execute command: {:?}", e);
        }
    }

    pub(crate) fn step(&self, delta_time: f32) {
        self.physics_state.borrow_mut().set_delta_time(delta_time);
        self.physics_state.borrow_mut().step();

        self.sync_states(); // after physics step, need to sync game state
    }

    fn sync_states(&self) {
        let mut game_state = self.game_state.lock().unwrap();
        let physics_state = self.physics_state.borrow();

        // update player positions
        for player in game_state.players.iter_mut() {
            let rigid_body = physics_state.get_entity_rigid_body(player.id).unwrap();
            player.transform.translation = rigid_body.position().translation.vector.into();
            player.transform.rotation = rigid_body.position().rotation.into();
        }
    }

    pub(crate) fn sync_physics_state(&self) {
        let mut game_state = self.game_state.lock().unwrap();
        let physics_state = self.physics_state.borrow();
        for (i, player) in game_state.players.iter_mut().enumerate() {
            // player.transform.translation = physics_state.colliders.
        }
    }

    /// get a clone of the game state
    pub fn game_state(&self) -> GameState {
        self.game_state.lock().unwrap().clone()
    }
}
