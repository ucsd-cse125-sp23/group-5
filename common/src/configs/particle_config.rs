use serde::{Deserialize, Serialize};

pub const MODEL_1: &str = "attack shape 1";
pub const MODEL_2: &str = "attack shape 2";
pub const MODEL_3: &str = "attack shape 3";
pub const MODEL_4: &str = "attack shape 4";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigParticle {
    pub time_divider: f32,
    pub attack_particle_config: AttackParticleConfig,
    pub area_attack_particle_config: AreaAttackParticleConfig,
    pub blizzard_particle_config: BlizzardParticleConfig,
    pub powerup_particle_config: PowerUpParticleConfig,
    pub powerup_aura_particle_config: PowerUpAuraParticleConfig,
    pub winning_area_ribbon_particle_config: WinningAreaRibbonParticleConfig,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlizzardParticleConfig {
    pub time: f32,
    pub linear_variance: f32,
    pub angular_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub size_growth: f32,
    pub gen_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PowerUpParticleConfig {
    pub time: f32,
    pub max_dist: f32,
    pub linear_variance: f32,
    pub angular_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub size_growth: f32,
    pub gen_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PowerUpAuraParticleConfig {
    pub time: f32,
    pub r: f32,
    pub half_height: f32,
    pub aura_colors: std::collections::HashMap<String, (f32, f32, f32, f32)>,
    pub linear_speed: f32,
    pub linear_variance: f32,
    pub angular_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub size_growth: f32,
    pub gen_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WinningAreaRibbonParticleConfig {
    pub bounds_min: (f32, f32, f32),
    pub bounds_max: (f32, f32, f32),
    pub v_dir: (f32, f32, f32),
    pub visible_time: f32,
    pub time: f32,
    pub gen_time: f32,
    pub linear_speed: f32,
    pub linear_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub subdivisions: u32,
    pub gen_speed: f32,
}
