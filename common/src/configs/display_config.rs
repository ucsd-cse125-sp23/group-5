use serde::{Deserialize, Serialize};

// NOTE: all units are relative to either screen height or screen width
//    with the exception of aspect ratio, which is unitless
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplay {
    pub displays: Vec<ConfigDisplayGroup>,
    pub default_display: String,
    pub game_display: String,
    pub screens: Vec<ConfigScreen>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplayGroup {
    pub id: String,
    pub screen: Option<String>,
    pub scene: Option<String>, // To load the scene graph
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreen {
    pub id: String,
    pub background: Option<ConfigScreenBackground>,
    pub buttons: Vec<ConfigButton>,
    pub icons: Vec<ConfigIcon>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreenBackground {
    pub tex: String,
    pub aspect: f32,
    pub mask_tex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigButton {
    pub id: Option<String>,
    pub location: ScreenLocation,
    pub aspect: f32, // both textures must be the same aspect ratio
    pub height: f32, // relative to screen height
    pub default_tint: [f32; 4],
    pub hover_tint: [f32; 4],
    pub default_tex: String,
    pub hover_tex: String,
    pub selected_tex: Option<String>,
    pub mask_tex: String,
    pub on_click: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigIcon {
    pub id: String,
    pub location: ScreenLocation,
    pub aspect: f32,
    pub height: f32,
    pub tint: [f32; 4],
    pub tex: String,
    pub mask_tex: String,
    pub instances: Vec<ConfigScreenTransform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreenTransform {
    // Scale, then rotate, then translate
    // No shearing...
    pub translation: ScreenLocation,
    pub rotation: f32, // in radians, rotating CCW
    pub scale: (f32, f32),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ScreenLocation {
    // each tuple represents (rel_width, rel_height)
    // e.g. (0.5, 0.5) means a displacement of 1/4th the height + 1/4th the width
    pub vert_disp: (f32, f32),
    pub horz_disp: (f32, f32),
}
