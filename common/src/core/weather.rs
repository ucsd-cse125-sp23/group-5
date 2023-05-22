use nalgebra_glm::Vec3 as Vector;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd)]

pub enum Weather {
    Rainy,
    Windy(Vector),
}
