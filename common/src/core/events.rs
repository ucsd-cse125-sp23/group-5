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
}

/// Particle specification
#[derive(Constructor, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleSpec {
    pub position: glm::Vec3,
    pub particle_id: String,
    // TODO: may add more fields like duration, shape, etc. But this can also be done on the client side only if the specs of particles are finite
}