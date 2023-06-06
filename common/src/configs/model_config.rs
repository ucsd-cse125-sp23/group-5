use serde::{Deserialize, Serialize};

pub type ModelIndex = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigModel {
    pub name: ModelIndex,
    pub path: String,
    pub animated: Option<bool>,
    pub phantom: Option<bool>,
}

impl ConfigModel {
    pub fn animated(&self) -> bool {
        self.animated.unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigModels {
    pub models: Vec<ConfigModel>,
}

impl ConfigModels {
    pub fn model(&self, name: String) -> Option<&ConfigModel> {
        self.models.iter().find(|m| m.name == name)
    }
}
