use ambisonic::{
    rodio::{
        source::{Buffered, Source},
        Decoder,
    },
    AmbisonicBuilder,
};
use instant::{Duration, SystemTime};
use nalgebra_glm as glm;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
    thread,
};

use common::{configs::audio_config::ConfigAudioAssets, core::states::GameLifeCycleState};
use common::core::{events::SoundSpec, states::GameState};

pub const AUDIO_POS_AT_CLIENT: [f32; 3] = [0.0, 25.0, 0.0];
pub static CURR_DISP: OnceCell<Mutex<String>> = OnceCell::new();

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum AudioAsset {
    BKGND_MAIN = 0,
    BKGND_WINNER = 1,
    BKGND_LOSER = 2,
    WIND = 3,
    STEP = 4,
}

pub struct SoundInstance {
    controller: ambisonic::SoundController,
    position: glm::Vec3,
    start: SystemTime,
    initial_dir: glm::Vec3,
    at_client: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SoundQueue {
    sound_queue: Vec<SoundSpec>,
}

impl SoundQueue {
    pub fn add_sound(&mut self, sound: SoundSpec) {
        self.sound_queue.push(sound);
    }
}

pub struct Audio {
    audio_scene: ambisonic::Ambisonic,
    audio_assets: Vec<(Buffered<Decoder<BufReader<File>>>, Duration, f32)>,
    sound_controllers_fx: HashMap<AudioAsset, Vec<SoundInstance>>,
    sound_controller_background: (Option<ambisonic::SoundController>, bool),
    time: SystemTime,
    sfx_queue: Arc<Mutex<SoundQueue>>,
    curr_state: GameLifeCycleState,
}

impl Audio {
    pub fn new(q: Arc<Mutex<SoundQueue>>) -> Self {
        CURR_DISP.set(Mutex::new("display:title".to_string())).expect("failed to initialze CURR_DISP");
        Audio {
            audio_scene: AmbisonicBuilder::default().build(),
            audio_assets: Vec::new(),
            sound_controllers_fx: HashMap::new(),
            sound_controller_background: (None, true),
            time: SystemTime::now(),
            sfx_queue: q,
            curr_state: GameLifeCycleState::Waiting,
        }
    }

    pub fn play_background_track(&mut self, bkgd: AudioAsset, pos: [f32; 3]) {
        let source = self.audio_assets[bkgd as usize]
            .0
            .clone()
            .fade_in(Duration::new(1, 0))
            .repeat_infinite();
        let sound = self.audio_scene.play_at(source.convert_samples(), pos);
        self.sound_controller_background = (Some(sound), false);
    }

    pub fn update_bkgd_track(&mut self, state: GameLifeCycleState, curr_player: u32, winner: u32){
        if std::mem::discriminant(&self.curr_state) != std::mem::discriminant(&state) {
            match state {
                GameLifeCycleState::Waiting => {
                    if CURR_DISP.get().unwrap().lock().unwrap().clone() == "display:title".to_string() {
                        self.switch_background_track(AudioAsset::BKGND_MAIN, AUDIO_POS_AT_CLIENT);
                        self.curr_state = state;
                    }
                },
                GameLifeCycleState::Ended => {
                    if curr_player == winner {
                        self.switch_background_track(AudioAsset::BKGND_WINNER, AUDIO_POS_AT_CLIENT);
                    }
                    else {
                        self.switch_background_track(AudioAsset::BKGND_LOSER, AUDIO_POS_AT_CLIENT);
                    }
                    self.curr_state = state;
                },
                _ => {}
            }

        }
    }

    pub fn switch_background_track(&mut self, bkgd: AudioAsset, pos: [f32; 3]) {
        self.sound_controller_background.0.as_ref().unwrap().stop();
        self.play_background_track(bkgd, pos);
    }

    // function in case a button to mute music will be added in the future:
    pub fn toggle_background_track(&mut self) {
        if self.time.elapsed().unwrap_or(Duration::new(1, 0)) > Duration::new(0, 250000000) {
            let (sound, paused) = &mut self.sound_controller_background;
            if *paused {
                sound.as_ref().unwrap().resume();
                *paused = false;
            } else {
                sound.as_ref().unwrap().pause();
                *paused = true;
            }
            self.time = SystemTime::now();
        }
    }

