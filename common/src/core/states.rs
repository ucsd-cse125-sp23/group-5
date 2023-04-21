use nalgebra_glm::Vec3;
use crate::core::components::{Physics, Transform};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerState {
    pub id: u32,
    pub transform: Transform,
    pub physics: Physics,
    pub camera_forward: Vec3,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameState {
    pub world: WorldState,
    pub players: Vec<PlayerState>,
}

mod tests {
    #[test]
    fn test_default() {
        use super::*;
        let state = GameState {
            world: WorldState::default(),
            players: vec![PlayerState::default()],
        };
        assert_eq!(state.players.len(), 1);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        use super::*;
        let state = GameState {
            world: WorldState::default(),
            players: vec![PlayerState::default()],
        };
        let serialized = bson::to_bson(&state).unwrap();
        let deserialized: GameState = bson::from_bson(serialized).unwrap();
        assert_eq!(state.players.len(), deserialized.players.len());
    }
}
