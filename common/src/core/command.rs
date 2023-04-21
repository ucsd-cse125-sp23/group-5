extern crate nalgebra_glm as glm;
use glm::Quat;
use serde::{Deserialize, Serialize};

/// Direction of the movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveDirection {
    Forward,
    Backward,
    Left,
    Right,
}

/// Game actions that can be performed by the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    Attack,
    Jump,
}

/// Commands that can be issued by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Spawn,
    Move(MoveDirection),
    Turn(Quat),
    UpdateCamera{position: glm::Vec3, spherical_coords: glm::Vec3},
    Action(GameAction),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_and_deserialize_json() {
        let command = Command::Move(MoveDirection::Forward);
        let serialized = serde_json::to_string(&command).unwrap();
        println!("{}", serialized);
        let deserialized: Command = serde_json::from_str(&serialized).unwrap();
        // assert_eq!(command, deserialized);
    }
}
