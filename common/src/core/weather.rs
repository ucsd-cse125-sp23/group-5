use serde::{Deserialize, Serialize};
use nalgebra_glm::Vec3 as Vector;
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq,PartialOrd)]

pub enum Weather {
    Rainy,
    Windy(Vector),
}
