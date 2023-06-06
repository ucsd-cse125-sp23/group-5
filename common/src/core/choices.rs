use crate::configs::model_config::ModelIndex;
use crate::core::mesh_color::MeshColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const LOBBY_STARTING_MODEL: &str = "korok_1";
pub const OBJECT_PLAYER_MODEL: &str = "object:player_model";
pub const LEAF_MESH: &str = "leaf";
pub const BODY_MESH: &str = "korok";
pub const DEFAULT_MODEL: &str = "korok_1";
pub const NOSE_MESH: &str = "nose";

#[derive(Debug, Clone)]
pub struct CurrentSelections {
    pub final_choices: FinalChoices,
    pub ready: bool,
    pub curr_leaf_type: String,
    pub curr_leaf_color: String,
    pub curr_wood_color: String,
}

impl CurrentSelections {
    pub fn default() -> Self {
        Self {
            final_choices: FinalChoices::default(),
            ready: false,
            curr_leaf_type: LOBBY_STARTING_MODEL.to_string(),
            curr_leaf_color: String::new(),
            curr_wood_color: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalChoices {
    pub color: HashMap<String, MeshColor>,
    pub materials: HashMap<String, String>,
    pub model: ModelIndex,
}

impl FinalChoices {
    fn default() -> Self {
        Self {
            color: HashMap::new(),
            materials: HashMap::new(),
            model: LOBBY_STARTING_MODEL.to_owned(),
        }
    }
}
