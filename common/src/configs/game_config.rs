use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigGame {
    pub spawn_points: Vec<rapier3d::prelude::Vector<f32>>,
    pub refill_points: Vec<rapier3d::prelude::Vector<f32>>,
    pub spawn_cooldown: f32,
    pub respawn_coef: f32,
    pub score_lower_x: f32,
    pub score_upper_x: f32,
    pub max_wind_charge: u32,
    pub one_charge: u32,
    pub flag_xz: (f32, f32),
    pub flag_radius: f32,
    pub flag_z_bound: (Option<f32>, Option<f32>),
    pub winning_threshold: f32,
    pub decay_rate: f32,
    pub decay_coef: f32,
    pub refill_radius: f32,
    pub refill_rate_limit: f32,
    pub camera_config: ConfigCamera,
    pub powerup_config: ConfigPowerUp,
    pub weather_config: ConfigWeather,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigCamera {
    pub x_sensitivity: f32,
    pub y_sensitivity: f32,
    pub scroll_sensitivity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigPowerUp {
    pub power_up_locations: std::collections::HashMap<u32, (f32, f32, f32)>,
    pub power_up_radius: f32,
    pub power_up_respawn_cooldown: f32,
    pub power_up_buff_duration: f32,
    pub power_up_debuff_duration: f32,
    pub power_up_cooldown: f32,
    pub blizzard_max_attack_dist: f32,
    pub blizzard_max_attack_angle: f32,
    pub wind_enhancement_scalar: f32,
    pub dash_impulse: f32,
    pub flash_distance_scalar: f32,
    pub invincible_effective_distance: f32,
    pub invincible_effective_impulse: f32,
    pub flash_cooldown: f32,
    pub spawn_invincible_duration: f32,
    pub dash_blocking_duration: f32,
    pub dash_cooldown: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigWeather {
    pub default_weather_ambient_multiplier: f32,
    pub rainy_weather_ambient_multiplier: f32,
    pub environment_lighting_rate_change_coefficient: f32,
    pub rain_ratio: f32,
    pub wind_ratio: f32,
    pub no_weather_ratio: f32,
}
