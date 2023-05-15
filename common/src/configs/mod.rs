pub mod audio_config;
pub mod display_config;
pub mod model_config;
pub mod physics_config;
pub mod player_config;
pub mod scene_config;
pub mod texture_config;

use once_cell::sync::Lazy as OnceCellLazy;
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::configs::audio_config::ConfigAudioAssets;
use crate::configs::display_config::ConfigDisplay;
use crate::configs::model_config::ConfigModels;
use crate::configs::player_config::ConfigPlayer;
use crate::configs::scene_config::ConfigSceneGraph;
use crate::configs::texture_config::ConfigTexture;

const MODELS_CONFIG_PATH: &str = "models.json";
const SCENE_CONFIG_PATH: &str = "scene.json";
const LOBBY_SCENE_CONFIG_PATH: &str = "lobby_scene.json";
const AUDIO_CONFIG_PATH: &str = "audio.json";
const PLAYER_CONFIG_PATH: &str = "player.json";
const DISPLAY_CONFIG_PATH: &str = "display.json";
const TEXTURE_CONFIG_PATH: &str = "tex.json";

// pub static CONFIG_INSTANCE: OnceCellLazy<RwLock<Option<Arc<Config>>>> =
//     OnceCellLazy::new(|| RwLock::new(None));

pub static CONFIG_INSTANCE: OnceCellLazy<RwLock<Option<Arc<Config>>>> = OnceCellLazy::new(|| {
    let models: ConfigModels = from_file(MODELS_CONFIG_PATH).expect("Failed to load models config");
    let scene: ConfigSceneGraph =
        from_file(SCENE_CONFIG_PATH).expect("Failed to load scene config");
    let lobby_scene: ConfigSceneGraph =
        from_file(LOBBY_SCENE_CONFIG_PATH).expect("Failed to load scene config");
    let audio: ConfigAudioAssets =
        from_file(AUDIO_CONFIG_PATH).expect("Failed to load audio config");
    let player: ConfigPlayer = from_file(PLAYER_CONFIG_PATH).expect("Failed to load player config");
    let display: ConfigDisplay =
        from_file(DISPLAY_CONFIG_PATH).expect("Failed to load display config");
    let texture: ConfigTexture =
        from_file(TEXTURE_CONFIG_PATH).expect("Failed to load texture config");
    let config = Config::new(models, scene, lobby_scene, audio, player, display, texture);
    RwLock::new(Some(Arc::new(config)))
});

pub struct Config {
    pub models: ConfigModels,
    pub scene: ConfigSceneGraph,
    pub lobby_scene: ConfigSceneGraph,
    pub audio: ConfigAudioAssets,
    pub player: ConfigPlayer,
    pub display: ConfigDisplay,
    pub texture: ConfigTexture,
}

impl Config {
    pub fn new(
        models: ConfigModels,
        scene: ConfigSceneGraph,
        lobby_scene: ConfigSceneGraph,
        audio: ConfigAudioAssets,
        player: ConfigPlayer,
        display: ConfigDisplay,
        texture: ConfigTexture,
    ) -> Self {
        Config {
            models,
            scene,
            lobby_scene,
            audio,
            player,
            display,
            texture,
        }
    }
}

pub mod ConfigurationManager {
    use super::*;

    // pub fn load_configuration() -> Result<(), Box<dyn std::error::Error>> {
    //     let models: ConfigModels = from_file(MODELS_CONFIG_PATH)?;
    //     let scene: ConfigSceneGraph = from_file(SCENE_CONFIG_PATH)?;
    //     let audio: ConfigAudioAssets = from_file(AUDIO_CONFIG_PATH)?;
    //     let player: ConfigPlayer = from_file(PLAYER_CONFIG_PATH)?;
    //     let display: ConfigDisplay = from_file(DISPLAY_CONFIG_PATH)?;
    //     let texture: ConfigTexture = from_file(TEXTURE_CONFIG_PATH)?;
    //
    //     let config = Config::new(models, scene, audio, player, display, texture);
    //     *CONFIG_INSTANCE.write().unwrap() = Some(Arc::new(config));
    //     Ok(())
    // }

    pub fn get_configuration() -> Arc<Config> {
        CONFIG_INSTANCE
            .read()
            .unwrap()
            .as_ref()
            .expect("Configuration not loaded.")
            .clone()
    }
}

pub fn from_file<P: AsRef<Path>, S: serde::de::DeserializeOwned>(
    path: P,
) -> Result<S, serde_json::Error> {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read the file");
    serde_json::from_str(&contents)
}

pub fn to_file<P: AsRef<Path>, S: Serialize>(s: &S, path: P) -> Result<(), serde_json::Error> {
    let serialized = serde_json::to_string_pretty(s)?;
    let mut file = File::create(path).expect("Unable to create the file");
    file.write_all(serialized.as_bytes())
        .expect("Unable to write to the file");
    Ok(())
}
