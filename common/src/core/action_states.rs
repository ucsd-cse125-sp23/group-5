use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionState {
    Idle,
    Jumping,
    Walking,
    Attacking,
    SpecialAttacking,
    CastingPowerUp,
    Frozen,
    Wave
}

impl Default for ActionState {
    fn default() -> Self {
        Self::Idle
    }
}

impl ActionState {
    pub fn priority(&self) -> u8 {
        match self {
            ActionState::Idle => 0,
            ActionState::Jumping => 3,
            ActionState::Walking => 2,
            ActionState::Attacking => 3,
            ActionState::SpecialAttacking => 5,
            ActionState::CastingPowerUp => 6,
            ActionState::Frozen => 7,
            ActionState::Wave => 1
        }
    }

    pub fn animation_id(&self) -> &str {
        match self {
            ActionState::Idle => "idle",
            ActionState::Jumping => "jump",
            ActionState::Walking => "walk",
            ActionState::Attacking => "regular_attack",
            ActionState::SpecialAttacking => "special_attack",
            ActionState::CastingPowerUp => "powerup_attack",
            ActionState::Frozen => "frozen",
            ActionState::Wave => "wave"
        }
    }
}
