use phf::phf_map;
use serde::{Deserialize, Serialize};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PowerUp {
    Lightning,
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
            PowerUp::Lightning => 1,
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
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PowerUp {
        match rng.gen_range(1..=5) {
            1 => PowerUp::Lightning,
            2 => PowerUp::WindEnhancement,
            3 => PowerUp::Dash,
            4 => PowerUp::Flash,
            5 => PowerUp::Invisible,
            _ => PowerUp::TripleJump,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum StatusEffect {
    EnhancedWind,
    EnabledDash,
    EnabledFlash,
    Invisible,
    TripleJump,
    Invincible,
    Stun,
    // for later weather effect
    Blinded,
    Slippery,
}

pub static POWER_UP_TO_EFFECT_MAP: phf::Map<u32, StatusEffect> = phf_map! {
    2u32 => StatusEffect::EnhancedWind,
    3u32 => StatusEffect::EnabledDash,
    4u32 => StatusEffect::EnabledFlash,
    5u32 => StatusEffect::Invisible,
    6u32 => StatusEffect::TripleJump,
    7u32 => StatusEffect::Invincible,
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
