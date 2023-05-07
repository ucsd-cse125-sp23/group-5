use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use anyhow::Context;
use derive_more::Constructor;
use log::error;
use crate::model::{Material, Mesh, Model, StaticModel};
use crate::resources::{find_in_search_path, ModelLoadingResources};

type AnimationId = String;

#[derive(Debug)]
pub struct AnimatedModel {
    path: String,
    animations: HashMap<AnimationId, Animation>,
    active_animation: Option<AnimationId>,
}

#[derive(Constructor)]
pub struct Keyframe {
    frame_model: StaticModel,
    time: f32,
}

impl Debug for Keyframe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyframe")
            .field("frame_model", &self.frame_model.path)
            .field("time", &self.time)
            .finish()
    }
}

#[derive(Debug)]
pub struct Animation {
    path: String,
    name: String,
    keyframes: Vec<Keyframe>,
    current_time: f32,
}

impl AnimatedModel {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            animations: HashMap::new(),
            active_animation: None,
        }
    }

    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.insert(animation.name.clone(), animation);
    }

    pub fn set_active_animation(&mut self, animation_id: &str) {
        self.active_animation = Some(animation_id.to_string());
    }

    pub fn get_active_animation(&self) -> Option<&Animation> {
        match &self.active_animation {
            Some(animation_id) => self.animations.get(animation_id),
            None => None,
        }
    }

    pub async fn load_animation(&mut self, path: &str, res: ModelLoadingResources<'_>) -> anyhow::Result<()> {
        let path_buf = find_in_search_path(path).context("Could not find animation file")?;
        // name is the filename without the extension
        let name = path_buf
            .file_stem().context("Could not get animation name")?
            .to_str().context("Could not convert animation name to string")?;

        println!("Loading animation {} from {}", name, path);

        let animation = Animation::load_from_dir(path_buf.to_str().context("Could not convert animation path to string")?, name, res).await?;


        self.add_animation(animation);
        Ok(())
    }

    pub async fn load_all_animations_from_dir(&mut self, path: &str, res: ModelLoadingResources<'_>) -> anyhow::Result<()> {
        let dir = find_in_search_path(path).context("Could not find animation directory")?;
        let dir = std::fs::read_dir(dir).context("Could not read animation directory")?;

        for entry in dir {
            let entry = entry.context("Could not read animation directory entry")?;
            let path = entry.path();
            let path = path.to_str().context("Could not convert animation path to string")?;
            if self.load_animation(path, res).await.is_err() {
                error!("Skipping animation {}", path);
            }
        }

        Ok(())
    }
}

impl Animation {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            keyframes: Vec::new(),
            current_time: 0.0,
        }
    }

    /// load all obj files in the directory as keyframes, sorted by name, using a default frame rate
    pub async fn load_from_dir(path: &str, name: &str, res: ModelLoadingResources<'_>) -> anyhow::Result<Self> {

        // read the directory
        let mut dir = std::fs::read_dir(path)?;
        let mut animation = Animation::new(path, name);

        const DEFAULT_FRAME_RATE: f32 = 24.0;
        let mut time = 0.0;

        //sort the files by name
        let mut entries: Vec<_> = dir
            .by_ref()
            // filter out any files that aren't obj files
            .filter(|entry| {
                let entry = entry.as_ref().unwrap();
                entry.file_name().to_str().unwrap().ends_with(".obj")
            })
            .map(|entry| entry.unwrap())
            .collect();
        entries.sort_by_key(|entry| entry.file_name());

        //load each file as a keyframe
        for entry in entries {
            animation.load_keyframe(entry.path().to_str().unwrap(), time, res).await?;
            time += 1.0 / DEFAULT_FRAME_RATE;
        }

        Ok(animation)
    }

    pub async fn load_keyframe(&mut self,
                               path: &str,
                               time: f32,
                               res: ModelLoadingResources<'_>,
    ) -> anyhow::Result<()> {
        let frame_model = StaticModel::load(path, res).await?;
        let keyframe = Keyframe::new(frame_model, time);
        self.add_keyframe(keyframe);
        Ok(())
    }

    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
    }

    pub fn get_current_keyframe(&self) -> &Keyframe {
        let mut current_keyframe = &self.keyframes[0];
        for keyframe in &self.keyframes {
            if keyframe.time > self.current_time {
                break;
            }
            current_keyframe = keyframe;
        }
        current_keyframe
    }
}

impl Model for AnimatedModel {
    fn meshes(&self) -> &[Mesh] {
        match self.get_active_animation() {
            Some(animation) => animation.get_current_keyframe().frame_model.meshes(),
            None => &[],
        }
    }

    fn materials(&self) -> &[Material] {
        match self.get_active_animation() {
            Some(animation) => animation.get_current_keyframe().frame_model.materials(),
            None => &[],
        }
    }
}