use crate::executor::command_handlers::{
    CommandHandler, JumpCommandHandler, MoveCommandHandler, SpawnCommandHandler,
    StartupCommandHandler, UpdateCameraFacingCommandHandler,
};
use crate::game_loop::ClientCommand;
use crate::simulation::physics_state::PhysicsState;
use common::core::command::{Command, MoveDirection};
use common::core::states::GameState;
use itertools::Itertools;
use log::{debug, error, info, warn};
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

    pub fn init(&self) {
        let mut game_state = self.game_state.lock().unwrap();
        let mut physics_state = self.physics_state.borrow_mut();

        let handler = StartupCommandHandler::new("assets/island.obj".to_string());
        if let Err(e) = handler.handle(&mut game_state, &mut physics_state) {
            panic!("Failed init executor game/physics states: {:?}", e);
        }
    }

    pub(crate) fn plan_and_execute(&self, commands: Vec<ClientCommand>) {
        // aggregate concurrent movement commands from the same client into a single command
        let movement_commands = commands
            .iter()
            .filter(|command| matches!(command.command, Command::Move(_)))
            .into_grouping_map_by(|&command| command.client_id)
            .aggregate(|acc, _key, val| {
                Some(Command::Move(
                    val.command.unwrap_move()
                        + acc
                            .unwrap_or(Command::Move(MoveDirection::zeros()))
                            .unwrap_move(),
                ))
            })
            .iter()
            .map(|(key, val)| ClientCommand {
                client_id: *key,
                command: val.clone(),
            })
            .collect::<Vec<_>>();

        // execute all commands
        commands
            .into_iter()
            .filter(|command| !matches!(command.command, Command::Move(_)))
            .chain(movement_commands.into_iter())
            .for_each(|command| self.execute(command));
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
            Command::UpdateCamera { forward } => Box::new(UpdateCameraFacingCommandHandler::new(
                client_command.client_id,
                forward,
            )),
            Command::Jump => Box::new(JumpCommandHandler::new(client_command.client_id)),
            _ => {
                warn!("Unsupported command: {:?}", client_command.command);
                return;
            }
        };

        if let Err(e) = handler.handle(&mut game_state, &mut self.physics_state.borrow_mut()) {
            error!("Failed to execute command: {:?}", e);
        }
        info!("GameState: {:?}", game_state);
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
            player.transform.translation = rigid_body.position().translation.vector;
            player.transform.rotation = rigid_body.position().rotation.coords.into();
        }
    }

    /// get a clone of the game state
    pub fn game_state(&self) -> GameState {
        self.game_state.lock().unwrap().clone()
    }
}
