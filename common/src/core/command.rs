extern crate nalgebra_glm as glm;

use std::hash::{Hash, Hasher};
use std::mem;

use glm::Quat;
use serde::{Deserialize, Serialize};

use crate::core::choices::FinalChoices;
use crate::core::powerup_system::PowerUp;

/// Direction of the movement
pub type MoveDirection = glm::Vec3;

/// Commands for ui interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerSync {
    Ready,
    Choices(FinalChoices),
    End,
}

// Commands for activating/deactivating cheatkeys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheatCodeControl {
    Activate,
    Deactivate,
}

// Commands for manipulating weather with cheatkeys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheatKeyWeather {
    Rain,
    Wind,
    Reset,
}

/// Commands that can be issued by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    UI(ServerSync),
    Spawn,
    Die,
    Move(MoveDirection),
    Turn(Quat),
    Jump,
    UpdateCamera { forward: glm::Vec3 },
    Attack,
    AreaAttack,
    Refill,
    CastPowerUp,
    GivePowerUp,
    StatusEffects,
    Dash,
    Flash,
    UpdateWeather,
    WeatherEffects,
    CheatCode(PowerUp),
    CheatCodeControl(CheatCodeControl),
    WeatherCheatKey(CheatKeyWeather),
    Wave
}

impl Command {
    pub fn unwrap_move(&self) -> MoveDirection {
        match self {
            Command::Move(dir) => *dir,
            _ => panic!("Command is not a move command"),
        }
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Command::Move(x) => match other {
                Command::Move(y) => x.eq(y),
                _ => false,
            },
            _ => mem::discriminant(self).eq(&mem::discriminant(other)),
        }
    }
}

impl Eq for Command {}

impl Hash for Command {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Command::Move(x) => {
                let _x = ((x.x * 1000000_f32).round() / 1.0) as i64;
                let _y = ((x.y * 1000000_f32).round() / 1.0) as i64;
                let _z = ((x.z * 1000000_f32).round() / 1.0) as i64;
                state.write_i64(_x);
                state.write_i64(_y);
                state.write_i64(_z);
            }
            _ => {}
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
