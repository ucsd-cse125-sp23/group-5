use crate::executor::command_handlers::{
    CommandHandler, JumpCommandHandler, MoveCommandHandler, RespawnCommandHandler,
    SpawnCommandHandler, StartupCommandHandler, UpdateCameraFacingCommandHandler,
};
use crate::game_loop::ClientCommand;
use crate::simulation::physics_state::PhysicsState;
use common::core::command::{Command, MoveDirection};
use common::core::states::GameState;

use crate::Recipients;
use common::core::events::GameEvent;
use itertools::Itertools;
use log::{debug, error, info, warn};
use std::cell::{RefCell, RefMut};
use std::sync::{Arc, Mutex};
use common::configs::from_file;
use common::configs::scene_config::ConfigSceneGraph;

mod command_handlers;

pub const DEFAULT_RESPAWN_LIMIT: f32 = -20.0; // 5ms
/// Executor is a struct that is used to execute a command issued by a client.
/// It maintains the state of the game and is responsible for updating it.
pub struct Executor {
    /// state of the game, owned by the executor, encapsulated in a Arc and protected by Mutex
    game_state: Arc<Mutex<GameState>>,
    physics_state: RefCell<PhysicsState>,
    game_events: RefCell<Vec<(GameEvent, Recipients)>>,
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
            game_events: RefCell::new(Vec::new()),
        }
    }

    pub fn init(&self) {
        let mut game_state = self.game_state.lock().unwrap();
        let mut physics_state = self.physics_state.borrow_mut();
        let mut game_events = self.game_events.borrow_mut();

        let scene_config = from_file("scene.json").unwrap();
        let models_config = from_file("models.json").unwrap();


        let handler = StartupCommandHandler::new(models_config, scene_config);

        if let Err(e) = handler.handle(&mut game_state, &mut physics_state, &mut game_events) {
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

        let mut game_state = self.game_state.lock().unwrap();
        let mut physics_state = self.physics_state.borrow_mut();
        let mut game_events = self.game_events.borrow_mut();

        let handler: Box<dyn CommandHandler> = match client_command.command {
            Command::Spawn => Box::new(SpawnCommandHandler::new(client_command.client_id)),
            Command::Respawn => Box::new(RespawnCommandHandler::new(client_command.client_id)),
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

        if let Err(e) = handler.handle(&mut game_state, &mut physics_state, &mut game_events) {
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
        for (_id, player) in game_state.players.iter_mut() {
            let rigid_body = physics_state.get_entity_rigid_body(player.id).unwrap();
            player.transform.translation = rigid_body.position().translation.vector;
            player.transform.rotation = rigid_body.position().rotation.coords.into();
        }
    }

    pub(crate) fn collect_game_events(&self) -> Vec<(GameEvent, Recipients)> {
        self.game_events.replace(Vec::new())
    }

    pub(crate) fn check_respawn_players(&self) -> Vec<u32> {
        self.game_state()
            .players
            .iter()
            .filter(|(_, player)| player.transform.translation.y < DEFAULT_RESPAWN_LIMIT)
            .map(|(&id, _)| id)
            .collect::<Vec<_>>()
    }

    /// get a clone of the game state
    pub fn game_state(&self) -> GameState {
        self.game_state.lock().unwrap().clone()
    }
}

type GameEventWithRecipients = (GameEvent, Recipients);

pub trait GameEventCollector {
    fn add(&mut self, event: GameEvent, recipients: Recipients);
}

impl GameEventCollector for RefMut<'_, Vec<GameEventWithRecipients>> {
    fn add(&mut self, event: GameEvent, recipients: Recipients) {
        self.push((event, recipients));
    }
}
