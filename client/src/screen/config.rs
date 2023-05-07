// NOTE: all units are relative to either screen height or screen width
//    with the exception of aspect ratio, which is unitless

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplays{
    pub displays: Vec<ConfigDisplayGroup>,
    pub default_display: String,
    pub game_display: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplayGroup {
    pub id: String,
    pub screen: Option<ConfigScreen>,
    pub scene: Option<String>, // To load the scene graph
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreen {
    pub background: Option<ConfigScreenBackground>,
    pub buttons: Vec<ConfigButton>,
    pub icons: Vec<ConfigIcon>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConfigScreenBackground{
    pub file: String,
    pub aspect: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigButton{
    pub location: ScreenLocation,
    pub aspect: f32,    // both textures must be the same aspect ratio
    pub height: f32,    // relative to screen height
    pub default_tint: [f32; 4],
    pub hover_tint: [f32; 4],
    pub default_tex: String,
    pub hover_tex: String,
    pub on_click: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigIcon{
    pub location: ScreenLocation,
    pub aspect: f32,
    pub height: f32,
    pub tint: [f32; 4],
    pub tex: String,
    pub instances: Vec<ConfigScreenTransform>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreenTransform{
    // Scale, then rotate, then translate
    // No shearing...
    pub translation: ScreenLocation,
    pub rotation: f32, // in radians, rotating CCW
    pub scale: (f32, f32),
}