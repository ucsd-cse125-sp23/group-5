use crate::configs::model_config::ModelIndex;
use crate::core::mesh_color::MeshColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const LOBBY_STARTING_MODEL: &str = "cube";
pub const LOBBY_STARTING_TYPE: &str = "leaf";
pub const LOBBY_STARTING_TYPE_BTN_ID: &str = "cust_leaf";
pub const LOBBY_STARTING_TYPE_DEF_TEXTURE: &str = "btn:leaf";

#[derive(Debug)]
pub struct CustomizationChoices {
    pub color: HashMap<String, MeshColor>,
    pub current_model: ModelIndex,
    pub prev_color_selection: (String, String), // (btn_name, default_texture)
    pub prev_type_selection: (String, String),
    pub cur_leaf_color: String,
    pub cur_body_color: String,
    pub current_type_choice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalChoices {
    pub color: HashMap<String, MeshColor>,
    pub model: ModelIndex,
}

impl FinalChoices {
    pub fn new(choices: &CustomizationChoices) -> Self {
        Self {
            color: choices.color.clone(),
            model: choices.current_model.clone(),
        }
    }
}

impl CustomizationChoices {
    pub fn default() -> Self {
        // TODO: fix later, hard-coded with constants for now
        Self {
            color: HashMap::new(),
            current_model: LOBBY_STARTING_MODEL.to_owned(),
            current_type_choice: LOBBY_STARTING_TYPE.to_owned(),
            prev_type_selection: (
                LOBBY_STARTING_TYPE_BTN_ID.to_owned(),
                LOBBY_STARTING_TYPE_DEF_TEXTURE.to_owned(),
            ),
            prev_color_selection: (String::new(), String::new()),
            cur_leaf_color: String::new(),
            cur_body_color: String::new(),
        }
    }
}
