use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use common::core::powerup_system::{OtherEffects, StatusEffect};
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra_glm::Vec3;

#[derive(Constructor)]
pub struct UpdateCameraFacingCommandHandler {
    player_id: u32,
    forward: Vec3,
}

impl CommandHandler for UpdateCameraFacingCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        // Game state
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        // if player is stunned
        if player_state.holds_status_effect_mut(StatusEffect::Other(OtherEffects::Stun)) {
            return Ok(());
        }

        player_state.camera_forward = self.forward;

        Ok(())
    }
}
