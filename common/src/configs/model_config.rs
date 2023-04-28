use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigModel {
    pub index: usize,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigModels {
    pub models: Vec<ConfigModel>,
}