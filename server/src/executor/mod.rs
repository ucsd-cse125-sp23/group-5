use std::sync::{Arc, Mutex};

use common::configs::*;
use common::core::command::{Command, MoveDirection, ServerSync};
use common::core::events::GameEvent;
use common::core::states::GameState;

use crate::game_loop::ClientCommand;
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;

use common::core::states::GameLifeCycleState::{Running, Waiting, Ended};
use itertools::Itertools;
use log::{debug, error, info, warn};
use std::cell::RefCell;
use std::time::Duration;

use command_handlers::prelude::*;

pub mod command_handlers;

pub const DEFAULT_RESPAWN_LIMIT: f32 = -20.0;

// 5ms
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

    pub fn game_init(&self, commands: &mut Vec<ClientCommand>) {
        if matches!(self.game_state().life_cycle_state, Running(_)) {
            // TODO: still have bugs when handling multiple game in a row without exiting
            if !*self.spawn_command_pushed.borrow() {
                for client_id in self.ready_players.borrow().iter() {
                    info!("Ready Players: {:?}", *self.ready_players.borrow());
                    commands.push(ClientCommand::new(*client_id, Command::Spawn));
                }
                *self.spawn_command_pushed.borrow_mut() = true;
            }
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

    pub(crate) fn game_state_tick(&self) {
        // increment tick
        if let Running(ref mut tick) = &mut self.game_state.lock().unwrap().life_cycle_state {
            *tick += 1;
        }
    }

    /// Executes a command issued by a client.
    pub(crate) fn execute(&self, client_command: ClientCommand) {
        debug!("Executing command: {:?}", client_command);

        let mut game_state = self.game_state.lock().unwrap();
        let mut physics_state = self.physics_state.borrow_mut();
        let mut game_events = self.game_events.borrow_mut();

        let player_config = self.config_instance.player.clone();

        #[cfg(not(feature = "debug-ready-sync"))]
            let player_upper_bound = 4;

        #[cfg(feature = "debug-ready-sync")]
            let player_upper_bound = 1;

        if game_state.life_cycle_state == Waiting {
            if let Command::UI(ServerSync::Ready) = client_command.command {
                if !self
                    .ready_players
                    .borrow()
                    .contains(&client_command.client_id)
                {
                    self.ready_players
                        .borrow_mut()
                        .push(client_command.client_id);
                    // change the 1 to 4 for working correctly
                    // here I just change it to 1 for testing purpose
                    if self.ready_players.borrow().len() == player_upper_bound {
                        game_state.life_cycle_state = Running(0);
                    }
                } else {
                    warn!("player has already been ready!");
                }
            }
        } else {
            let handler: Box<dyn CommandHandler> = match client_command.command {
                Command::Spawn => Box::new(SpawnCommandHandler::new(
                    client_command.client_id,
                    player_config,
                )),
                Command::Die => Box::new(DieCommandHandler::new(client_command.client_id)),
                Command::Move(dir) => {
                    Box::new(MoveCommandHandler::new(client_command.client_id, dir))
                }
                Command::UpdateCamera { forward } => Box::new(
                    UpdateCameraFacingCommandHandler::new(client_command.client_id, forward),
                ),
                Command::Jump => Box::new(JumpCommandHandler::new(client_command.client_id)),
                Command::Attack => Box::new(AttackCommandHandler::new(client_command.client_id)),
                Command::AreaAttack => {
                    Box::new(AreaAttackCommandHandler::new(client_command.client_id))
                }
                Command::Refill => Box::new(RefillCommandHandler::new(client_command.client_id)),
                Command::CastPowerUp => {
                    Box::new(CastPowerUpCommandHandler::new(client_command.client_id))
                }
                Command::Dash => Box::new(DashCommandHandler::new(client_command.client_id)),
                Command::Flash => Box::new(FlashCommandHandler::new(client_command.client_id)),


                // weather systems
                Command::UpdateWeather => Box::new(UpdateWeatherCommandHandler::new()),
                Command::WeatherEffects => Box::new(WeatherEffectCommandHandler::new()),
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

        // update the cooldowns
        game_state.update_cooldowns(delta_time);
        game_state.update_action_states(Duration::from_secs_f32(delta_time));

        // update the powerup counters for players
        game_state.update_player_status_effect(delta_time);

        // update the powerup for each server location
        game_state.update_powerup_locations(delta_time);

        if let Some(id) = game_state.update_player_on_flag_times(delta_time) {
            println!("Winner is {}, game finished!", id);
            game_state.game_winner = Some(id);
            game_state.life_cycle_state = Ended;
        }
        game_state.previous_tick_winner = game_state.has_single_winner();
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

    pub fn reset_game(&self) {
        // If game ended, reset game back to waiting state
        let mut game_state = self.game_state.lock().unwrap();
        if game_state.life_cycle_state == Ended {
            let mut physics_state = self.physics_state.borrow_mut();
            let mut game_events = self.game_events.borrow_mut();
            let mut ready_players = self.ready_players.borrow_mut();
            let mut spawn_command_pushed = self.spawn_command_pushed.borrow_mut();
            
            // Remove all players from physics state 
            for player_id in game_state.players.keys() {
                physics_state.remove_entity(*player_id);
            }
            
            // Reset other instance variables 
            *game_state = GameState::new();
            game_events.clear();
            ready_players.clear(); 
            *spawn_command_pushed = false; 
        }
    }

    /// get a clone of the game state
    pub fn game_state(&self) -> GameState {
        self.game_state.lock().unwrap().clone()
    }
}
