use crate::inputs::{ButtonState};
use common::communication::commons::*;
use common::communication::message::*;
use common::core::command::{Command, MoveDirection};
use log::{error, info};
use queues::{IsQueue, Queue};
use std::time::Instant;
use std::collections::HashMap;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};
use common::core::command::Command::{Action, Spawn};
use common::core::command::GameAction::Attack;

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
    F,
}


pub fn handle_keyboard_input(
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    input: KeyboardInput, protocol: &mut Protocol, client_id: u8,
) {
    // change state
    //let mut functional_key= Some(..);
    if let Some(keycode) = input.virtual_keycode {
        update_held_map(held_map, keycode, input.state);
    }

    // map keyboard input to command
    let key_command: Option<(GameKey, Command)> = match input.virtual_keycode {
        // match heldable keys
        Some(VirtualKeyCode::W) => Some((GameKey::Heldable(HeldableKey::W), Command::Move(MoveDirection::Forward))),
        Some(VirtualKeyCode::A) => Some((GameKey::Heldable(HeldableKey::A), Command::Move(MoveDirection::Left))),
        Some(VirtualKeyCode::S) => Some((GameKey::Heldable(HeldableKey::S), Command::Move(MoveDirection::Backward))),
        Some(VirtualKeyCode::D) => Some((GameKey::Heldable(HeldableKey::D), Command::Move(MoveDirection::Right))),
        // match Pressable keys
        Some(VirtualKeyCode::Space) => Some((GameKey::Pressable(PressableKey::SPACE), Spawn)),
        // match PressRelease keys
        Some(VirtualKeyCode::F) => Some(((GameKey::PressRelease(PressReleaseKey::F)), Action(Attack))),
        _ => None,
    };


    if let Some((game_key, command)) = key_command {
        handle_game_key_input(game_key, command, held_map, protocol, client_id);
    } else {
        info!("No game key or action to handle");
    }
}

pub fn update_held_map(
    held_map: &mut HashMap<winit::event::VirtualKeyCode, ButtonState>,
    keycode: VirtualKeyCode, ele_state: ElementState,
) {
    use winit::event::ElementState as es;
    use ButtonState as bs;
    let next_state = if let Some(state) = held_map.get(&keycode) {
        match (state, ele_state) {
            (bs::Pressed, es::Released) => ButtonState::Released, // pressed -> (released) -> released
            (bs::Pressed, es::Pressed) => ButtonState::Held, // pressed -> (pressed) -> held
            (bs::Released, es::Pressed) => ButtonState::Pressed, // released -> (pressed) -> pressed
            (bs::Held, es::Released) => ButtonState::Released, // held -> (released) -> released
            (bs, _) => bs.clone(), // same state
        }
    } else { ButtonState::Pressed };
    held_map.insert(keycode, next_state);
}

pub fn handle_game_key_input(
    game_key: GameKey, command: Command,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol, client_id: u8,
) {
    match game_key {
        GameKey::Pressable(key) => {
            handle_pressable_key(command, key, held_map, protocol, client_id);
        }
        GameKey::Heldable(key) => {
            handle_heldable_key(command, key, held_map, protocol, client_id);
        }
        GameKey::PressRelease(key) => {
            handle_press_release_key(command, key, held_map, protocol, client_id);
        }
    }
}

fn handle_pressable_key(
    command: Command, key: PressableKey,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol, client_id: u8,
) {
    info!("Received game key: {:?}", command);
    let message: Message = Message::new(
        HostRole::Client(client_id),
        Payload::Command(command.clone()),
    );
    protocol.send_message(&message).expect("send message fails");
    info!("Sent command: {:?}", command);
}

fn handle_heldable_key(
    command: Command, key: HeldableKey,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol, client_id: u8,
) {
    info!("Received game key: {:?}", command);
    let message: Message = Message::new(
        HostRole::Client(client_id),
        Payload::Command(command.clone()),
    );
    protocol.send_message(&message).expect("send message fails");
    info!("Sent command: {:?}", command);
}

fn handle_press_release_key(
    command: Command, key: PressReleaseKey,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol, client_id: u8,
) {
    info!("Received game key: {:?}", command);
    let message: Message = Message::new(
        HostRole::Client(client_id),
        Payload::Command(command.clone()),
    );
    protocol.send_message(&message).expect("send message fails");
    info!("Sent command: {:?}", command);
}

/// ***********************************************Mouse********************************************

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