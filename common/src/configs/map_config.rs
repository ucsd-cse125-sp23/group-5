use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use nalgebra_glm::{Quat, TVec3};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTransform {
    pub position: TVec3<f32>,
    pub rotation: Quat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub transform: ConfigTransform,
    pub children: Option<Vec<ConfigNode>>,
    pub model: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSceneGraph {
    pub nodes: Vec<ConfigNode>,
}
