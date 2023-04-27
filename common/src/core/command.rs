extern crate nalgebra_glm as glm;

use glm::Quat;
use serde::{Deserialize, Serialize};

/// Direction of the movement
pub type MoveDirection = glm::Vec3;

/// Game actions that can be performed by the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    Attack,
    Jump,
}

/// Spawn type that can be issued by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnType {
    NewSpawn,
    Respawn,
    Dead,
}

/// Commands that can be issued by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Spawn,
    Move(MoveDirection),
    Turn(Quat),
    Jump,
    UpdateCamera { forward: glm::Vec3 },
    Action(GameAction),
}

impl Command {
    pub fn unwrap_move(&self) -> MoveDirection {
        match self {
            Command::Move(dir) => *dir,
            _ => panic!("Command is not a move command"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_and_deserialize_json() {
        let command = Command::Move(MoveDirection::new(1., 0., 0.));
        let serialized = serde_json::to_string(&command).unwrap();
        let _deserialized: Command = serde_json::from_str(&serialized).unwrap();
        // assert_eq!(command, deserialized);
    }
}
