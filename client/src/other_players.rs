use common::core::states::GameState;

use common::configs::game_config::ConfigGame;
use nalgebra_glm as glm;
use std::ops::Deref;

pub struct OtherPlayer {
    pub id: u32,
    pub visible: bool, // don't want to render location above players not in scene graph
    pub location: glm::Vec4,
    pub score: f32,
}

pub fn load_game_state(
    vec: &mut Vec<OtherPlayer>,
    game_state: impl Deref<Target = GameState>,
    game_config: ConfigGame,
) {
    for player_state in game_state.players.values() {
        let id = player_state.id as usize - 1;
        let score = player_state.on_flag_time / game_config.winning_threshold;
        vec[id].score = score;
    }
}
