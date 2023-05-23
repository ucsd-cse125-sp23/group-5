use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigPhysics {
    pub attack_config: ConfigAttack,
    pub movement_config: ConfigAction,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigAttack {
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConfigAction {
    pub walking_cooldown: f32,
    pub step_size: f32,
    pub gain: f32,
    pub damping: f32,
    pub max_jump_count: u32,
    pub jump_impulse: f32,
}
