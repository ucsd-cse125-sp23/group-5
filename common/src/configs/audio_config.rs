use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioElement {
    pub name: String,
    pub path: String,
    pub seconds: u64,
    pub nanoseconds: u32,
    pub fall_off_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigAudioAssets {
    pub sounds: Vec<AudioElement>,
}
