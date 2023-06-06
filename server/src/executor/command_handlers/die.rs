use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
extern crate nalgebra_glm as glm;
use crate::Recipients;
use crate::simulation::physics_state::PhysicsState;
use common::core::events::GameEvent;
use common::{configs::game_config::ConfigGame, core::events::SoundSpec};
use common::core::command::Command;
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::zero;
use rapier3d::math::Isometry;
use rapier3d::prelude as rapier;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // calculate elapsed time since game start in seconds
        let elapsed_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - game_state.game_start_time;
        let elapsed_seconds = elapsed_time.as_secs();
        // increase spawn_cooldown based on elapsed time
        let spawn_cooldown_increase = elapsed_seconds as f32 * self.game_config.respawn_coef;
        let new_spawn_cooldown = self.game_config.spawn_cooldown + spawn_cooldown_increase;

        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_state.transform.translation,
                "die".to_string(),
                (self.player_id, true),
                (false, false),
                player_state.camera_forward,

            )),
            Recipients::One(self.player_id as u8),
        );

        player_state.reset_status_effects();
        player_state.power_up = None;

        let spawn_position = player_state.spawn_point;

        // Teleport the player back to their spawn position and disable physics.
        let new_position = Isometry::new(spawn_position, zero());
        if let Some(player_rigid_body) = physics_state.get_entity_rigid_body_mut(self.player_id) {
            player_rigid_body.set_position(new_position, true);
            player_rigid_body.set_linvel(rapier::vector![0.0, 0.0, 0.0], true);
            player_rigid_body.set_enabled(false);
        }

        player_state.is_dead = true;
        player_state.jump_count = 1;
        player_state.respawn_sec = 3;
        player_state.insert_cooldown(Command::Spawn, new_spawn_cooldown);

        Ok(())
    }
}
