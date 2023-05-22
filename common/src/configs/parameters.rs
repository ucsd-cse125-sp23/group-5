use phf::phf_map;

// powerup
pub const POWER_UP_1_XYZ: (f32, f32, f32) = (5.0, -5.0, -5.0);
pub const POWER_UP_2_XYZ: (f32, f32, f32) = (-5.0, -5.0, 5.0);
pub const POWER_UP_3_XYZ: (f32, f32, f32) = (-5.0, -5.0, -5.0);
pub const POWER_UP_4_XYZ: (f32, f32, f32) = (5.0, -5.0, 5.0);

pub static POWER_UP_LOCATIONS: phf::Map<u32, (f32, f32, f32)> = phf_map! {
    1u32 => POWER_UP_1_XYZ,
    2u32 => POWER_UP_2_XYZ,
    3u32 => POWER_UP_3_XYZ,
    4u32 => POWER_UP_4_XYZ,
};

pub const POWER_UP_RADIUS: f32 = 1.0;
pub const POWER_UP_RESPAWN_COOLDOWN: f32 = 15.0;
pub const POWER_UP_BUFF_DURATION: f32 = 10.0;
pub const POWER_UP_DEBUFF_DURATION: f32 = 3.0;
pub const POWER_UP_COOLDOWN: f32 = 5.0;

pub const WIND_ENHANCEMENT_SCALAR: f32 = 1.5;
pub const DASH_IMPULSE: f32 = 100.0;

pub const FLASH_DISTANCE_SCALAR: f32 = 5.0;

pub const INVINCIBLE_EFFECTIVE_DISTANCE: f32 = 2.5;
pub const INVINCIBLE_EFFECTIVE_IMPULSE: f32 = 20.0;

pub const SPECIAL_MOVEMENT_COOLDOWN: f32 = 0.5;

// TODO:
/* there are some more constants in command_handler,
such as 1.4, 0.9, PI/3, 0.0 etc. but all seem quite refined */
