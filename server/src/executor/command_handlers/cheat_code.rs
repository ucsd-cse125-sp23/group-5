use derive_more::Constructor;

use common::core::powerup_system::{PowerUp, PowerUpStatus};
use common::core::states::GameState;

use crate::executor::command_handlers::{CommandHandler, GameEventCollector, HandlerResult};
use crate::simulation::physics_state::PhysicsState;

#[derive(Constructor)]
pub struct CheatCodeCommandHandler {
    player_id: u32,
    powerup: PowerUp,
}

impl CommandHandler for CheatCodeCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        _: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state.player_mut(self.player_id).unwrap();
        if player_state.cheat_keys_enabled {
            player_state.power_up = Some((self.powerup.clone(), PowerUpStatus::Held));
        }
        Ok(())
    }
}
