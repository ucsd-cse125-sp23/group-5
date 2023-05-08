pub mod model_config;
pub mod physics_config;
pub mod player_config;
pub mod scene_config;

use std::env;
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use structopt::lazy_static;

use crate::configs::model_config::ConfigModels;
use crate::configs::scene_config::ConfigSceneGraph;

const MODELS_CONFIG_PATH: &str = "models.json";
const SCENE_CONFIG_PATH: &str = "scene.json";

pub struct Config {
    pub models: ConfigModels,
    pub scene: ConfigSceneGraph,
}

impl Config {
    fn new(models: ConfigModels, scene: ConfigSceneGraph) -> Self {
        Config { models, scene }
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG_INSTANCE: Arc<Mutex<Option<Config>>> = Arc::new(Mutex::new(None));
}

pub fn load_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let models: ConfigModels = from_file(MODELS_CONFIG_PATH)?;

    #[cfg(test)]
    println!("{:?}", env::current_dir());
    let scene: ConfigSceneGraph = from_file("../scene.json")?;

    #[cfg(not(test))]
    let scene: ConfigSceneGraph = from_file(SCENE_CONFIG_PATH)?;

    let config = Config::new(models, scene);
    let mut instance = CONFIG_INSTANCE.lock().unwrap();
    *instance = Some(config);

    Ok(())
}

pub fn get_configuration() -> Arc<Mutex<Option<Config>>> {
    CONFIG_INSTANCE.clone()
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
