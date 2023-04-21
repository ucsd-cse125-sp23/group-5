use crate::event_loop::UserInput;
use common::communication::commons::{Protocol, DEFAULT_MOUSE_MOVEMENT_INTERVAL};
use common::communication::message::{HostRole, Message, Payload};
use common::core::command::Command;
use log::{error, info};
use queues::{IsQueue, Queue};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use winit::event::{DeviceEvent, KeyboardInput, MouseScrollDelta, VirtualKeyCode};

pub mod handlers;

#[derive(Debug)]
pub enum Input {
    Keyboard(KeyboardInput),
    Mouse(DeviceEvent),
}

#[derive(Debug, Clone)]
pub enum ButtonState {
    Pressed,
    Held,
    Released,
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
        let mut held_map: HashMap<VirtualKeyCode, ButtonState> = HashMap::new();

        let mut mouse_motion_buf = Queue::new();
        let mut mouse_wheel_buf = Queue::new();
        let mut sample_start_time = Instant::now();

        // should keyboard and mouse be running on separate threads as well?
        while let Ok(user_input) = self.rx.recv() {
            info!("Received input: {:?}", user_input);
            match user_input.input {
                Input::Keyboard(input) => {
                    handlers::handle_keyboard_input(
                        &mut held_map,
                        input,
                        &mut self.protocol,
                        self.client_id,
                    );
                }
                Input::Mouse(input) => {
                    handlers::handle_mouse_input(
                        input,
                        &mut mouse_motion_buf,
                        &mut mouse_wheel_buf,
                    );
                }
            }

            // Should always check? buffered mouse inputs cannot be good right?
            // ideally runs in a always checking thread
            if !(mouse_motion_buf.size() == 0 && mouse_wheel_buf.size() == 0) {
                if sample_start_time.elapsed()
                    >= Duration::from_millis(DEFAULT_MOUSE_MOVEMENT_INTERVAL)
                {
                    handlers::send_mouse_input(
                        &mut mouse_motion_buf,
                        &mut mouse_wheel_buf,
                        &mut sample_start_time,
                        &mut self.protocol,
                        self.client_id,
                    );
                }
            }
        }
    }
}
