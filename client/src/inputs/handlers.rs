use crate::inputs::ButtonState;
use common::communication::commons::*;
use common::communication::message::*;
use common::core::command::Command::{Action, Spawn};
use common::core::command::GameAction::Attack;
use common::core::command::{Command, MoveDirection};
use log::{error, info};
use queues::{IsQueue, Queue};
use std::collections::HashMap;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};

pub enum GameKey {
    Pressable,
    Holdable,
    PressRelease,
}


pub fn handle_keyboard_input(
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    input: KeyboardInput,
    protocol: &mut Protocol,
    client_id: u8,
) {
    // change state
    //let mut functional_key= Some(..);
    if let Some(keycode) = input.virtual_keycode {
        update_held_map(held_map, keycode, input.state);
        // map keyboard input to command
        let key_command: Option<(GameKey, Command)> = match input.virtual_keycode {
            // match Holdable keys
            Some(VirtualKeyCode::W) => Some((
                GameKey::Holdable,
                Command::Move(MoveDirection::Forward),
            )),
            Some(VirtualKeyCode::A) => Some((
                GameKey::Holdable,
                Command::Move(MoveDirection::Left),
            )),
            Some(VirtualKeyCode::S) => Some((
                GameKey::Holdable,
                Command::Move(MoveDirection::Backward),
            )),
            Some(VirtualKeyCode::D) => Some((
                GameKey::Holdable,
                Command::Move(MoveDirection::Right),
            )),
            // match Pressable keys
            Some(VirtualKeyCode::Space) => Some((GameKey::Pressable, Spawn)),
            // match PressRelease keys
            Some(VirtualKeyCode::F) => {
                Some((GameKey::PressRelease, Action(Attack)))
            }
            _ => None,
        };

        if let Some((game_key, command)) = key_command {
            handle_game_key_input(game_key, command, keycode, held_map, protocol, client_id);
        } else {
            info!("No game key or action to handle");
        }
    }
}

pub fn update_held_map(
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    keycode: VirtualKeyCode,
    ele_state: ElementState,
) {
    use winit::event::ElementState as es;
    use ButtonState as bs;
    let next_state = if let Some(state) = held_map.get(&keycode) {
        match (state, ele_state) {
            (bs::Pressed, es::Pressed) => ButtonState::Held, // pressed -> (pressed) -> held
            (bs::Released, es::Pressed) => ButtonState::Pressed, // released -> (pressed) -> pressed
            (_, es::Released) => ButtonState::Released,      // some -> (released) -> released
            (bs, _) => bs.clone(),                           // same state
        }
    } else {
        ButtonState::Pressed
    };
    held_map.insert(keycode, next_state);
}

pub fn handle_game_key_input(
    game_key: GameKey,
    command: Command,
    keycode: VirtualKeyCode,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol,
    client_id: u8,
) {
    match game_key {
        GameKey::Pressable => {
            handle_pressable_key(command, keycode, held_map, protocol, client_id);
        }
        GameKey::Holdable => {
            handle_holdable_key(command, keycode, held_map, protocol, client_id);
        }
        GameKey::PressRelease => {
            handle_press_release_key(command, keycode, held_map, protocol, client_id);
        }
    }
}

fn handle_pressable_key(
    command: Command,
    keycode: VirtualKeyCode,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol,
    client_id: u8,
) {
    info!("Received game key: {:?}", command);
    match held_map.get(&keycode) {
        // if pressed, send command
        Some(ButtonState::Pressed) => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            info!("Sent command: {:?}", command);
        }
        // if released, remove from the held map
        Some(ButtonState::Released) => {
            held_map.remove(&keycode);
        }
        // if held or others, don't do nothing
        _ => {}
    }
}

fn handle_holdable_key(
    command: Command,
    keycode: VirtualKeyCode,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol,
    client_id: u8,
) {
    info!("Received game key: {:?}", command);
    match held_map.get(&keycode) {
        // if pressed or held, keep sending command
        Some(ButtonState::Pressed) | Some(ButtonState::Held) => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            info!("Sent command: {:?}", command);
        }
        // if released, remove from the held map
        Some(ButtonState::Released) => {
            held_map.remove(&keycode);
        }
        _ => {}
    }
}

fn handle_press_release_key(
    command: Command,
    keycode: VirtualKeyCode,
    held_map: &mut HashMap<VirtualKeyCode, ButtonState>,
    protocol: &mut Protocol,
    client_id: u8,
) {
    info!("Received game key: {:?}", command);
    match held_map.get(&keycode) {
        // if pressed or held, keep sending command
        Some(ButtonState::Pressed) | Some(ButtonState::Held) => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            info!("Sent command: {:?}", command);
        }
        // if released, do the corresponding action and then remove from the held map
        Some(ButtonState::Released) => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            info!("Sent command: {:?}", command);
            held_map.remove(&keycode);
        }
        _ => {}
    }
}

/// ***************************************** Mouse *********************************************///

// mw for mouse wheel, mm for mouse motion
pub fn handle_mouse_input(
    input: DeviceEvent,
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
