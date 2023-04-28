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
pub struct ConfigModel {
    pub index: usize,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSceneGraph {
    pub nodes: Vec<ConfigNode>,
    pub models: Vec<ConfigModel>,
}

impl ConfigSceneGraph {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, serde_json::Error> {
        let mut file = File::open(path).expect("Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the file");
        serde_json::from_str(&contents)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), serde_json::Error> {
        let serialized = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path).expect("Unable to create the file");
        file.write_all(serialized.as_bytes()).expect("Unable to write to the file");
        Ok(())
    }
}
