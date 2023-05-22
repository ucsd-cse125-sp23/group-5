use super::{CommandHandler, GameEventCollector, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use common::configs::game_config::ConfigGame;
use common::core::command::Command;
use common::core::powerup_system::StatusEffect;
use common::core::states::GameState;
use derive_more::Constructor;

#[derive(Constructor)]
pub struct RefillCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for RefillCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        let spawn_position = game_state.player(self.player_id).unwrap().spawn_point;
        let player_state = game_state.player_mut(self.player_id).unwrap();

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        if !player_state.is_in_circular_area(
            (spawn_position.x, spawn_position.z),
            self.game_config.refill_radius,
            (None, None),
        ) || player_state.command_on_cooldown(Command::Refill)
        {
            // signal player that he/she is not in refill area
            return Ok(());
        }
        player_state.refill_wind_charge(
            Some(self.game_config.one_charge),
            self.game_config.max_wind_charge,
        );
        player_state.insert_cooldown(Command::Refill, self.game_config.refill_rate_limit);
        Ok(())
    }
}
