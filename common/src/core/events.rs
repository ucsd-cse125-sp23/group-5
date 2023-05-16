use derive_more::Constructor;
use serde::{Deserialize, Serialize};

extern crate nalgebra_glm as glm;

/// Event that the server send to the client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameEvent {
    SoundEvent(SoundSpec),
    ParticleEvent(ParticleSpec),
}

/// Sound specification
#[derive(Constructor, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoundSpec {
    pub position: glm::Vec3,
    pub sound_id: String,
    pub at_client: (u32, bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticleType {
    ATTACK,
}
/// Particle specification
#[derive(Constructor, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleSpec {
    pub p_type: ParticleType,
    pub position: glm::Vec3,
    pub direction: glm::Vec3,
    pub up: glm::Vec3,
    pub color: glm::Vec4,
    pub particle_id: String,
}
