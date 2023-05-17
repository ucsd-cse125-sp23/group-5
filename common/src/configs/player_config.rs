use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigPlayer {
    pub spawn_points: Vec<rapier3d::prelude::Vector<f32>>,
}
