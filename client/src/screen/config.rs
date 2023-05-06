// NOTE: all units are relative to either screen height or screen width
//    with the exception of aspect ratio, which is unitless

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDisplayGroup {
    pub id: String,
    pub screen: Option<ConfigScreen>,
    pub scene: Option<String>, // To load the scene graph
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigScreen {
    pub id: String,
    pub background: Option<ConfigScreenBackground>,
    pub items: Vec<ConfigScreenObject>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConfigScreenBackground{
    pub file: String,
    pub aspect: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigScreenObject{
    ConfigButton{
        pub location: ScreenLocation,
        pub aspect: f32,    // both textures must be the same aspect ratio
        pub default_tint: [f32; 4],
        pub hover_tint: [f32; 4],
        pub default_tex: String,
        pub hover_tex: String,
        pub on_click: Option<String>,
    },
    ConfigIcon{
        pub location: ScreenLocation,
        pub aspect: f32,
        pub tint: [f32; 4],
        pub tex: String,
        pub instances: Vec<ConfigTransform>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScreenLocation{
    // each tuple represents (rel_width, rel_height)
    // e.g. (0.5, 0.5) means a displacement of 1/4th the height + 1/4th the width
    vert_disp: (f32, f32),
    horz_disp: (f32, f32),
}