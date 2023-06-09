use ambisonic::{
    rodio::{
        source::{self, Buffered, Empty, Source},
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

use common::core::{events::SoundSpec, states::GameState};
use common::{configs::audio_config::ConfigAudioAssets, core::states::GameLifeCycleState};

pub const AUDIO_POS_AT_CLIENT: [f32; 3] = [0.0, 10.0, 0.0];
pub const FADE_DIST: f32 = 50.0;
pub const SOUND_RADIUS: f32 = 25.0;
pub const RADIUS_OFFSET: f32 = 0.2;
pub const RADIUS_MUL: f32 = 1.0;

pub static CURR_DISP: OnceCell<Mutex<String>> = OnceCell::new();

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub enum AudioAsset {
    BKGND_WAIT = 0,
    BKGND_GAME = 1,
    BKGND_WINNER = 2,
    BKGND_LOSER = 3,
    WIND = 4,
    STEP = 5,
    RAIN = 6,
    JUMP = 7,
    LAND = 8,
    SPAWN_BEEP = 9,
    DIE = 10,
    SPAWN = 11,
    WIND_WEATHER = 12,
    PICKUP = 13,
    ICE = 14,
    FLASH = 15,
    DASH = 16,
    POWERUP = 17,
    REFILL = 18,
    POINTS_GAIN = 19,
    WEATHER_ENV = 20,
}

pub struct SoundInstance {
    controller: ambisonic::SoundController,
    position: glm::Vec3,
    start: SystemTime,
    initial_dir: glm::Vec3,
    at_client: bool,
    ambient: bool,
    client: u32,
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
    sound_controllers_ambient: HashMap<AudioAsset, SoundInstance>, // for sound events that should only ever be played once at a time e.g. weather
    fading_out: HashMap<AudioAsset, SoundInstance>,
    sound_controller_background: (Option<ambisonic::SoundController>, bool),
    time: SystemTime,
    sfx_queue: Arc<Mutex<SoundQueue>>,
    curr_state: GameLifeCycleState,
}

impl Audio {
    pub fn new(q: Arc<Mutex<SoundQueue>>) -> Self {
        CURR_DISP
            .set(Mutex::new("display:title".to_string()))
            .expect("failed to initialze CURR_DISP");
        Self {
            audio_scene: AmbisonicBuilder::default().build(),
            audio_assets: Vec::new(),
            sound_controllers_fx: HashMap::new(),
            sound_controllers_ambient: HashMap::new(),
            fading_out: HashMap::new(),
            sound_controller_background: (None, true),
            time: SystemTime::now(),
            sfx_queue: q,
            curr_state: GameLifeCycleState::Waiting,
        }
    }

    pub fn reset_audio(&mut self){
        for (_,v) in self.sound_controllers_fx.iter_mut(){
            for i in v.iter_mut() {
                i.controller.stop();
            }
        }

        for (_,v) in self.sound_controllers_ambient.iter_mut(){
            v.controller.stop();
        }

        for (_,v) in self.fading_out.iter_mut(){
            v.controller.stop();
        }

        self.sound_controllers_fx.clear();
        self.sound_controllers_ambient.clear();
        self.fading_out.clear();
    }

    pub fn play_background_track(&mut self, bkgd: AudioAsset, pos: [f32; 3]) {
        let source = self.audio_assets[bkgd as usize]
            .0
            .clone()
            .fade_in(Duration::new(0, 250000000))
            .repeat_infinite();
        let sound = self.audio_scene.play_at(source.convert_samples(), pos);
        self.sound_controller_background = (Some(sound), false);
    }

    pub fn update_bkgd_track(&mut self, state: GameLifeCycleState, curr_player: u32, winner: u32) {
        if std::mem::discriminant(&self.curr_state) != std::mem::discriminant(&state) {
            // println!("audio registered state change: {:?}", self.curr_state);
            match state {
                // title, lobby background track
                GameLifeCycleState::Waiting => {
                    if CURR_DISP.get().unwrap().lock().unwrap().clone()
                        == "display:title".to_string()
                    {
                        self.switch_background_track(AudioAsset::BKGND_WAIT, AUDIO_POS_AT_CLIENT);
                        self.curr_state = state;
                    }
                }
                // in game background track
                GameLifeCycleState::Running(_) => {
                    self.switch_background_track(AudioAsset::BKGND_GAME, AUDIO_POS_AT_CLIENT);
                    self.curr_state = state;
                }

                // winner, loser background track
                GameLifeCycleState::Ended => {
                    if curr_player == winner {
                        self.switch_background_track(AudioAsset::BKGND_WINNER, AUDIO_POS_AT_CLIENT);
                    } else {
                        self.switch_background_track(AudioAsset::BKGND_LOSER, AUDIO_POS_AT_CLIENT);
                    }
                    self.curr_state = state;
                }
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

    pub fn handle_fade_out(&mut self){
        let percent = 0.75;
        let mut to_remove = Vec::new();

        for (k,v) in self.fading_out.iter_mut() {
            //println!("{}, {}", v.position, FADE_DIST*v.initial_dir);
            if glm::magnitude(&v.position) > glm::magnitude(&(FADE_DIST * v.initial_dir)) {
                v.controller.stop();
                to_remove.push(k.clone());
                continue;
            }
            let mut offset = v.initial_dir;
            if offset != glm::Vec3::new(0.0, 0.0, 0.0) {
                offset = glm::normalize(&offset);
            }
            offset = glm::Vec3::new(
                offset.x * percent,
                offset.y * percent,
                offset.z * percent,
            );
            v.position += offset;
            thread::sleep(Duration::from_millis(1));

            v.controller.adjust_position([v.position.x, v.position.z, 0.0]);
        }

        for r in to_remove.iter(){
            self.fading_out.remove(r);
        }
    }

    pub fn loop_sound(&mut self, sound: AudioAsset, pos: [f32; 3]) -> ambisonic::SoundController {
        let source = self.audio_assets[sound as usize]
            .0
            .clone()
            .fade_in(Duration::new(1, 0))
            .repeat_infinite();            
        let sc = self.audio_scene.play_at(source.convert_samples(), pos);
        sc
    }

    pub fn handle_ambient_event(&mut self, sfxevent: SoundSpec, player_pos: glm::Vec3, player_dir: glm::Vec3) {
        let index = to_audio_asset(sfxevent.sound_id.clone()).unwrap();
        let (_, play_sound, fade_out) = sfxevent.ambient;
        let instance = self.sound_controllers_ambient.get_mut(&index);

        if play_sound {
            match instance {
                None => {
                    let sound = self.loop_sound(index, AUDIO_POS_AT_CLIENT); // [0.0,1.0,0.0]);
                    let mut pos = glm::Vec3::new(AUDIO_POS_AT_CLIENT[0], AUDIO_POS_AT_CLIENT[2], AUDIO_POS_AT_CLIENT[1]);
                    let mut dir = glm::Vec3::new(0.0, 0.0, 1.0);
                    // if index == AudioAsset::WIND_WEATHER {
                    //     dir = glm::normalize(&sfxevent.direction);
                    //     pos = get_weather_start_position(dir.clone());
                    // }
                    let si = SoundInstance{
                        controller: sound,
                        position: pos,
                        start: SystemTime::now(),
                        initial_dir: dir,
                        at_client: false,
                        ambient: true,
                        client: sfxevent.at_client.0,
                    };
                    self.sound_controllers_ambient.insert(index.clone(), si);

                    // if index == AudioAsset::WIND_WEATHER {
                    //     let sound = self.loop_sound(index, AUDIO_POS_AT_CLIENT); // [0.0,1.0,0.0]);
                    //     let mut pos = glm::Vec3::new(AUDIO_POS_AT_CLIENT[0], AUDIO_POS_AT_CLIENT[2], AUDIO_POS_AT_CLIENT[1]);
                    //     let mut dir = glm::Vec3::new(0.0, 0.0, 1.0);

                    //     let si = SoundInstance{
                    //         controller: sound,
                    //         position: pos,
                    //         start: SystemTime::now(),
                    //         initial_dir: dir,
                    //         at_client: false,
                    //         ambient: true,
                    //         client: sfxevent.at_client.0,
                    //     };
                    //     self.sound_controllers_ambient.insert(AudioAsset::WEATHER_ENV, si);
                    // }
                }
                Some(_s) => {
                    // if index == AudioAsset::WIND_WEATHER {
                    //     let p = self.audio_assets[index as usize].2;
                    //     update_weather_position(s, p, player_pos, player_dir);
                    // }
                }
            }
        }
        else {
            let si = self.sound_controllers_ambient.remove(&index);
            if let Some(s) = si {
                if !fade_out {
                    s.controller.stop();
                }
                else {
                    self.fading_out.insert(index.clone(), s);
                    // if index == AudioAsset::WIND_WEATHER {
                    //     let si1 = self.sound_controllers_ambient.remove(&AudioAsset::WEATHER_ENV).unwrap();
                    //     self.fading_out.insert(AudioAsset::WEATHER_ENV, si1);
                    // }
                }
            }
        }
    }

    pub fn update_sound_positions(&mut self, player_pos: glm::Vec3, dir: glm::Vec3, client_id: u32) {
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
                        .adjust_position([0.0, 15.0, 0.0]); // 10.0
                } else {
                    if true { // sound_instances[i].client == client_id {
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
                    if glm::magnitude(&pos) < SOUND_RADIUS { // !basically_zero(pos) {
                        sound_instances[i]
                            .controller
                            .adjust_position([pos.x, pos.z, 0.0]);
                        //println!("POS: {}, {}", pos.x, pos.z);

                    } 
                    else {
                        sound_instances[i]
                            .controller
                            .adjust_position([10000.0, 10000.0, 10000.0]); // TODO: f32::powf(pos.x, 1.5), f32::powf(pos.x, 1.5), 0.0
                    }
                    // else {
                    //     sound_instances[i]
                    //         .controller
                    //         .adjust_position([0.0, 2.5, 0.0]);
                    // }
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
                0.0, // sfxevent.position.y,
            ],
        ); // double check y,z should be switched

        let sound_vec = self.sound_controllers_fx.get_mut(&index);
        match sound_vec {
            Some(vec) => {
                vec.push(SoundInstance {
                    controller: sound,
                    position: sfxevent.position,
                    start: SystemTime::now(),
                    initial_dir: glm::Vec3::new(sfxevent.direction.x, 0.0, sfxevent.direction.z),
                    at_client,
                    ambient: false,
                    client:sfxevent.at_client.0,
                });
            }
            None => {
                self.sound_controllers_fx.insert(
                    index,
                    vec![SoundInstance {
                        controller: sound,
                        position: sfxevent.position,
                        start: SystemTime::now(),
                        initial_dir: glm::Vec3::new(sfxevent.direction.x, 0.0, sfxevent.direction.z),
                        at_client,
                        ambient: false,
                        client: sfxevent.at_client.0,
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
            self.handle_fade_out();

            match player_curr {
                Ok(player) => {
                    cf = player_curr.unwrap().camera_forward;
                    pos = player.transform.translation;
                }
                _ => {}
            }

            let mut sfx_queue = self.sfx_queue.lock().unwrap().clone();
            match &gs.life_cycle_state {
                GameLifeCycleState::Running(_)  => {
                    if !sfx_queue.sound_queue.is_empty() {
                        for i in 0..sfx_queue.sound_queue.len() {
                            let se = sfx_queue.sound_queue[i].clone();
                            // println!("SOUND: {}, {:?}", se.sound_id, se.ambient);
                            let at_client = se.at_client.0 == client_id as u32 && se.at_client.1;
                            let ambient = se.ambient.0;
                            if !ambient {
                                self.handle_sfx_event(se, cf, at_client);
                            } else {
                                self.handle_ambient_event(se, pos, cf);
                            }
                        }
                        sfx_queue.sound_queue.clear();
                        *self.sfx_queue.lock().unwrap() = sfx_queue;
                    }

                    self.update_sound_positions(pos, cf, client_id as u32);
                }
                _ =>  {
                    sfx_queue.sound_queue.clear();
                    self.reset_audio();
                }
            }
        }
    }
}

pub fn to_audio_asset(sound_id: String) -> Option<AudioAsset> {
    match sound_id.as_str() {
        "wind" => Some(AudioAsset::WIND),
        "foot_step" => Some(AudioAsset::STEP),
        "rain" => Some(AudioAsset::RAIN),
        "jump" => Some(AudioAsset::JUMP),
        "land" => Some(AudioAsset::LAND),
        "spawn_beep" => Some(AudioAsset::SPAWN_BEEP),
        "die" => Some(AudioAsset::DIE),
        "spawn" => Some(AudioAsset::SPAWN),
        "wind_weather" => Some(AudioAsset::WIND_WEATHER),
        "pickup" => Some(AudioAsset::PICKUP),
        "ice" => Some(AudioAsset::ICE),
        "flash" => Some(AudioAsset::FLASH),
        "dash" => Some(AudioAsset::DASH),
        "powerup" => Some(AudioAsset::POWERUP),
        "refill" => Some(AudioAsset::REFILL),
        "points_gain" => Some(AudioAsset::POINTS_GAIN),
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

    // let r = glm::magnitude(&rel_pos)*RADIUS_MUL + RADIUS_OFFSET;
    let mut r = glm::magnitude(&rel_pos)*RADIUS_MUL + RADIUS_OFFSET; 
    if r < 3.0 {r = 3.0; }
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

pub fn get_weather_start_position(dir: glm::Vec3) -> glm::Vec3 {
    let x = dir * -20.0;
    glm::Vec3::new(x.x, 0.0, x.z)
}

pub fn update_weather_position(s: &mut SoundInstance, percent: f32, player_pos: glm::Vec3, player_dir: glm::Vec3) {
    if glm::magnitude(&s.position) > 20.0 {
        s.position = get_weather_start_position(s.initial_dir);
    }
    else {
        let mut offset = s.initial_dir;
        if offset != glm::Vec3::new(0.0, 0.0, 0.0) {
            offset = glm::normalize(&offset);
        }
        offset = glm::Vec3::new(
            offset.x * percent,
            offset.y * percent,
            offset.z * percent,
        );
        s.position += offset;
    }
    let pos = relative_position(s.position, glm::Vec3::new(0.0,0.0,0.0), player_dir);
    s.controller.adjust_position([pos.x, pos.z+1.0, 0.0]);
    thread::sleep(Duration::from_millis(10));
}

// audio assets from config file
impl Audio {
    pub fn from_config(json: &ConfigAudioAssets, sound_queue: Arc<Mutex<SoundQueue>>) -> Self {
        let mut audio = Self::new(sound_queue);

        for sound in &json.sounds {
            let file = BufReader::new(File::open(sound.path.clone()).unwrap());
            // println!("{}", sound.path.clone());
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
