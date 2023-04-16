use common::communication::commons::*;
use common::communication::message::*;
use common::core::command::{Command, MoveDirection};
use log::info;
use queues::{IsQueue, Queue};
use winit::event::{DeviceEvent, KeyboardInput, VirtualKeyCode};

pub fn handle_keyboard_input(input: KeyboardInput, protocol: &mut Protocol, client_id: u8) {

    // map keyboard input to command
    let command: Option<Command> = match input.virtual_keycode {
        Some(VirtualKeyCode::W) => Some(Command::Move(MoveDirection::Forward)),
        Some(VirtualKeyCode::A) => Some(Command::Move(MoveDirection::Left)),
        Some(VirtualKeyCode::S) => Some(Command::Move(MoveDirection::Backward)),
        Some(VirtualKeyCode::D) => Some(Command::Move(MoveDirection::Right)),
        Some(VirtualKeyCode::Space) => {
            Some(Command::Spawn) // TODO: Place it somewhere else
        }
        _ => None,
    };

    info!("Received command: {:?}", command);
    if let Some(command) = command {
        let message: Message = Message::new(
            HostRole::Client(client_id),
            Payload::Command(command.clone()),
        );
        protocol.send_message(&message).expect("send message fails");
        info!("Sent command: {:?}", command);
    } else {
        info!("No command to send");
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
