use crate::executor::command_handlers::{AttackCommandHandler, CommandHandler, DieCommandHandler, JumpCommandHandler, MoveCommandHandler, RefillCommandHandler, SpawnCommandHandler, StartupCommandHandler, UpdateCameraFacingCommandHandler};
use crate::game_loop::ClientCommand;
use crate::simulation::physics_state::PhysicsState;
use common::core::command::{Command, MoveDirection};
use common::core::states::{GameLifeCycleState, GameState};

use crate::Recipients;
use common::configs::*;
use common::core::events::GameEvent;
use itertools::Itertools;
use log::{debug, error, info, warn};
use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use rapier3d::crossbeam::channel::internal::SelectHandle;
use common::core::states::GameLifeCycleState::{Running, Waiting};

mod command_handlers;

pub const DEFAULT_RESPAWN_LIMIT: f32 = -20.0; // 5ms
/// Executor is a struct that is used to execute a command issued by a client.
/// It maintains the state of the game and is responsible for updating it.
pub struct Executor {
    /// state of the game, owned by the executor, encapsulated in a Arc and protected by Mutex
    game_state: Arc<Mutex<GameState>>,
    physics_state: RefCell<PhysicsState>,
    game_events: RefCell<Vec<(GameEvent, Recipients)>>,
    config_instance: Arc<Config>,
    ready_players: RefCell<Vec<u32>>,
    spawn_command_pushed: RefCell<bool>,
}

impl Executor {
    /// Creates a new Executor with default game state.
    pub fn new(game_state: Arc<Mutex<GameState>>) -> Executor {
        Executor {
            game_state,
            physics_state: RefCell::new(PhysicsState::new()),
            game_events: RefCell::new(Vec::new()),
            config_instance: ConfigurationManager::get_configuration(),
            ready_players: RefCell::new(Vec::new()),
            spawn_command_pushed: RefCell::new(false),
        }
    }

    pub fn world_init(&self) {
        let mut game_state = self.game_state.lock().unwrap();
        let mut physics_state = self.physics_state.borrow_mut();
        let mut game_events = self.game_events.borrow_mut();

        let handler = StartupCommandHandler::new(
            self.config_instance.models.clone(),
            self.config_instance.scene.clone(),
        );

        if let Err(e) = handler.handle(&mut game_state, &mut physics_state, &mut game_events) {
            panic!("Failed init executor game/physics states: {:?}", e);
        }
    }

    pub fn game_init(&self, mut commands: Vec<ClientCommand>) -> Vec<ClientCommand> {
        if self.game_state().life_cycle_state == Running {
            // TODO: still have bugs when handling multiple game in a row without exiting
            if !*self.spawn_command_pushed.borrow() {
                for client_id in self.ready_players.borrow().iter() {
                    info!("Ready Players: {:?}", *self.ready_players.borrow());
                    commands.push(ClientCommand::new(*client_id, Command::Spawn));
                }
                *self.spawn_command_pushed.borrow_mut() = true;
            }
        }
        commands
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

        let player_config = self.config_instance.player.clone();

        if game_state.life_cycle_state == Waiting {
            match client_command.command {
                Command::Ready => {
                    if !self.ready_players.borrow().contains(&client_command.client_id) {
                        self.ready_players.borrow_mut().push(client_command.client_id);
                        if self.ready_players.borrow().len() == 4 {
                            game_state.life_cycle_state = Running;
                        }
                    } else {
                        warn!("player has already been ready!");
                    }
                },
                _ => {}
            }
        }
        else {
            let handler: Box<dyn CommandHandler> = match client_command.command {
                Command::Spawn => Box::new(SpawnCommandHandler::new(
                    client_command.client_id,
                    player_config,
                )),
                Command::Die => Box::new(DieCommandHandler::new(client_command.client_id)),
                Command::Move(dir) => Box::new(MoveCommandHandler::new(client_command.client_id, dir)),
                Command::UpdateCamera { forward } => Box::new(UpdateCameraFacingCommandHandler::new(
                    client_command.client_id,
                    forward,
                )),
                Command::Jump => Box::new(JumpCommandHandler::new(client_command.client_id)),
                Command::Attack => Box::new(AttackCommandHandler::new(client_command.client_id)),
                Command::Refill => Box::new(RefillCommandHandler::new(client_command.client_id)),
                _ => {
                    warn!("Unsupported command: {:?}", client_command.command);
                    return;
                }
            };

            if let Err(e) = handler.handle(&mut game_state, &mut physics_state, &mut game_events) {
                error!("Failed to execute command: {:?}", e);
            }
        }

        info!("GameState: {:?}", game_state);
    }

    pub(crate) fn step(&self, delta_time: f32) {
        self.physics_state.borrow_mut().set_delta_time(delta_time);
        self.physics_state.borrow_mut().step();

        self.sync_states(delta_time); // after physics step, need to sync game state
    }

    fn sync_states(&self, delta_time: f32) {
        let mut game_state = self.game_state.lock().unwrap();
        let physics_state = self.physics_state.borrow();

        // update player positions
        for (_id, player) in game_state.players.iter_mut() {
            let rigid_body = physics_state.get_entity_rigid_body(player.id).unwrap();
            player.transform.translation = rigid_body.position().translation.vector;
            player.transform.rotation = rigid_body.position().rotation.coords.into();
        }

        game_state.update_cooldowns(delta_time);

        match game_state.update_player_on_flag_times(delta_time) {
            Some(id) => {
                panic!("Winner is {}, game finished!", id)
            }
            None => {}
        }
        game_state.previous_tick_winner = match game_state.has_single_winner() {
            Some(id) => Some(id),
            None => None,
        }
    }

    pub(crate) fn collect_game_events(&self) -> Vec<(GameEvent, Recipients)> {
        self.game_events.replace(Vec::new())
    }

    pub(crate) fn update_dead_players(&self) -> Vec<u32> {
        self.game_state()
            .players
            .iter()
            .filter(|(_, player)| {
                !player.is_dead && player.transform.translation.y < DEFAULT_RESPAWN_LIMIT
            })
            .map(|(&id, _)| id)
            .collect::<Vec<_>>()
    }

    pub(crate) fn check_respawn_players(&self) -> Vec<u32> {
        self.game_state()
            .players
            .iter()
            .filter(|(_, player)| {
                player.is_dead && !player.on_cooldown.contains_key(&Command::Spawn)
            })
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
