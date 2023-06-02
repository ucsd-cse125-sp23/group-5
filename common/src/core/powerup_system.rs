use phf::phf_map;
use rand::distributions::Uniform;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PowerUp {
    Blizzard,
    WindEnhancement,
    Dash,
    Flash,
    Invisible,
    TripleJump,
    Invincible, // maybe
}

impl PowerUp {
    pub fn value(&self) -> u32 {
        match *self {
            PowerUp::Blizzard => 1,
            PowerUp::WindEnhancement => 2,
            PowerUp::Dash => 3,
            PowerUp::Flash => 4,
            PowerUp::Invisible => 5,
            PowerUp::TripleJump => 6,
            PowerUp::Invincible => 7, //to be implemented
        }
    }
}

impl Distribution<PowerUp> for Standard {
    // adjusted ratios
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PowerUp {
        let between = Uniform::from(1..13);
        match between.sample(rng) {
            1 => PowerUp::Blizzard,
            2 | 3 => PowerUp::WindEnhancement,
            4 | 5 => PowerUp::Dash,
            6 | 7 => PowerUp::Flash,
            8 | 9 => PowerUp::Invisible,
            10 | 11 => PowerUp::TripleJump,
            _ => PowerUp::Invincible,
            // _ => PowerUp::Dash,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum StatusEffect {
    Power(PowerUpEffects),
    Other(OtherEffects),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PowerUpEffects {
    EnhancedWind,
    EnabledDash,
    EnabledFlash,
    Invisible,
    TripleJump,
    Invincible,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum OtherEffects {
    Stun,
    MovementDisabled,
    // for later weather effect
    Blinded,
    Slippery,
}

pub static POWER_UP_TO_EFFECT_MAP: phf::Map<u32, StatusEffect> = phf_map! {
    2u32 => StatusEffect::Power(PowerUpEffects::EnhancedWind),
    3u32 => StatusEffect::Power(PowerUpEffects::EnabledDash),
    4u32 => StatusEffect::Power(PowerUpEffects::EnabledFlash),
    5u32 => StatusEffect::Power(PowerUpEffects::Invisible),
    6u32 => StatusEffect::Power(PowerUpEffects::TripleJump),
    7u32 => StatusEffect::Power(PowerUpEffects::Invincible),
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PowerUpLocations {
    PowerUp1XYZ,
    PowerUp2XYZ,
    PowerUp3XYZ,
    PowerUp4XYZ,
}

impl PowerUpLocations {
    pub fn value(&self) -> u32 {
        match *self {
            PowerUpLocations::PowerUp1XYZ => 1,
            PowerUpLocations::PowerUp2XYZ => 2,
            PowerUpLocations::PowerUp3XYZ => 3,
            PowerUpLocations::PowerUp4XYZ => 4,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PowerUpStatus {
    Active,
    Held,
}
