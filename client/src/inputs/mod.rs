use crate::inputs::handlers::{handle_camera_update, handle_game_key_input, GameKeyKind};
use common::communication::commons::Protocol;
use common::core::command::Command;
use common::core::command::Command::{Attack, Die, Jump, Refill, Spawn};
use common::core::states::CameraInfo;
use glm::{vec3, Vec3};
use log::debug;
use nalgebra_glm as glm;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

pub mod handlers;

#[derive(Debug)]
pub enum Input {
    Keyboard(KeyboardInput),
    Mouse(DeviceEvent),
    Camera{info: CameraInfo},
}

#[derive(Debug, Clone)]
pub enum ButtonState {
    Pressed,
    Held,
    Released,
}

/// Input polling interval
pub const POLLER_INTERVAL: Duration = Duration::from_millis(60);

pub struct InputEventProcessor {
    protocol: Protocol,
    client_id: u8,
    rx: Receiver<Input>,
    button_states: Arc<Mutex<HashMap<VirtualKeyCode, ButtonState>>>,
    /*
    camera_forward: Arc<Mutex<Vec3>>,
    camera_prelim_position: Arc<Mutex<Vec3>>,
    */ 
    camera_info: Arc<Mutex<CameraInfo>>, 
    poller_signal: Arc<(Mutex<bool>, Condvar)>,
}

impl InputEventProcessor {
    pub fn new(protocol: Protocol, client_id: u8, rx: Receiver<Input>) -> Self {
        InputEventProcessor {
            protocol,
            client_id,
            rx,
            button_states: Arc::new(Mutex::new(HashMap::new())),
            camera_info: Arc::new(Mutex::new(Default::default())),
            /*
            camera_forward: Arc::new(Mutex::new(Default::default())),
            camera_prelim_position: Arc::new(Mutex::new(Default::default())),
            */ 
            poller_signal: Arc::new((Mutex::new(true), Condvar::new())),
        }
    }

    // TODO: make this more maintainable
    pub fn map_key(virtual_keycode: VirtualKeyCode) -> Option<(GameKeyKind, Command)> {
        match virtual_keycode {
            // match Holdable keys
            VirtualKeyCode::W => Some((GameKeyKind::Holdable, Command::Move(vec3(0., 0., 1.)))),
            VirtualKeyCode::A => Some((GameKeyKind::Holdable, Command::Move(vec3(1., 0., 0.)))),
            VirtualKeyCode::S => Some((GameKeyKind::Holdable, Command::Move(vec3(0., 0., -1.)))),
            VirtualKeyCode::D => Some((GameKeyKind::Holdable, Command::Move(vec3(-1., 0., 0.)))),
            VirtualKeyCode::R => Some((GameKeyKind::Holdable, Refill)),
            // match Pressable keys
            VirtualKeyCode::Space => Some((GameKeyKind::Pressable, Jump)),
            VirtualKeyCode::LShift => Some((GameKeyKind::Pressable, Spawn)),
            VirtualKeyCode::RShift => Some((GameKeyKind::Pressable, Die)),
            // match PressRelease keys
            // VirtualKeyCode::LShift => Some((GameKeyKind::PressRelease, Spawn)),
            VirtualKeyCode::F => Some((GameKeyKind::PressRelease, Attack)),
            _ => None,
        }
    }

    pub fn start_poller(&self) {
        let mut protocol = self.protocol.try_clone().unwrap();
        let client_id = self.client_id;
        let button_states = Arc::clone(&self.button_states);
        let camera_info = Arc::clone(&self.camera_info); 
        let poller_signal = Arc::clone(&self.poller_signal);

        thread::spawn(move || {
            let (lock, cvar) = &*poller_signal;
            loop {
                // wait for signal (asap event) or timeout
                let signal = lock.lock().unwrap();
                let (mut signal, res) = cvar.wait_timeout(signal, POLLER_INTERVAL).unwrap();

                if res.timed_out() {
                    debug!("poller timed out");
                    *signal = false;
                } else {
                    debug!("poller received signal");
                }

                let mut button_states = button_states.lock().unwrap();
                let camera_info = camera_info.lock().unwrap(); 

                button_states.retain(|key, state| {
                    if let Some((key_type, command)) = Self::map_key(*key) {
                        let retain = handle_game_key_input(
                            key_type,
                            command,
                            state,
                            &mut protocol,
                            client_id,
                        );

                        // naturally progress the button state
                        *state = Self::internal_next_state(state.clone());

                        retain
                    } else {
                        false
                    }
                });

                // send camera update
                // TODO: send camera update only when the camera has moved
                handle_camera_update(*camera_info, &mut protocol, client_id);
            }
        });
    }

    /// listen to input events from the event loop and update the states when necessary
    pub fn listen(&mut self) {
        while let Ok(input) = self.rx.recv() {
            match input {
                Input::Keyboard(KeyboardInput {
                    virtual_keycode: Some(key_code),
                    state,
                    ..
                }) => {
                    // on receiving keyboard input, update the button state
                    self.update_button_state(key_code, state);
                    debug!(
                        "processed_keyboard_input: {:?}, button_states: {:?}",
                        key_code,
                        self.button_states.lock().unwrap()
                    );

                    // Signal the poller to send data as soon as possible
                    // Should be only for "Pressable" keys since otherwise the sampling rate will be inconsistent
                    // This optimization will be significant if we decide to use a longer polling interval (e.g. > 100ms) to save bandwidth
                    if let Some((GameKeyKind::Pressable, _)) = Self::map_key(key_code) {
                        let (lock, cvar) = &*self.poller_signal;
                        let mut signal = lock.lock().unwrap();
                        *signal = true;
                        cvar.notify_one(); // notify the poller to send data immediately
                    }
                }

                // receive camera update
                Input::Camera {info} => {
                    let mut camera_info = self.camera_info.lock().unwrap(); 
                    *camera_info = info;
                }
                _ => {}
            }
        }
    }

    /// updates the button state based on the input state (`ele_state`)
    fn next_state(es: ElementState, bs: ButtonState) -> ButtonState {
        use winit::event::ElementState as es;
        use ButtonState as bs;

        match (bs, es) {
            (bs::Pressed, es::Pressed) => ButtonState::Held, // pressed -> (pressed) -> held
            (bs::Released, es::Pressed) => ButtonState::Pressed, // released -> (pressed) -> pressed
            (_, es::Released) => ButtonState::Released,      // some -> (released) -> released
            (bs, _) => bs,                                   // same state
        }
    }

    /// models the natural progression of button states between sample ticks when there's no input
    fn internal_next_state(bs: ButtonState) -> ButtonState {
        use ButtonState as bs;

        match bs {
            bs::Pressed => bs::Held,
            other => other,
        }
    }

    pub fn update_button_state(&mut self, keycode: VirtualKeyCode, ele_state: ElementState) {
        let mut button_states = self.button_states.lock().unwrap();

        let next_state = if let Some(state) = button_states.get(&keycode) {
            Self::next_state(ele_state, state.clone())
        } else {
            ButtonState::Pressed
        };
        button_states.insert(keycode, next_state);
    }
}
