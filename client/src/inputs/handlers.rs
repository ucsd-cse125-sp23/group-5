use crate::inputs::ButtonState;
use common::communication::commons::*;
use common::communication::message::*;

use common::core::command::Command;
use glm::Vec3;
use log::{debug, error, info};
use nalgebra_glm as glm;
use queues::{IsQueue, Queue};

use std::time::Instant;
use winit::event::{DeviceEvent, MouseScrollDelta};

pub enum GameKeyKind {
    Pressable,
    Holdable,
    PressRelease,
}

pub fn handle_camera_update(camera_forward: Vec3, protocol: &mut Protocol, client_id: u8) {
    let message: Message = Message::new(
        HostRole::Client(client_id),
        Payload::Command(Command::UpdateCamera {
            forward: camera_forward,
        }),
    );
    protocol.send_message(&message).expect("send message fails");
    // info!("Sent camera update");
}

pub fn handle_game_key_input(
    game_key_kind: GameKeyKind,
    command: Command,
    button_state: &ButtonState,
    protocol: &mut Protocol,
    client_id: u8,
) -> bool {
    match game_key_kind {
        GameKeyKind::Pressable => handle_pressable_key(command, button_state, protocol, client_id),
        GameKeyKind::Holdable => handle_holdable_key(command, button_state, protocol, client_id),
        GameKeyKind::PressRelease => {
            handle_press_release_key(command, button_state, protocol, client_id)
        }
    }
}

fn handle_pressable_key(
    command: Command,
    button_state: &ButtonState,
    protocol: &mut Protocol,
    client_id: u8,
) -> bool {
    match button_state {
        // if pressed, send command
        ButtonState::Pressed => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            debug!("Sent command: {:?}", command);
        }
        // if released, remove from the held map
        ButtonState::Released => return false,
        // if held or others, don't do nothing
        _ => {}
    };
    true
}

fn handle_holdable_key(
    command: Command,
    button_state: &ButtonState,
    protocol: &mut Protocol,
    client_id: u8,
) -> bool {
    match button_state {
        // if pressed or held, keep sending command
        ButtonState::Pressed | ButtonState::Held => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            debug!("Sent command: {:?}", command);
        }
        // if released, remove from the held map
        ButtonState::Released => {
            return false;
        }
        _ => {}
    }
    true
}

fn handle_press_release_key(
    command: Command,
    button_state: &ButtonState,
    protocol: &mut Protocol,
    client_id: u8,
) -> bool {
    match button_state {
        // if pressed or held, keep sending command
        ButtonState::Pressed | ButtonState::Held => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            debug!("Sent command: {:?}", command);
        }
        // if released, do the corresponding action and then remove from the held map
        ButtonState::Released => {
            let message: Message = Message::new(
                HostRole::Client(client_id),
                Payload::Command(command.clone()),
            );
            protocol.send_message(&message).expect("send message fails");
            debug!("Sent command: {:?}", command);
            return false;
        }
        _ => {}
    };
    true
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
