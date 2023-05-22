use crate::simulation::physics_state::PhysicsState;
use common::core::command::Command;
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::zero;
use rapier3d::math::Isometry;

use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};

extern crate nalgebra_glm as glm;
use common::configs::game_config::ConfigGame;
use rapier3d::prelude as rapier;

#[derive(Constructor)]
pub struct DieCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for DieCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        player_state.reset_status_effects();

        let spawn_position = player_state.spawn_point;

        // Teleport the player back to their spawn position and disable physics.
        let new_position = Isometry::new(spawn_position, zero());
        if let Some(player_rigid_body) = physics_state.get_entity_rigid_body_mut(self.player_id) {
            player_rigid_body.set_position(new_position, true);
            player_rigid_body.set_linvel(rapier::vector![0.0, 0.0, 0.0], true);
            player_rigid_body.set_enabled(false);
        }

        player_state.is_dead = true;
        player_state.insert_cooldown(Command::Spawn, self.game_config.spawn_cooldown);

        Ok(())
    }
}
