use nalgebra_glm::{Quat, TVec3};
use serde::{Deserialize, Serialize};

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
    pub decompose: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSceneGraph {
    pub nodes: Vec<ConfigNode>,
}
