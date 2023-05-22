use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigParticle {
    pub time_divider: f32,
    pub attack_particle_config: AttackParticleConfig,
    pub area_attack_particle_config: AreaAttackParticleConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttackParticleConfig {
    pub linear_variance: f32,
    pub angular_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub size_growth: f32,
    pub gen_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaAttackParticleConfig {
    pub linear_variance: f32,
    pub angular_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub size_growth: f32,
    pub gen_speed: f32,
}