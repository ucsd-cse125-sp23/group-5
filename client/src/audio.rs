use std::{io::BufReader, fs::File};
use ambisonic::{rodio, AmbisonicBuilder};
use instant::{SystemTime, Duration};
use std::collections::HashMap;

pub struct SoundSpec{
    pub position: glm::Vec3,
    pub sound_id: String,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum AudioAsset {
    BACKGROUND = 0,
    WIND = 1,
}

pub struct SoundInstance{
    controller: ambisonic::SoundController,
    position: glm::Vec3,
    start: SystemTime,
}

pub struct Audio {
    audio_scene: ambisonic::Ambisonic,
    audio_assets: Vec<(String, Duration)>, // find a better way later
    sound_controllers_fx: HashMap<AudioAsset, Vec<SoundInstance>>,
    sound_controller_background: (Option<ambisonic::SoundController>, bool),
    time: SystemTime, // hacky fix for now
}

impl Audio {
    pub fn new() -> Self{        
        Audio {
            audio_scene: AmbisonicBuilder::default().build(),
            audio_assets: vec![("client/res/royalty-free-sample.mp3".to_string(), Duration::new(0,0)), // duration of background track doesn't matter
                               ("client/res/woosh_sound.mp3".to_string(), Duration::new(5, 2500)),],
            sound_controllers_fx: HashMap::new(),
            sound_controller_background: (None, true),
            time: SystemTime::now(),
        }
    }

    pub fn play_background_track(&mut self, track: AudioAsset){
        let file = File::open(self.audio_assets[track as usize].0.clone()).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        let source = rodio::Source::repeat_infinite(source);
        let mut sound = self.audio_scene.play_at(rodio::Source::convert_samples(source), [0.0, 50.0, 0.0]);
        // sound.set_velocity([10.0,0.0,0.0]);
        self.sound_controller_background = (Some(sound), false);
    }

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
                let pos = relative_position(sound_instances[i].position, player_pos, dir);
                sound_instances[i].controller.adjust_position([pos.x, pos.z, 0.0]);

            }
            for i in 0..to_remove.len() {
                sound_instances.remove(to_remove[i]);
            }
        }
    }

    pub fn handle_sfx_event(&mut self, sfxevent: SoundSpec){
        let index = to_audio_asset(sfxevent.sound_id).unwrap();

        let file = File::open(self.audio_assets[index as usize].0.clone()).unwrap();
        let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
        let sound = self.audio_scene.play_at(rodio::Source::convert_samples(source), [sfxevent.position.x, sfxevent.position.y, sfxevent.position.z]);

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
}

pub fn to_audio_asset(sound_id: String) -> Option<AudioAsset> {
    match sound_id.as_str() {
        "wind" => Some(AudioAsset::WIND),
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
    let angle = glm::atan2(&glm::Vec1::new(det), &glm::Vec1::new(dot)).x;
    // let mut angle = glm::angle(&right_dir,&new_pos);
    // let matrix = glm::mat3(right_dir.x, new_pos.x, 0.0, 
    //                        right_dir.y, new_pos.y, 0.0, 
    //                        right_dir.z, new_pos.z, 1.0);
    // let det = glm::determinant(&matrix);
    let r = glm::magnitude(&new_pos);
    // if r <= 10.0 {
        // if det > 0.0 {
        //     angle = angle + glm::pi::<f32>();
        // }
        let x = r * glm::cos(&glm::Vec1::new(-angle)).x;
        let z = r * glm::sin(&glm::Vec1::new(-angle)).x;
        // println!("Position: {}", new_pos);
        // println!("x: {}, and z: {}", x, z);
        // println!("Determinant: {}", det);
        // println!("Angle: {}", glm::degrees(&glm::Vec1::new(angle)));
        glm::Vec3::new(x, 0.0,z) 
    // }
    // else{ glm::Vec3::new(10000000.0, 0.0,0.0) }
}