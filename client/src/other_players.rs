use common::core::states::GameState;

use nalgebra_glm as glm;
use std::ops::Deref;

pub struct OtherPlayer {
    pub id: u32,
    pub visible: bool, // don't want to render location above players not in scene graph
    pub location: glm::Vec4,
    pub score: f32,
}

pub fn load_game_state(vec: &mut Vec<OtherPlayer>, game_state: impl Deref<Target = GameState>) {
    for player_state in game_state.players.values() {
        let id = player_state.id as usize - 1;
        let score = player_state.on_flag_time / common::configs::parameters::WINNING_THRESHOLD;
        vec[id].score = score;
    }
}
