extern crate nalgebra_glm as glm;

use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::game_config::ConfigGame;
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::states::GameState;
use derive_more::Constructor;

#[derive(Constructor)]
pub struct GivePowerUpCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for GivePowerUpCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        let player_pos = player_state.transform.translation;
        // TODO: maybe add sound for picking up powerup?

        game_events.add(
            GameEvent::ParticleEvent(ParticleSpec::new(
                ParticleType::POWERUP,
                player_pos,
                glm::vec3(0.0, 0.0, 0.0),
                //TODO: placeholder for player color
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(0.882, 0.749, 0.165, 1.0),
                format!("Give Power Up from player {}", self.player_id),
            )),
            Recipients::All,
        );

        Ok(())
    }
}