    pub fn update_sound_positions(&mut self, player_pos: glm::Vec3, dir: glm::Vec3) {
        let mut to_remove = Vec::new();
        // for each sound effect
        for (asset, sound_instances) in self.sound_controllers_fx.iter_mut() {
            // for each sound event for each sound effect
            let sound_duration = self.audio_assets[*asset as usize].1;
            let percent = self.audio_assets[*asset as usize].2;
            for i in 0..sound_instances.len() {
                let time_elapsed = sound_instances[i]
                    .start
                    .elapsed()
                    .unwrap_or(Duration::new(1, 0));
                if time_elapsed >= sound_duration {
                    to_remove.push(i);
                } else if sound_instances[i].at_client {
                    sound_instances[i]
                        .controller
                        .adjust_position([0.0, 10.0, 0.0]);
                } else {
                    if true {
                        // in case some sound effects shouldn't get quieter the farther they get
                        let mut offset = sound_instances[i].initial_dir;
                        if offset != glm::Vec3::new(0.0, 0.0, 0.0) {
                            offset = glm::normalize(&offset);
                        }
                        offset = glm::Vec3::new(
                            offset.x * percent,
                            offset.y * percent,
                            offset.z * percent,
                        );
                        sound_instances[i].position += offset;
                        thread::sleep(Duration::from_millis(1));
                        //println!(""); // without this print_statement the sounds don't play; maybe need delay?
                    }
                    let pos = relative_position(sound_instances[i].position, player_pos, dir);
                    if !basically_zero(pos) {
                        sound_instances[i]
                            .controller
                            .adjust_position([pos.x, pos.z, 0.0]);
                    } else {
                        sound_instances[i]
                            .controller
                            .adjust_position([0.0, 0.1, 0.0]);
                    }
                }
            }

            // remove all sounds that have ended
            for _ in 0..to_remove.len() {
                sound_instances.remove(to_remove.pop().unwrap());
            }
            to_remove.clear();
        }
    }

    pub fn handle_sfx_event(&mut self, sfxevent: SoundSpec, dir: glm::Vec3, at_client: bool) {
        let index = to_audio_asset(sfxevent.sound_id).unwrap();
        let sound = self.audio_scene.play_at(
            self.audio_assets[index as usize]
                .0
                .clone()
                .convert_samples(),
            [
                sfxevent.position.x,
                sfxevent.position.z,
                sfxevent.position.y,
            ],
        ); // double check y,z should be switched

        let sound_vec = self.sound_controllers_fx.get_mut(&index);
        match sound_vec {
            Some(vec) => {
                vec.push(SoundInstance {
                    controller: sound,
                    position: sfxevent.position,
                    start: SystemTime::now(),
                    initial_dir: dir,
                    at_client,
                });
            }
            None => {
                self.sound_controllers_fx.insert(
                    index,
                    vec![SoundInstance {
                        controller: sound,
                        position: sfxevent.position,
                        start: SystemTime::now(),
                        initial_dir: dir,
                        at_client,
                    }],
                );
            }
        }
    }

    pub fn handle_audio_updates(&mut self, game_state: Arc<Mutex<GameState>>, client_id: u8) {
        loop {
            let gs = game_state.lock().unwrap().clone();
            let player_curr = gs.player(client_id as u32).ok_or_else(|| print!("")); //Player {} not found", client_id));
            let mut cf = glm::Vec3::new(0.0, 0.0, 0.0);
            let mut pos = glm::Vec3::new(0.0, 0.0, 0.0);

            self.update_bkgd_track(gs.life_cycle_state.clone(), client_id as u32, gs.game_winner.unwrap_or(0));

            match player_curr {
                Ok(player) => {
                    cf = player_curr.unwrap().camera_forward;
                    pos = player.transform.translation;
                }
                _ => {}
            }

            let mut sfx_queue = self.sfx_queue.lock().unwrap().clone();
            if !sfx_queue.sound_queue.is_empty() {
                for i in 0..sfx_queue.sound_queue.len() {
                    let se = sfx_queue.sound_queue[i].clone();
                    let at_client = se.at_client.0 == client_id as u32 && se.at_client.1;
                    self.handle_sfx_event(se, cf, at_client);
                }
                sfx_queue.sound_queue.clear();
                *self.sfx_queue.lock().unwrap() = sfx_queue;
            }

            self.update_sound_positions(pos, cf);
        }
    }
}

pub fn to_audio_asset(sound_id: String) -> Option<AudioAsset> {
    match sound_id.as_str() {
        "wind" => Some(AudioAsset::WIND),
        "foot_step" => Some(AudioAsset::STEP),
        _ => None,
    }
}

pub fn relative_position(
    sound_position: glm::Vec3,
    player_pos: glm::Vec3,
    dir: glm::Vec3,
) -> glm::Vec3 {
    let new_dir = glm::Vec3::new(dir.x, 0.0, dir.z);
    let rel_pos = sound_position - player_pos;
    let new_pos = glm::Vec3::new(rel_pos.x, 0.0, rel_pos.z);
    let up = glm::Vec3::new(0.0, 1.0, 0.0);
    let right_dir = glm::cross(&new_dir, &up);

    let dot = right_dir.x * new_pos.x + right_dir.z * new_pos.z;
    let det = right_dir.x * new_pos.z - right_dir.z * new_pos.x;
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
    pub fn from_config(json: &ConfigAudioAssets, sound_queue: Arc<Mutex<SoundQueue>>) -> Self {
        let mut audio = Self::new(sound_queue);

        for sound in &json.sounds {
            let file = BufReader::new(File::open(sound.path.clone()).unwrap());
            let source = Decoder::new(file).unwrap().buffered();

            audio.audio_assets.push((
                source,
                Duration::new(sound.seconds, sound.nanoseconds),
                sound.fall_off_speed,
            ));
        }
        audio
    }
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
