use std::{io::BufReader, fs::File, sync::{Mutex, Arc}};
use ambisonic::{rodio::{self, Decoder, source::{Source, Buffered}}, AmbisonicBuilder};
use instant::{SystemTime, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use common::core::{events::SoundSpec, states::GameState};

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum AudioAsset {
    BACKGROUND = 0,
    WIND = 1,
    STEP = 2,
}

pub struct SoundInstance{
    controller: ambisonic::SoundController,
    position: glm::Vec3,
    start: SystemTime,
}

pub struct Audio {
    audio_scene: ambisonic::Ambisonic,
    audio_assets: Vec<(Buffered<Decoder<BufReader<File>>>, Duration)>,
    sound_controllers_fx: HashMap<AudioAsset, Vec<SoundInstance>>,
    sound_controller_background: (Option<ambisonic::SoundController>, bool),
    time: SystemTime,
    pub sfx_queue: Arc<Mutex<Vec<SoundSpec>>>,
}

impl Audio {
    pub fn new() -> Self{        
        Audio {
            audio_scene: AmbisonicBuilder::default().build(),
            audio_assets: Vec::new(),
            sound_controllers_fx: HashMap::new(),
            sound_controller_background: (None, true),
            time: SystemTime::now(),
            sfx_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn play_background_track(&mut self, pos: [f32; 3]){
        let source = self.audio_assets[AudioAsset::BACKGROUND as usize].0.clone().repeat_infinite();
        let sound = self.audio_scene.play_at(source.convert_samples(), pos);
        self.sound_controller_background = (Some(sound), false);
    }

    // function in case a button to mute music will be added in the future:
    pub fn toggle_background_track(&mut self){
        if self.time.elapsed().unwrap_or(Duration::new(1,0)) > Duration::new(0,250000000) {
            let (sound, paused) = &mut self.sound_controller_background;
            if *paused {
                sound.as_ref().unwrap().resume();
                *paused = false;
            }
            else{
                sound.as_ref().unwrap().pause();
                *paused = true;
            }
            self.time = SystemTime::now();
        }
    }

    pub fn update_sound_positions(&mut self, player_pos: glm::Vec3, dir: glm::Vec3){
        let mut to_remove = Vec::new();
        for (asset, sound_instances) in self.sound_controllers_fx.iter_mut() {
            let sound_duration = self.audio_assets[*asset as usize].1;
            if sound_instances.len() != 0 {println!("{:#?} LEN: {}", asset, sound_instances.len());} else {println!("ZERO");}
            for i in 0..sound_instances.len() {
                if sound_instances[i].start.elapsed().unwrap_or(Duration::new(1,0)) >= sound_duration {
                    to_remove.push(i);
                }
                else{
                    let pos = relative_position(sound_instances[i].position, player_pos, dir);
                    if !basically_zero(pos) {
                        sound_instances[i].controller.adjust_position([pos.x, pos.z, 0.0]);
                    }
                    else {
                        sound_instances[i].controller.adjust_position([0.0, 1.0, 0.0]);
                    }
                }
            }
            for _ in 0..to_remove.len() {
                sound_instances.remove(to_remove.pop().unwrap());
            }
            to_remove.clear();
        }
    }

    pub fn handle_sfx_event(&mut self, sfxevent: SoundSpec){
        let index = to_audio_asset(sfxevent.sound_id).unwrap();
        let sound = self.audio_scene.play_at(
            self.audio_assets[index as usize].0.clone().convert_samples(), 
            [sfxevent.position.x, sfxevent.position.z, sfxevent.position.y]
        ); // double check y,z should be switched

        let sound_vec = self.sound_controllers_fx.get_mut(&index);
        match sound_vec {
            Some(vec) => {
                vec.push(SoundInstance {
                    controller: sound,
                    position: sfxevent.position,
                    start: SystemTime::now(),
                });
            }
            None => {
                self.sound_controllers_fx.insert(
                    index,
                    vec![SoundInstance {
                        controller: sound,
                        position: sfxevent.position,
                        start: SystemTime::now(),
                    }],
                );
            }
        }    
    }           

    pub fn handle_audio_updates(&mut self, game_state: Arc<Mutex<GameState>>, client_id: u8){
        loop {
            let mut sfx_queue = self.sfx_queue.lock().unwrap().clone();
            if !sfx_queue.is_empty() {
                for i in 0..sfx_queue.len() {
                    self.handle_sfx_event(sfx_queue[i].clone());
                }
                sfx_queue.clear();
                *self.sfx_queue.lock().unwrap() = sfx_queue;
            }

            let gs = game_state.lock().unwrap().clone();
            let player_curr = gs.player(client_id as u32)
                .ok_or_else(|| print!("")); //Player {} not found", client_id));

            match player_curr{
                Ok(player) => {
                    let cf = player_curr.unwrap().camera_forward;
                    self.update_sound_positions(player.transform.translation, cf); 
                }
                _ => {}
            }
        }
    }
}

pub fn to_audio_asset(sound_id: String) -> Option<AudioAsset> {
    match sound_id.as_str() {
        "wind" => Some(AudioAsset::WIND),
        "foot_step" => Some(AudioAsset::STEP),
        _ => None
    }
}

pub fn relative_position(sound_position: glm::Vec3, player_pos: glm::Vec3, dir: glm::Vec3) -> glm::Vec3{
    let new_dir = glm::Vec3::new(dir.x, 0.0, dir.z);
    let rel_pos = sound_position - player_pos;
    let new_pos = glm::Vec3::new(rel_pos.x, 0.0, rel_pos.z);
    let up = glm::Vec3::new(0.0, 1.0, 0.0);
    let right_dir = glm::cross(&new_dir, &up);

    let dot = right_dir.x*new_pos.x + right_dir.z*new_pos.z;
    let det = right_dir.x*new_pos.z - right_dir.z*new_pos.x;
    let angle = glm::atan2(&glm::Vec1::new(det), &glm::Vec1::new(dot)).x; // theta    

    let r = glm::magnitude(&rel_pos);
    let x = r * glm::cos(&glm::Vec1::new(-angle)).x;
    let z = r * glm::sin(&glm::Vec1::new(-angle)).x;
    
    // println!("Position: {}", new_pos);
    // println!("x: {}, and z: {}", x, z);
    // println!("Determinant: {}", det);
    // println!("Angle: {}", glm::degrees(&glm::Vec1::new(angle)));
    glm::Vec3::new(x, 0.0, z) 
}

pub fn basically_zero(position: glm::Vec3) -> bool {
    let abs_pos = glm::abs(&position);
    abs_pos.x < 0.1 && abs_pos.y < 0.1 && abs_pos.z < 0.1
}

// audio assets from config file
impl Audio {
    pub fn from_config(json: &ConfigAudioAssets) -> Self {
        let mut audio = Self::new();

        for sound in &json.sounds {
            let file = BufReader::new(File::open(sound.path.clone()).unwrap());
            let source = Decoder::new(file).unwrap().buffered();

            audio.audio_assets.push((source, Duration::new(sound.seconds, sound.nanoseconds)));
        }
        audio
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioElement {
    pub name: String,
    pub path: String,
    pub seconds: u64,
    pub nanoseconds: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigAudioAssets {
    pub sounds: Vec<AudioElement>,
}

// let side = glm::Vec3::new(1.0, 0.0, 0.0);
// let new_pos_2 = glm::Vec3::new(rel_pos.x, rel_pos.y, 0.0);
// let dot2 = side.x*new_pos_2.x + side.y*new_pos_2.y;
// let det2 = side.x*new_pos_2.y - side.y*new_pos_2.x;
// let angle2 = glm::atan2(&glm::Vec1::new(det2), &glm::Vec1::new(dot2)).x; // phi
// let r = glm::magnitude(&rel_pos);
// let x = r * glm::cos(&glm::Vec1::new(-angle)).x * glm::sin(&glm::Vec1::new(-angle2)).x;
// let z = r * glm::sin(&glm::Vec1::new(-angle)).x * glm::sin(&glm::Vec1::new(-angle2)).x;
// let y = r * glm::cos(&glm::Vec1::new(-angle2)).x;