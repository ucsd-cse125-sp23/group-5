use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTexture {
    pub textures: Vec<ConfigTextureItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTextureItem {
    pub name: String,
    pub path: String,
}