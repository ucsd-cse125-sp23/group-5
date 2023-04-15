use winit::event::*;

pub enum Inputs {
    Keyboard(KeyboardInput),
    Mouse(DeviceEvent),
}
