use crate::executor::command_handlers::{CommandHandler, GameEventCollector, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use common::core::command::CheatCodeControl;
use common::core::states::GameState;
use derive_more::Constructor;

#[derive(Constructor)]
pub struct CheatCodeControlCommandHandler {
    player_id: u32,
    _command: CheatCodeControl,
}

impl CommandHandler for CheatCodeControlCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        _: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        match self._command {
            CheatCodeControl::Activate => {
                game_state
                    .player_mut(self.player_id)
                    .unwrap()
                    .cheat_keys_enabled = true;
            }
            CheatCodeControl::Deactivate => {
                game_state
                    .player_mut(self.player_id)
                    .unwrap()
                    .cheat_keys_enabled = false;
            }
        }
        Ok(())
    }
}
