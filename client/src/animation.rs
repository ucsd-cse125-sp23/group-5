use crate::model::{Material, Mesh, Model, StaticModel};
use crate::resources::{find_in_search_path, ModelLoadingResources};
use anyhow::Context;
use derive_more::Constructor;
use log::{error, info};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
extern crate nalgebra_glm as glm;
use crate::scene::{NodeId, NodeKind};

use common::core::states::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AnimationState {
    Playing {
        animation_id: AnimationId,
        time: f32,
    },
    Stopped,
}

#[derive(Debug, Default)]
pub struct AnimationController {
    // to keep record of which node is playing which animation
    animation_states: HashMap<NodeId, AnimationState>,
}

impl AnimationController {
    pub fn update_animated_model_state(
        &mut self,
        object: &mut Box<dyn Model>,
        node_id: &NodeId,
    ) -> Option<()> {
        let animation_state = self.animation_states.get(node_id)?;
        let model = object;
        if let Some(animated_model) = model.as_any_mut().downcast_mut::<AnimatedModel>() {
            let next_state = if let AnimationState::Playing { animation_id, time } = animation_state
            {
                let animation = animated_model.animations.get(animation_id)?;
                let animation_duration = animation.duration();
                let cyclic = animation.cyclic;

                if *time > animation_duration {
                    if cyclic {
                        AnimationState::Playing {
                            animation_id: animation_id.clone(),
                            time: *time % animation_duration,
                        }
                    } else {
                        AnimationState::Stopped
                    }
                } else {
                    AnimationState::Playing {
                        animation_id: animation_id.clone(),
                        time: *time,
                    }
                }
            } else {
                AnimationState::Stopped
            };

            animated_model.set_active_animation_state(next_state);
        }
        None
    }

    pub fn play_animation(&mut self, animation_id: AnimationId, node_id: NodeId) {
        // println!("Playing animation {:?}", animation_id);

        // if already player, do nothing
        if let Some(AnimationState::Playing {
            animation_id: playing_animation_id,
            ..
        }) = self.animation_states.get(&node_id)
        {
            if playing_animation_id == &animation_id {
                return;
            }
        }

        // println!("Playing animation {:?}", animation_id);
        self.animation_states.insert(
            node_id,
            AnimationState::Playing {
                animation_id,
                time: 0.0,
            },
        );
    }

    pub fn stop_animation(&mut self, node_id: NodeId) {
        self.animation_states
            .insert(node_id, AnimationState::Stopped);
    }

    pub fn update(&mut self, dt: Duration) {
        for (_node_id, animation_state) in self.animation_states.iter_mut() {
            match animation_state {
                AnimationState::Playing {
                    animation_id: _,
                    time,
                } => {
                    *time += dt.as_secs_f32();
                }
                AnimationState::Stopped => {}
            }
        }
    }

    pub fn load_game_state(&mut self, game_state: impl Deref<Target = GameState>) {
        for player_state in game_state.players.values() {
            let node_id = NodeKind::Player.node_id(player_state.id.to_string());

            let action_state = player_state
                .active_action_states
                .iter()
                .map(|(action_state, _)| action_state)
                .max_by_key(|action_state| action_state.priority());

            if let Some(action_state) = action_state {
                self.play_animation(action_state.animation_id().to_string(), node_id);
            } else {
                self.stop_animation(node_id);
            }
        }
    }
}

type AnimationId = String;

#[derive(Debug, Clone)]
pub struct AnimatedModel {
    path: String,
    animations: HashMap<AnimationId, Animation>,
    default_animation: Option<AnimationId>,
    active_animation: AnimationState,
}

#[derive(Constructor, Clone)]
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

// some thing like {"attack": {
//   "cyclic": true,
// },

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDesc {
    cyclic: bool,
}

