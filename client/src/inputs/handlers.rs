use crate::inputs::{ButtonState};
use common::communication::commons::*;
use common::communication::message::*;
use common::core::command::{Command, MoveDirection};
use log::{error, info};
use queues::{IsQueue, Queue};
use std::time::Instant;
use std::collections::HashMap;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};
use common::core::command::Command::Spawn;
use crate::inputs::ButtonState::Pressed;

pub enum GameKey {
    Pressable(PressableKey),
    Heldable(HeldableKey),
    PressRelease(PressReleaseKey),
}

pub enum PressableKey {
    ESC,
    SPACE,
}

pub enum HeldableKey {
    W,
    A,
    S,
    D,
}

pub enum PressReleaseKey {
    LeftClick,
}


pub fn handle_keyboard_input(
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    input: KeyboardInput, protocol: &mut Protocol, client_id: u8)
{
    // change state
    //let mut functional_key= Some(..);
    if let Some(keycode) = input.virtual_keycode{
        update_held_map(held_map, keycode, input.state);
    }

    // map keyboard input to command
    let game_key: Option<(GameKey, Command)> = match input.virtual_keycode {
        // match the keys
        // match heldable keys
        Some(VirtualKeyCode::W) => Some((GameKey::Heldable(HeldableKey::W), Command::Move(MoveDirection::Forward))),
        Some(VirtualKeyCode::A) => Some((GameKey::Heldable(HeldableKey::A), Command::Move(MoveDirection::Left))),
        Some(VirtualKeyCode::S) => Some((GameKey::Heldable(HeldableKey::S), Command::Move(MoveDirection::Backward))),
        Some(VirtualKeyCode::D) => Some((GameKey::Heldable(HeldableKey::D), Command::Move(MoveDirection::Right))),
        // match Pressable keys
        Some(VirtualKeyCode::Space) => Some((GameKey::Pressable(PressableKey::SPACE),Command::Spawn)),
        // match PressRelease keys

        _ => None,
    };


    if let Some(game_key) = game_key {
        handle_game_key_input(game_key, input, held_map, protocol, client_id);

    } else {
        info!("No game key to handle");
    }

    // info!("Received game key: {:?}", game_key);
    // if let Some(command) = command {
    //     let message: Message = Message::new(
    //         HostRole::Client(client_id),
    //         Payload::Command(command.clone()),
    //     );
    //     protocol.send_message(&message).expect("send message fails");
    //     info!("Sent command: {:?}", command);
    // } else {
    //     info!("No command to send");
    // }
}

pub fn update_held_map(held_map: &mut HashMap<winit::event::VirtualKeyCode, ButtonState>,
                       keycode : VirtualKeyCode, ele_state : ElementState) {
    if held_map.contains_key(&keycode) {
        match held_map.get(&keycode) {
            Some(ButtonState::Pressed) => {
                if ele_state == ElementState::Pressed {
                    held_map.insert(keycode, ButtonState::Held);
                }
                else {
                    held_map.insert(keycode, ButtonState::Released);
                }
            }
            Some(ButtonState::Held) => {
                if ele_state == ElementState::Released {
                    held_map.insert(keycode, ButtonState::Released);
                }
            }
            Some(ButtonState::Released) => {
                if ele_state == ElementState::Pressed {
                    held_map.insert(keycode, ButtonState::Pressed);
                }
            }
            None => {}
        }
    }
    else {
        held_map.insert(keycode, ButtonState::Pressed);
    }
}

// mw for mouse wheel, mm for mouse motion
pub fn handle_mouse_input(
    input: DeviceEvent,
    protocol: &mut Protocol,
    mm_buffer: &mut Queue<DeviceEvent>,
    mw_buffer: &mut Queue<DeviceEvent>,
) {
    match input {
        DeviceEvent::MouseMotion { .. } => {
            mm_buffer.add(input).expect("adding to mm_buffer failed \n");
        }
        DeviceEvent::MouseWheel { .. } => {
            mw_buffer.add(input).expect("adding to mw_buffer failed \n");
        }
        // what's that possibly for?
        DeviceEvent::Button { .. } => {
            // if we receive those button events, then should send right away with protocol
        }
        _ => {}
    }
}

pub fn send_mouse_input(
    mouse_motion_buf: &mut Queue<DeviceEvent>,
    mouse_wheel_buf: &mut Queue<DeviceEvent>,
    sample_start_time: &mut Instant,
    protocol: &mut Protocol,
    client_id: u8,
) {
    let mut mm_tot_dx = 0.0;
    let mut mm_tot_dy = 0.0;
    let mut mw_tot_line_dx = 0.0;
    let mut mw_tot_line_dy = 0.0;
    let mut mw_tot_pixel_dx = 0.0;
    let mut mw_tot_pixel_dy = 0.0;

    let mm_n = mouse_motion_buf.size();
    for _ in 1..mm_n {
        let mm_event = mouse_motion_buf.remove().unwrap();
        match mm_event {
            DeviceEvent::MouseMotion { delta } => {
                let (dx, dy) = delta;
                mm_tot_dx += dx;
                mm_tot_dy += dy;
            }
            _ => {
                error!("non-mouse-motion in mouse motion buffer \n")
            }
        }
    }

    let mw_n = mouse_wheel_buf.size();
    for _ in 1..mw_n {
        let mw_event = mouse_wheel_buf.remove().unwrap();
        if let DeviceEvent::MouseWheel { delta } = mw_event {
            match delta {
                MouseScrollDelta::LineDelta(dx, dy) => {
                    mw_tot_line_dx += dx;
                    mw_tot_line_dy += dy;
                }
                MouseScrollDelta::PixelDelta(pixel_delta) => {
                    mw_tot_pixel_dx += pixel_delta.x;
                    mw_tot_pixel_dy += pixel_delta.y;
                }
            }
        }
    }

    Instant::now().clone_into(sample_start_time);

    protocol
        .send_message(&Message::new(
            HostRole::Client(client_id),
            Payload::Command(Command::Turn(Default::default())),
        ))
        .expect("send message fails");
    info!("Sent command: {:?}", "Turn");
}