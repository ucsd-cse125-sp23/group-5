use crate::inputs::ButtonState;
use common::communication::commons::*;
use common::communication::message::*;

use common::core::command::Command;
use glm::Vec3;
use log::{debug, error, info};
use nalgebra_glm as glm;

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
