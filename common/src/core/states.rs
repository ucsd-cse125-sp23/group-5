use crate::core::command::Command;
use crate::core::components::{Physics, Transform};
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
    pub ammo_count: u32,
    pub camera_forward: Vec3,
    pub connected: bool,
    pub is_dead: bool,
    pub on_cooldown: HashMap<Command, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameState {
    pub world: WorldState,
    pub players: HashMap<u32, PlayerState>,
}

impl GameState {
    pub fn player_mut(&mut self, id: u32) -> Option<&mut PlayerState> {
        self.players.get_mut(&id)
    }

    pub fn player(&self, id: u32) -> Option<&PlayerState> {
        self.players.get(&id)
    }

    pub fn insert_cooldown(&mut self, id: u32, command: Command, cooldown_in_sec: u64) {
        let cd_secs = Duration::from_secs(cooldown_in_sec).as_secs_f32();
        //let cd_until = SystemTime::now().checked_add(cd_secs).unwrap();
        self.player_mut(id)
            .unwrap()
            .on_cooldown
            .insert(command, cd_secs);
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

    pub fn command_on_cooldown(&self, client_id: u32, command: Command) -> bool {
        self.player(client_id)
            .unwrap()
            .on_cooldown
            .contains_key(&command)
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
