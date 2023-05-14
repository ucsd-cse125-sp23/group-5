use crate::inputs::ClientSync::Ready;
use crate::inputs::Input;
use crate::screen;
use log::{debug, info, warn};
use phf::phf_map;

pub static BUTTON_MAP: phf::Map<&'static str, fn(&mut screen::Display)> = phf_map! {
    "game_start" => game_start,
};

// Place click events here ----------------------
fn game_start(display: &mut screen::Display) {
    // once start game, send ready to the client main.
    match display.sender.send(Input::UI(Ready)) {
        Ok(_) => {}
        Err(e) => {
            warn!("Error sending command: {:?}", e);
        }
    }
    display.current = display.game_display.clone();
}

// end click events ----------------------
