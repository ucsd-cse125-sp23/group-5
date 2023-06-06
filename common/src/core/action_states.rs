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
            ActionState::Jumping => 2,
            ActionState::Walking => 1,
            ActionState::Attacking => 3,
            ActionState::SpecialAttacking => 4,
            ActionState::CastingPowerUp => 5,
            ActionState::Frozen => 6,
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
        }
    }
}
