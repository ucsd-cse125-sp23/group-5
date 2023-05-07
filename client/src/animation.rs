use std::any::Any;
use crate::model::{Material, Mesh, Model, StaticModel};
use crate::resources::{find_in_search_path, ModelLoadingResources};
use anyhow::{anyhow, Context};
use derive_more::Constructor;
use log::error;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::time::Duration;
use glm::scale;
use common::configs::model_config::ModelIndex;
use common::core::states::GameState;
use crate::scene::{Node, NodeId, NodeKind, Scene};

#[derive(Debug, Clone)]
enum AnimationState {
    Playing { animation_id: AnimationId, time: f32 },
    Stopped,
}

#[derive(Debug, Default)]
pub struct AnimationController {
    // to keep record of which node is playing which animation
    animation_states: HashMap<NodeId, AnimationState>,
}

impl AnimationController {
    pub fn update_animated_model_state(&mut self, object: &mut Box<dyn Model>, node_id: &NodeId) -> Option<()> {
        let animation_state = self.animation_states.get(node_id)?;
        let model = object;
        if let Some(animated_model) = model.as_any_mut().downcast_mut::<AnimatedModel>() {
            return match animation_state {
                AnimationState::Playing { animation_id, time } => {
                    animated_model.set_active_animation(animation_id);
                    animated_model.set_active_animation_time(*time);
                    Some(())
                }
                AnimationState::Stopped => {
                    animated_model.remove_active_animation();
                    Some(())
                }
            };
        }
        None
    }

    pub fn play_animation(&mut self, animation_id: AnimationId, node_id: NodeId) {
        // if already player, do nothing
        if let Some(AnimationState::Playing { animation_id: playing_animation_id, .. }) = self.animation_states.get(&node_id) {
            if playing_animation_id == &animation_id {
                return;
            }
        }


        println!("Playing animation {:?}", animation_id);
        self.animation_states.insert(node_id, AnimationState::Playing { animation_id, time: 0.0 });
    }

    pub fn stop_animation(&mut self, node_id: NodeId) {
        self.animation_states.insert(node_id, AnimationState::Stopped);
    }

    pub fn update(&mut self, dt: Duration) {
        for (node_id, animation_state) in self.animation_states.iter_mut() {
            match animation_state {
                AnimationState::Playing { animation_id, time } => {
                    *time += dt.as_secs_f32();
                }
                AnimationState::Stopped => {}
            }
        }
    }

    pub fn load_game_state(&mut self, game_state: impl Deref<Target = GameState>,) {
        for player_state in game_state.players.values() {
            let node_id =  NodeKind::Player.node_id(player_state.id.to_string());

            if let Some(animation_id) = player_state.animation_id.as_ref() {
                println!("Playing animation {:?} for {:?}", animation_id, node_id);
                self.play_animation(animation_id.to_string(), node_id);
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
    active_animation: Option<AnimationId>,
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

#[derive(Debug, Clone)]
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
            default_animation: None,
            active_animation: None,
        }
    }
    pub fn default_animation(&self) -> &Animation {
        self.animations.get(self.default_animation.as_ref().unwrap()).unwrap()
    }

    pub fn add_animation(&mut self, animation: Animation) {

        // if there is no idle animation, set the first animation as the idle
        if self.default_animation.is_none() || animation.name == "idle" {
            self.default_animation = Some(animation.name.clone());
        }

        self.animations.insert(animation.name.clone(), animation);
    }

    pub fn set_active_animation(&mut self, animation_id: &str) {
        self.active_animation = Some(animation_id.to_string());
    }

    pub fn remove_active_animation(&mut self) {
        self.set_active_animation_time(0.0);
        self.active_animation = None;
    }

    pub fn get_active_animation(&self) -> Option<&Animation> {
        match &self.active_animation {
            Some(animation_id) => self.animations.get(animation_id),
            None => self.animations.get(self.default_animation.as_ref().unwrap()),
        }
    }

    pub fn get_active_animation_mut(&mut self) -> Option<&mut Animation> {
        match &self.active_animation {
            Some(animation_id) => self.animations.get_mut(animation_id),
            None => self.animations.get_mut(self.default_animation.as_ref().unwrap()),
        }
    }

    pub fn set_active_animation_time(&mut self, time: f32) {
        if let Some(animation) = self.get_active_animation_mut() {
            animation.current_time = time % animation.duration();
        }
    }

    pub async fn load_animation(
        &mut self,
        path: &str,
        res: ModelLoadingResources<'_>,
    ) -> anyhow::Result<()> {
        let path_buf = find_in_search_path(path).context("Could not find animation file")?;
        // name is the filename without the extension
        let name = path_buf
            .file_stem()
            .context("Could not get animation name")?
            .to_str()
            .context("Could not convert animation name to string")?;

        println!("Loading animation {} from {}", name, path);

        let animation = Animation::load_from_dir(
            path_buf
                .to_str()
                .context("Could not convert animation path to string")?,
            name,
            res,
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

        for entry in dir {
            let entry = entry.context("Could not read animation directory entry")?;
            let path = entry.path();
            let path = path
                .to_str()
                .context("Could not convert animation path to string")?;
            if self.load_animation(path, res).await.is_err() {
                error!("Skipping animation {}", path);
            }
        }

        Ok(())
    }

    pub async fn load(
        path: &str,
        res: ModelLoadingResources<'_>,
    ) -> anyhow::Result<Self> {
        let mut animated_model = Self::new(path);
        animated_model.load_all_animations_from_dir(path, res).await?;
        Ok(animated_model)
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

    pub fn duration(&self) -> f32 {
        // TODO: we should have a separate duration field
        self.keyframes.last().unwrap().time
    }

    /// load all obj files in the directory as keyframes, sorted by name, using a idle frame rate
    pub async fn load_from_dir(
        path: &str,
        name: &str,
        res: ModelLoadingResources<'_>,
    ) -> anyhow::Result<Self> {
        // read the directory
        let mut dir = std::fs::read_dir(path)?;
        let mut animation = Animation::new(path, name);

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
            let a = a.file_name().to_str().unwrap()
                .trim_start_matches(|c: char| !c.is_numeric())
                .trim_end_matches(|c: char| !c.is_numeric())
                .parse::<i32>()
                .unwrap();
            let b = b.file_name().to_str().unwrap()
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
