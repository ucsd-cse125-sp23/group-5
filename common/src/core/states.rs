use crate::communication::commons::MAX_WIND_CHARGE;
use crate::core::command::Command;
use crate::core::components::{Physics, Transform};
use crate::core::events::ParticleSpec;
use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub id: u32,
    pub transform: Transform,
    pub physics: Physics,
    pub jump_count: u32,
    pub camera_forward: Vec3,
    pub connected: bool,
    pub is_dead: bool,
    pub on_cooldown: HashMap<Command, f32>,
    pub wind_charge: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameState {
    pub world: WorldState,
    pub players: HashMap<u32, PlayerState>,
}

impl PlayerState {
    // add to attack command handlers, put NONE for consume 1
    // returns if the consumption is successful
    pub fn try_consume_wind_charge(&mut self, consume_amount: Option<u32>) -> bool {
        let consume_amount = consume_amount.unwrap_or(1);
        return if self.wind_charge >= consume_amount {
            self.wind_charge -= consume_amount;
            true
        } else {
            false
        };
    }

    // add to refill command handlers, put NONE for refill all, won't exceed cap
    pub fn refill_wind_charge(&mut self, refill_amount: Option<u32>) {
        let refill_amount = refill_amount.unwrap_or(MAX_WIND_CHARGE);
        let mut charges = self.wind_charge;
        charges += refill_amount;
        self.wind_charge = if charges > MAX_WIND_CHARGE {
            MAX_WIND_CHARGE
        } else {
            charges
        };
    }

    pub fn insert_cooldown(&mut self, command: Command, cooldown_in_sec: u64) {
        let cd_secs = Duration::from_secs(cooldown_in_sec).as_secs_f32();
        //let cd_until = SystemTime::now().checked_add(cd_secs).unwrap();
        self.on_cooldown.insert(command, cd_secs);
    }

    pub fn command_on_cooldown(&self, command: Command) -> bool {
        self.on_cooldown.contains_key(&command)
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
}

#[derive(Debug, Clone, Default)]
pub struct ParticleQueue{
    pub particles: Vec<ParticleSpec>,
}

impl ParticleQueue{
    pub fn add_particle(&mut self, particle: ParticleSpec){
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
        };
        assert_eq!(state.players.len(), 0);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        use super::*;
        let state = GameState {
            world: WorldState::default(),
            players: HashMap::default(),
        };
        let serialized = bson::to_bson(&state).unwrap();
        let deserialized: GameState = bson::from_bson(serialized).unwrap();
        assert_eq!(state.players.len(), deserialized.players.len());
    }
}
