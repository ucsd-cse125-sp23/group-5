use std::time::{Duration, Instant};
use std::sync::mpsc::Receiver;
use glm::TVec3;
use log::{error, info};
use queues::{IsQueue, Queue};
use winit::event::{DeviceEvent, KeyboardInput, MouseScrollDelta};
use common::communication::commons::{DEFAULT_MOUSE_MOVEMENT_INTERVAL, Protocol};
use common::communication::message::{HostRole, Message, Payload};
use common::core::command::Command;
use crate::event_loop::UserInput;

pub mod handlers;

#[derive(Debug)]
pub enum Input {
    Keyboard(KeyboardInput),
    Mouse(DeviceEvent),
    Camera {position: TVec3<f32>, spherical_coords: TVec3<f32>},
}

pub struct InputProcessor {
    protocol: Protocol,
    client_id: u8,
    rx: Receiver<UserInput>,
}

impl InputProcessor {
    pub fn new(protocol: Protocol, client_id: u8, rx: Receiver<UserInput>) -> Self {
        InputProcessor {
            protocol,
            client_id,
            rx,
        }
    }

    pub fn run(&mut self) {
        let mut mouse_motion_buf = Queue::new();
        let mut mouse_wheel_buf = Queue::new();
        let mut sample_start_time = Instant::now();

        let mut last_camera_update_time = Instant::now();

        while let Ok(user_input) = self.rx.recv() {
            info!("Received input: {:?}", user_input);
            match user_input.input {
                Input::Keyboard(input) => {
                    handlers::handle_keyboard_input(input, &mut self.protocol, self.client_id);
                }
                Input::Mouse(input) => {
                    handlers::handle_mouse_input(
                        input,
                        &mut self.protocol,
                        &mut mouse_motion_buf,
                        &mut mouse_wheel_buf,
                    );

                    if sample_start_time.elapsed() < Duration::from_millis(DEFAULT_MOUSE_MOVEMENT_INTERVAL) {
                        continue;
                    }

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
                    sample_start_time = Instant::now();

                    self.protocol
                        .send_message(&Message::new(
                            HostRole::Client(self.client_id),
                            Payload::Command(Command::Turn(Default::default())),
                        ))
                        .expect("send message fails");
                    info!("Sent command: {:?}", "Turn");
                },
                // receive camera update and it is past the interval since last update
                Input::Camera { position, spherical_coords }
                if last_camera_update_time.elapsed() >= Duration::from_millis(100) => {
                    self.protocol
                        .send_message(&Message::new(
                            HostRole::Client(self.client_id),
                            Payload::Command(Command::UpdateCamera {
                                position,
                                spherical_coords,
                            }),
                        ))
                        .expect("send message fails");
                    last_camera_update_time = Instant::now();
                }
                _ => {}
            }
        }
    }
}
