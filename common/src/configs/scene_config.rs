use nalgebra_glm::{Quat, TVec3};
use serde::{Deserialize, Serialize};
use rapier3d::prelude;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTransform {
    pub position: TVec3<f32>,
    pub rotation: Quat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigNode {
    pub id: String,
    pub transform: ConfigTransform,
    pub children: Option<Vec<ConfigNode>>,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSceneGraph {
    pub spawn_points: Vec<rapier3d::prelude::Vector<f32>>,
    pub nodes: Vec<ConfigNode>,

}