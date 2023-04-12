use crate::core::components::{Physics, Transform};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct PlayerState {
    id: usize,
    transform: Transform,
    physics: Physics,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct WorldState {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct GameState {
    world: WorldState,
    players: Vec<PlayerState>,
}

mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let state = GameState {
            world: WorldState::default(),
            players: vec![PlayerState::default()],
        };
        assert_eq!(state.players.len(), 1);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let state = GameState {
            world: WorldState::default(),
            players: vec![PlayerState::default()],
        };
        let serialized = bson::to_bson(&state).unwrap();
        let deserialized: GameState = bson::from_bson(serialized).unwrap();
        assert_eq!(state.players.len(), deserialized.players.len());
    }
}