impl Default for AnimationDesc {
    fn default() -> Self {
        Self { cyclic: true }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    path: String,
    name: String,
    keyframes: Vec<Keyframe>,
    cyclic: bool,
    state: AnimationState,
}

impl AnimatedModel {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            animations: HashMap::new(),
            default_animation: None,
            active_animation: AnimationState::Stopped,
        }
    }
    pub fn default_animation(&self) -> &Animation {
        self.animations
            .get(self.default_animation.as_ref().unwrap())
            .unwrap()
    }

    pub fn add_animation(&mut self, animation: Animation) {
        // if there is no idle animation, set the first animation as the idle
        if self.default_animation.is_none() || animation.name == "idle" {
            self.default_animation = Some(animation.name.clone());
        }

        self.animations.insert(animation.name.clone(), animation);
    }

    pub fn set_active_animation_state(&mut self, animation_state: AnimationState) {
        self.active_animation = animation_state;
    }

    pub fn active_animation(&self) -> Option<&Animation> {
        match &self.active_animation {
            AnimationState::Playing { animation_id, .. } => self.animations.get(animation_id),
            AnimationState::Stopped => self.default_animation().into(),
        }
    }

    pub fn get_active_key_frame(&self) -> Option<&Keyframe> {
        if let Some(animation) = self.active_animation() {
            let time = match &self.active_animation {
                AnimationState::Playing { time, .. } => *time,
                AnimationState::Stopped => 0.0,
            };

            Some(animation.get_keyframe(time))
        } else {
            None
        }
    }

    pub async fn load_animation(
        &mut self,
        path: &str,
        res: ModelLoadingResources<'_>,
        desc: &AnimationDesc,
    ) -> anyhow::Result<()> {
        let path_buf = find_in_search_path(path).context("Could not find animation file")?;
        // name is the filename without the extension
        let name = path_buf
            .file_stem()
            .context("Could not get animation name")?
            .to_str()
            .context("Could not convert animation name to string")?;

        info!("Loading animation {} from {}", name, path);

        let animation = Animation::load_from_dir(
            path_buf
                .to_str()
                .context("Could not convert animation path to string")?,
            name,
            res,
            desc,
        )
        .await?;

        self.add_animation(animation);

        Ok(())
    }

    pub async fn load_all_animations_from_dir(
        &mut self,
        path: &str,
        res: ModelLoadingResources<'_>,
    ) -> anyhow::Result<()> {
        let dir = find_in_search_path(path).context("Could not find animation directory")?;
        let dir = std::fs::read_dir(dir).context("Could not read animation directory")?;

        // read desc.json into hashmap of animation desc (key is the

        // {"attack": {
        //     "cyclic": true,
        // }, ...

        let mut animation_descs = HashMap::new();
        let desc_path = format!("{}/desc.json", path);

        if let Ok(desc_file) = File::open(desc_path) {
            let reader = BufReader::new(desc_file);
            let desc: HashMap<String, AnimationDesc> =
                serde_json::from_reader(reader).context("Could not parse animation desc file")?;
            animation_descs = desc;
        }

        for entry in dir {
            let entry = entry.context("Could not read animation directory entry")?;

            // if entry is a file, skip it
            if entry.file_type().unwrap().is_file() {
                continue;
            }

            let path = entry.path();
            let path = path
                .to_str()
                .context("Could not convert animation path to string")?;

            let path_buf = PathBuf::from(path);
            let name = path_buf
                .file_stem()
                .context("Could not get animation name")?
                .to_str()
                .context("Could not convert animation name to string")?;

            if let Err(e) = self
                .load_animation(
                    path,
                    res,
                    animation_descs
                        .get(name)
                        .unwrap_or(&AnimationDesc::default()),
                )
                .await
            {
                error!("Skipping animation {}: {}", path, e);
            }
        }

        Ok(())
    }

    pub async fn load(path: &str, res: ModelLoadingResources<'_>) -> anyhow::Result<Self> {
        let mut animated_model = Self::new(path);
        animated_model
            .load_all_animations_from_dir(path, res)
            .await?;
        Ok(animated_model)
    }
}

impl Animation {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            keyframes: Vec::new(),
            state: AnimationState::Stopped,
            cyclic: true,
        }
    }

    pub fn duration(&self) -> f32 {
        // TODO: we should have a separate duration field
        self.keyframes.last().unwrap().time
    }

    /// load all obj files in the directory as keyframes, sorted by name, using a idle frame rate
    pub async fn load_from_dir(
        path: &str,
        name: &str,
        res: ModelLoadingResources<'_>,
        desc: &AnimationDesc,
    ) -> anyhow::Result<Self> {
        // read the directory
        let mut dir = std::fs::read_dir(path)?;
        let mut animation = Animation::new(path, name);

        animation.cyclic = desc.cyclic;

        const DEFAULT_FRAME_RATE: f32 = 24.0;
        let mut time = 0.0;

        let mut entries: Vec<_> = dir
            .by_ref()
            // filter out any files that aren't obj files
            .filter(|entry| {
                let entry = entry.as_ref().unwrap();
                entry.file_name().to_str().unwrap().ends_with(".obj")
            })
            .map(|entry| entry.unwrap())
            .collect();
        //sort the files by the numerical part of the name
        entries.sort_by(|a, b| {
            let a = a
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches(|c: char| !c.is_numeric())
                .trim_end_matches(|c: char| !c.is_numeric())
                .parse::<i32>()
                .unwrap();
            let b = b
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches(|c: char| !c.is_numeric())
                .trim_end_matches(|c: char| !c.is_numeric())
                .parse::<i32>()
                .unwrap();

            a.cmp(&b)
        });

        //load each file as a keyframe
        for entry in entries {
            animation
                .load_keyframe(entry.path().to_str().unwrap(), time, res)
                .await?;
            time += 1.0 / DEFAULT_FRAME_RATE;
        }

        Ok(animation)
    }

    pub async fn load_keyframe(
        &mut self,
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

    pub fn get_keyframe(&self, time: f32) -> &Keyframe {
        let mut current_keyframe = &self.keyframes[0];
        for keyframe in &self.keyframes {
            if keyframe.time > time {
                break;
            }
            current_keyframe = keyframe;
        }
        current_keyframe
    }
}

impl Model for AnimatedModel {
    fn meshes(&self) -> &[Mesh] {
        match self.get_active_key_frame() {
            Some(key_frame) => key_frame.frame_model.meshes(),
            None => &[],
        }
    }

    fn materials(&self) -> &[Material] {
        match self.get_active_key_frame() {
            Some(key_frame) => key_frame.frame_model.materials(),
            None => &[],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Model> {
        Box::new(self.clone())
    }
}
