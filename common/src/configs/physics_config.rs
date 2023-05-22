use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigPhysics {
    pub max_attack_dist: f32,
    pub max_attack_angle: f32,
    pub attack_impulse: f32,
    pub attack_coeff: f32,
    pub attack_cost: u32,
    pub attack_cooldown: f32,
    pub max_area_attack_dist: f32,
    pub area_attack_impulse: f32,
    pub area_attack_coeff: f32,
    pub area_attack_cost: u32,
    pub area_attack_cooldown: f32,
}
