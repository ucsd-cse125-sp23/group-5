use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use rapier3d::prelude::Vector;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::configs::game_config::ConfigGame;
use crate::core::action_states::ActionState;
use crate::core::choices::FinalChoices;
use crate::core::command::Command;
use crate::core::components::{Physics, Transform};
use crate::core::events::ParticleSpec;
use crate::core::powerup_system::StatusEffect::Power;
use crate::core::powerup_system::{
    PowerUp, PowerUpLocations, PowerUpStatus, StatusEffect, POWER_UP_TO_EFFECT_MAP,
};
use crate::core::weather::Weather;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {
    pub weather: Option<Weather>,
    pub prev_weather: Option<Weather>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameState {
    pub world: WorldState,
    pub players: HashMap<u32, PlayerState>,
    pub players_customization: HashMap<u32, FinalChoices>,
    pub previous_tick_winner: Option<u32>,
    pub active_power_ups:
        HashMap<PowerUpLocations, (f32 /* time till next spawn powerup */, Option<PowerUp>)>,
    pub life_cycle_state: GameLifeCycleState,
    pub game_winner: Option<u32>,
    pub game_start_time: Duration,
    pub prev_winner: Option<(u32, FinalChoices)>,
}

impl GameState {
    pub fn new() -> Self {
        let mut active_power_ups: HashMap<PowerUpLocations, (f32, Option<PowerUp>)> =
            HashMap::new();
        active_power_ups.insert(PowerUpLocations::PowerUp1XYZ, (0.0, Some(rand::random())));
        active_power_ups.insert(PowerUpLocations::PowerUp2XYZ, (0.0, Some(rand::random())));
        active_power_ups.insert(PowerUpLocations::PowerUp3XYZ, (0.0, Some(rand::random())));
        active_power_ups.insert(PowerUpLocations::PowerUp4XYZ, (0.0, Some(rand::random())));

        Self {
            active_power_ups,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub id: u32,
    pub transform: Transform,
    pub physics: Physics,
    pub jump_count: u32,
    pub camera_forward: Vec3,
    pub is_dead: bool,
    pub on_cooldown: HashMap<Command, f32>,
    pub wind_charge: u32,
    pub on_flag_time: f32,
    pub spawn_point: Vector<f32>,
    pub power_up: Option<(PowerUp, PowerUpStatus)>,
    pub status_effects: HashMap<StatusEffect, f32 /* time till status effect expire */>,
    pub active_action_states: HashSet<(ActionState, Duration)>,
    pub cheat_keys_enabled: bool,
    pub last_step: u64,
    pub respawn_sec: u32, // b/c seconds are unreliable
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Copy)]
pub enum GameLifeCycleState {
    #[default]
    Waiting,
    Running(u64),
    Ended,
    _Ended,
}

impl GameLifeCycleState {
    pub fn unwrap_running(&self) -> u64 {
        match self {
            GameLifeCycleState::Running(d) => *d,
            _ => panic!("GameLifeCycleState is not Running"),
        }
    }
}

// Notes to be removed:
//      1. Update who should get the powerup
//          - need to update the powerup counter, and the player's state
//      2. Implement Powerup Effects ***
//          - perhaps use fake keybinds for now, for testing
//      3. Implement Powerup Casting
//      4. Refactoring

impl PlayerState {
    // add to attack command handlers, put NONE for consume 1
    // returns if the consumption is successful
    pub fn try_consume_wind_charge(&mut self, consume_amount: Option<u32>) -> bool {
        let consume_amount = consume_amount.unwrap_or(1);
        if self.wind_charge >= consume_amount {
            self.wind_charge -= consume_amount;
            true
        } else {
            false
        }
    }

    // add to refill command handlers, put NONE for refill all, won't exceed cap
    pub fn refill_wind_charge(&mut self, refill_amount: Option<u32>, max_wind_charge: u32) {
        let refill_amount = refill_amount.unwrap_or(max_wind_charge);
        let mut charges = self.wind_charge;
        charges += refill_amount;
        self.wind_charge = if charges > max_wind_charge {
            max_wind_charge
        } else {
            charges
        };
    }

    pub fn is_in_refill_area(&self, game_config: ConfigGame) -> Option<Vec3> {
        let radius = game_config.refill_radius;
        let mut closest_point = None;

        for i in &game_config.refill_points {
            if self.is_in_circular_area((i.x, i.z), radius, (None, None)) {
                closest_point = Some(*i);
            }
        }

        closest_point
    }

    pub fn is_need_refill(&self, game_config: ConfigGame) -> bool {
        if self.wind_charge == game_config.max_wind_charge {
            return false;
        }
        true
    }

    pub fn insert_cooldown(&mut self, command: Command, cooldown_in_sec: f32) {
        let cd_secs = Duration::from_secs_f32(cooldown_in_sec).as_secs_f32();
        //let cd_until = SystemTime::now().checked_add(cd_secs).unwrap();
        self.on_cooldown.insert(command, cd_secs);
    }

    pub fn command_on_cooldown(&self, command: Command) -> bool {
        self.on_cooldown.contains_key(&command)
    }

    pub fn is_in_circular_area(
        &self,
        horizontal_center: (f32, f32),
        radius: f32,
        vertical_bounds: (Option<f32>, Option<f32>),
    ) -> bool {
        let (p_x, p_y, p_z) = (
            self.transform.translation.x,
            self.transform.translation.y,
            self.transform.translation.z,
        );
        let (c_x, c_z) = horizontal_center;

        match vertical_bounds {
            (Some(y1), Some(y2)) => {
                if p_y > y2 || p_y < y1 {
                    return false;
                }
            }
            (Some(y1), None) => {
                if p_y < y1 {
                    return false;
                }
            }
            (None, Some(y2)) => {
                if p_y > y2 {
                    return false;
                }
            }
            (None, None) => {}
        }
        (p_x - c_x).powi(2) + (p_z - c_z).powi(2) < radius.powi(2)
    }

    pub fn reset_status_effects(&mut self) {
        self.status_effects.clear();
    }

    pub fn add_action_state(&mut self, action_state: ActionState, duration: Duration) {
        self.active_action_states.insert((action_state, duration));
    }

    pub fn sweep_action_states(&mut self, delta_time: Duration) {
        let updated_action_states: HashSet<(ActionState, Duration)> = self
            .active_action_states
            .iter()
            .filter_map(|(action_state, duration)| {
                let new_duration = duration.checked_sub(delta_time)?;
                Some((*action_state, new_duration))
            })
            .collect();
        self.active_action_states = updated_action_states;
    }

    pub fn holds_status_effect(&self, effect: StatusEffect) -> bool {
        self.status_effects.contains_key(&effect)
    }
    pub fn holds_status_effect_mut(&mut self, effect: StatusEffect) -> bool {
        self.status_effects.contains_key(&effect)
    }
}

impl GameState {
    pub fn player_mut(&mut self, id: u32) -> Option<&mut PlayerState> {
        self.players.get_mut(&id)
    }

    pub fn player(&self, id: u32) -> Option<&PlayerState> {
        self.players.get(&id)
    }

    pub fn update_cooldowns(&mut self, delta_time: f32) {
        for (_, player_state) in self.players.iter_mut() {
            player_state.on_cooldown = player_state
                .on_cooldown
                .clone()
                .into_iter()
                .map(|(key, cooldown)| (key, cooldown - delta_time))
                .filter(|(_key, cooldown)| *cooldown > 0.0)
                .collect();
        }
    }

    pub fn update_action_states(&mut self, delta_time: Duration) {
        for (_, player_state) in self.players.iter_mut() {
            player_state.sweep_action_states(delta_time);
        }
    }

    pub fn has_single_winner(&self, game_config: ConfigGame) -> Option<u32> {
        let valid_players: HashMap<u32, bool> = self
            .clone()
            .players
            .into_iter()
            .map(|(id, player_state)| {
                (
                    id,
                    player_state.is_in_circular_area(
                        game_config.flag_xz,
                        game_config.flag_radius,
                        game_config.flag_z_bound,
                    ),
                )
            })
            .filter(|(_, res)| *res)
            .collect();
        if valid_players.len() != 1 {
            None
        } else {
            Some(*valid_players.keys().last().unwrap())
        }
    }

    // returns winner if winner is decided
    pub fn update_player_on_flag_times(
        &mut self,
        delta_time: f32,
        game_config: ConfigGame,
    ) -> Option<u32> {
        // calculate elapsed time since game start in seconds
        let elapsed_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - self.game_start_time;
        let elapsed_seconds = elapsed_time.as_secs();
        // increase spawn_cooldown based on elapsed time
        let decay_rate_decrease = elapsed_seconds as f32 * game_config.decay_coef;
        let new_decay_rate = game_config.decay_rate - decay_rate_decrease;
        let mut still_decay = true;
        if new_decay_rate < 0.0 {
            still_decay = false;
        }

        // decay
        for (_, player_state) in self.players.iter_mut() {
            let mut provisional_on_flag_time = player_state.on_flag_time;
            if still_decay {
                provisional_on_flag_time -= delta_time * new_decay_rate;
            }

            player_state.on_flag_time = if provisional_on_flag_time > 0.0 {
                provisional_on_flag_time
            } else {
                0.0
            };
        }

        match self.previous_tick_winner {
            None => None,
            Some(id) => {
                if still_decay {
                    self.player_mut(id).unwrap().on_flag_time +=
                        delta_time * (1.0 + new_decay_rate);
                } else {
                    self.player_mut(id).unwrap().on_flag_time += delta_time;
                }
                if self.player_mut(id).unwrap().on_flag_time > game_config.winning_threshold {
                    Some(id)
                } else {
                    None
                }
            }
        }
    }

    // Process the status_effect map, remove all ones reached time, remove powerups accordingly
    pub fn update_player_status_effect(&mut self, delta_time: f32) {
        for (_, player_state) in self.players.iter_mut() {
            let (new_status_effects, to_process) = player_state
                .status_effects
                .clone()
                .into_iter()
                .map(|(key, time_remaining)| (key, time_remaining - delta_time))
                .partition(|(_, time_remaining)| *time_remaining > 0.0);
            player_state.status_effects = new_status_effects;
            for (effect, _) in to_process {
                if let Power(_) = effect {
                    if let Some((current_powerup, powerup_status)) = player_state.power_up.clone() {
                        if (*POWER_UP_TO_EFFECT_MAP
                            .get(&current_powerup.value())
                            .unwrap_or(&StatusEffect::None)
                            == effect)
                            && powerup_status == PowerUpStatus::Active
                        {
                            player_state.power_up = None;
                        }
                    }
                }
            }
        }
    }

    pub fn update_powerup_respawn(&mut self, delta_time: f32) {
        for (_, (vacancy_time, powerup)) in self.active_power_ups.iter_mut() {
            if powerup.clone().is_none() {
                // case where the powerup is empty, we need to refill the powerup for the map
                *vacancy_time -= delta_time;
                if *vacancy_time <= 0.0 {
                    // refill
                    *vacancy_time = 0.0;
                    *powerup = Some(rand::random());
                }
            }
        }
    }

    // check if any players should get a powerup and record powerup
    pub fn check_powerup_pickup(&mut self, game_config: ConfigGame) -> HashSet<u32> {
        let mut res = HashSet::new();
        let powerup_radius = game_config.powerup_config.power_up_radius;
        let powerup_respawn_cd = game_config.powerup_config.power_up_respawn_cooldown;
        for (loc_id, (vacancy_time, powerup)) in self.active_power_ups.iter_mut() {
            if !powerup.clone().is_none() {
                // check if a player should get the powerup now
                for (player_id, player_state) in self.players.iter_mut() {
                    let power_up_location = *game_config
                        .powerup_config
                        .power_up_locations
                        .get(&loc_id.value())
                        .unwrap();
                    if player_state.power_up.is_none()
                        && player_state.is_in_circular_area(
                            (power_up_location.0, power_up_location.2),
                            powerup_radius,
                            (
                                Some(power_up_location.1 - powerup_radius),
                                Some(power_up_location.1 + powerup_radius),
                            ),
                        )
                    {
                        // player should get it, powerup is gone
                        player_state.power_up =
                            Some((powerup.clone().unwrap(), PowerUpStatus::Held));
                        res.insert(*player_id);
                        *vacancy_time = powerup_respawn_cd;
                        *powerup = None;
                    }
                }
            }
        }
        return res;
    }

    pub fn find_closest_player(&self, id_to_find: u32) -> Option<u32> {
        let mut min: Option<(u32, f32)> = None;

        for (id, player_state) in self.players.iter() {
            if *id == id_to_find {
                continue;
            }
            if !player_state.is_dead {
                if min.is_none() {
                    min = Some((
                        *id,
                        calculate_distance(
                            player_state.transform.translation,
                            self.players.get(id).unwrap().transform.translation,
                        ),
                    ));
                } else {
                    let temp = calculate_distance(
                        player_state.transform.translation,
                        self.players.get(id).unwrap().transform.translation,
                    );
                    if temp < min.unwrap().1 {
                        min = Some((*id, temp));
                    }
                }
            }
        }
        min.map(|(id, _)| id)
    }

    pub fn get_affected_players(&self, effect: StatusEffect) -> HashSet<u32> {
        let mut res = HashSet::new();
        for (id, player_state) in self.players.iter() {
            if player_state.status_effects.contains_key(&effect) {
                res.insert(*id);
            }
        }
        res
    }

    pub fn get_existing_powerups(&self) -> HashSet<u32> {
        let mut res = HashSet::new();
        for (id, (_, powerup)) in self.active_power_ups.iter() {
            if powerup.is_some() {
                res.insert(id.value());
            }
        }
        res
    }
}

pub fn calculate_distance(lhs: Vec3, rhs: Vec3) -> f32 {
    ((lhs.x - rhs.x).powi(2) + (lhs.y - rhs.y).powi(2) + (lhs.z - rhs.z).powi(2)).sqrt()
}

#[derive(Debug, Clone, Default)]
pub struct ParticleQueue {
    pub particles: Vec<ParticleSpec>,
}

impl ParticleQueue {
    pub fn add_particle(&mut self, particle: ParticleSpec) {
        self.particles.push(particle);
    }
}

mod tests {
    #[test]
    fn test_default() {
        use super::*;
        let state = GameState {
            world: WorldState::default(),
            players: HashMap::default(),
            players_customization: Default::default(),
            previous_tick_winner: None,
            active_power_ups: HashMap::default(),
            life_cycle_state: Default::default(),
            game_winner: None,
            game_start_time: Default::default(),
            prev_winner: None,
        };
        assert_eq!(state.players.len(), 0);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        use super::*;
        let state = GameState {
            world: WorldState::default(),
            players: HashMap::default(),
            players_customization: Default::default(),
            previous_tick_winner: None,
            active_power_ups: HashMap::default(),
            life_cycle_state: Default::default(),
            game_winner: None,
            game_start_time: Default::default(),
            prev_winner: None,
        };
        let serialized = bincode::serialize(&state).unwrap();
        let deserialized: GameState = bincode::deserialize(&serialized[..]).unwrap();
        assert_eq!(state.players.len(), deserialized.players.len());
    }
}
