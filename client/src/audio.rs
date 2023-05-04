use std::{io::BufReader, fs::File, sync::{Mutex, Arc}};
use ambisonic::{rodio, AmbisonicBuilder};
use instant::{SystemTime, Duration};
use std::collections::HashMap;
use bus::BusReader;
use common::core::{events::{SoundSpec, GameEvent}, states::GameState};

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
    adjust_pos: bool,
}

pub struct Audio {
    audio_scene: ambisonic::Ambisonic,
    audio_assets: Vec<(String, Duration)>, // find a better way later
    sound_controllers_fx: HashMap<AudioAsset, Vec<SoundInstance>>,
    sound_controller_background: (Option<ambisonic::SoundController>, bool),
    time: SystemTime, // hacky fix for now
    pub sfx_queue: Arc<Mutex<Vec<SoundSpec>>>,
}

impl Audio {
    pub fn new() -> Self{        
        Audio {
            audio_scene: AmbisonicBuilder::default().build(),
            audio_assets: vec![("client/res/royalty-free-sample.mp3".to_string(), Duration::new(0,0)), // duration of background track doesn't matter
                               ("client/res/woosh_sound.mp3".to_string(), Duration::new(5, 2500)),
                               ("client/res/step_sound.mp3".to_string(), Duration::new(1, 2500)),],
            sound_controllers_fx: HashMap::new(),
            sound_controller_background: (None, true),
            time: SystemTime::now(),
            sfx_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn play_background_track(&mut self, track: AudioAsset){
        let file = File::open(self.audio_assets[track as usize].0.clone()).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        let source = rodio::Source::repeat_infinite(source);
        let mut sound = self.audio_scene.play_at(rodio::Source::convert_samples(source), [0.0, 10.0, 0.0]);
        // sound.set_velocity([10.0,0.0,0.0]);
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
        // TODO: add velocity
        // TODO: add 3d audio in the up and down direction as well (not super important bc jumps aren't that high)
        for (asset, sound_instances) in self.sound_controllers_fx.iter_mut() {
            let sound_duration = self.audio_assets[*asset as usize].1;
            let mut to_remove = Vec::new();
            for i in 0..sound_instances.len() {
                if sound_instances[i].start.elapsed().unwrap_or(Duration::new(1,0)) >= sound_duration {
                    to_remove.push(i);
                }
                else if !sound_instances[i].adjust_pos {
                    continue;
                }
                else{
                    let pos = relative_position(sound_instances[i].position, player_pos, dir);
                    if pos != glm::Vec3::new(0.0,0.0,0.0) {
                        sound_instances[i].controller.adjust_position([pos.x, pos.z, 0.0]);
                    }
                    else {
                        sound_instances[i].controller.adjust_position([pos.x, pos.z+0.01, 0.0]);
                    }
                }
            }
            for i in 0..to_remove.len() {
                sound_instances.remove(to_remove[i]);
            }
        }
    }

    pub fn handle_sfx_event(&mut self, mut sfxevent: SoundSpec){
        let (index, adjust_position) = to_audio_asset(sfxevent.sound_id).unwrap();

        // adjust position false means the sound should be played at the client
        if !adjust_position {
            sfxevent.position = glm::Vec3::new(0.0,1.0,0.0);
        }

        let file = File::open(self.audio_assets[index as usize].0.clone()).unwrap();
        let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
        let sound = self.audio_scene.play_at(rodio::Source::convert_samples(source), [sfxevent.position.x, sfxevent.position.z, sfxevent.position.y]); // double check y,z should be switched

        let sound_vec = self.sound_controllers_fx.get_mut(&index);
        match sound_vec {
            Some(vec) => {
                vec.push(SoundInstance {
                    controller: sound,
                    position: sfxevent.position,
                    start: SystemTime::now(),
                    adjust_pos: adjust_position,
                });
            }
            None => {
                self.sound_controllers_fx.insert(
                    index,
                    vec![SoundInstance {
                        controller: sound,
                        position: sfxevent.position,
                        start: SystemTime::now(),
                        adjust_pos: adjust_position,
                    }],
                );
            }
        }    
    }           

    pub fn handle_audio_updates(&mut self, game_state: Arc<Mutex<GameState>>, client_id: u8){ //, mut game_event_receiver: BusReader<GameEvent>){
        loop {
            // match game_event_receiver.try_recv() {
            //     Ok(game_event) => {
            //         match game_event {
            //             GameEvent::SoundEvent(sound_event) => {
            //                 println!("SOUND EVENT FROM BROADCAST: {:?}", sound_event);
            //                 self.handle_sfx_event(sound_event);
            //             }
            //             _ => {}
            //         }
            //     }
            //     Err(_) => {},
            // }
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

pub fn to_audio_asset(sound_id: String) -> Option<(AudioAsset,bool)> {
    match sound_id.as_str() {
        "wind" => Some((AudioAsset::WIND, true)),
        "foot_step" => Some((AudioAsset::STEP, false)),
        _ => None
    }
}

pub fn relative_position(sound_position: glm::Vec3, player_pos: glm::Vec3, dir: glm::Vec3) -> glm::Vec3{
    let rel_pos = sound_position - player_pos;
    let new_pos = glm::Vec3::new(rel_pos.x, 0.0, rel_pos.z);

    let new_dir = glm::Vec3::new(dir.x, 0.0, dir.z);
    let up = glm::Vec3::new(0.0, 1.0, 0.0);

    let right_dir = glm::cross(&new_dir, &up);

    let dot = right_dir.x*new_pos.x + right_dir.z*new_pos.z;
    let det = right_dir.x*new_pos.z - right_dir.z*new_pos.x;
    let angle = glm::atan2(&glm::Vec1::new(det), &glm::Vec1::new(dot)).x; // theta

    let side = glm::Vec3::new(1.0, 0.0, 0.0);
    let new_pos_2 = glm::Vec3::new(rel_pos.x, rel_pos.y, 0.0);

    let dot2 = side.x*new_pos_2.x + side.y*new_pos_2.y;
    let det2 = side.x*new_pos_2.y - side.y*new_pos_2.x;
    let angle2 = glm::atan2(&glm::Vec1::new(det2), &glm::Vec1::new(dot2)).x; // phi


    // let mut angle = glm::angle(&right_dir,&new_pos);
    // let matrix = glm::mat3(right_dir.x, new_pos.x, 0.0, 
    //                        right_dir.y, new_pos.y, 0.0, 
    //                        right_dir.z, new_pos.z, 1.0);
    // let det = glm::determinant(&matrix);
    let r = glm::magnitude(&rel_pos);
    // if r <= 10.0 {
        // if det > 0.0 {
        //     angle = angle + glm::pi::<f32>();
        // }
        let x = r * glm::cos(&glm::Vec1::new(-angle)).x; // * glm::sin(&glm::Vec1::new(-angle2)).x;
        let z = r * glm::sin(&glm::Vec1::new(-angle)).x; // * glm::sin(&glm::Vec1::new(-angle2)).x;
        // let y = r * glm::cos(&glm::Vec1::new(-angle2)).x;
        // println!("Position: {}", new_pos);
        // println!("x: {}, and z: {}", x, z);
        // println!("Determinant: {}", det);
        // println!("Angle: {}", glm::degrees(&glm::Vec1::new(angle)));
        glm::Vec3::new(x, 0.0,z) 
    // }
    // else{ glm::Vec3::new(10000000.0, 0.0,0.0) }
}