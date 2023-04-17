use common::communication::commons::*;
use common::communication::message::*;
use common::core::command::{Command, MoveDirection};
use log::{error, info};
use queues::{IsQueue, Queue};
use std::time::Instant;
use winit::event::{DeviceEvent, KeyboardInput, MouseScrollDelta, VirtualKeyCode};

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

    let n = mouse_motion_buf.size();
    for _ in 1..n {
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
