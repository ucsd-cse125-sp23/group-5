use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigGame {
    pub spawn_points: Vec<rapier3d::prelude::Vector<f32>>,
    pub spawn_cooldown: f32,
    pub score_lower_x: f32,
    pub score_upper_x: f32,
    pub max_wind_charge: u32,
    pub one_charge: u32,
    pub flag_xz: (f32, f32),
    pub flag_radius: f32,
    pub flag_z_bound: (Option<f32>, Option<f32>),
    pub winning_threshold: f32,
    pub decay_rate: f32,
    pub refill_radius: f32,
    pub refill_rate_limit: f32,
}
