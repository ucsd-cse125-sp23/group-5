use glam::Quat;
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
}

/// Commands that can be issued by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Spawn,
    Move(MoveDirection),
    Turn(Quat),
    Action(GameAction),
}
