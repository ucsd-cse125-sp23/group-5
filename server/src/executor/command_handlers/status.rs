use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::powerup_system::{PowerUpEffects, StatusEffect, OtherEffects};
use common::core::states::GameState;
use derive_more::Constructor;

use crate::executor::command_handlers::{CommandHandler, GameEventCollector, HandlerResult};
use crate::simulation::physics_state::PhysicsState;

extern crate nalgebra_glm as glm;

const SLIPPERY_FRICTION_DECREASE: f32 = 1.2;

#[derive(Constructor)]
pub struct StatusEffectCommandHandler {}

impl CommandHandler for StatusEffectCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        for (&player_id, player_state) in game_state.players.iter() {
            if player_state.holds_status_effect(StatusEffect::Other(OtherEffects::Slippery)) {
                let body = physics_state.get_entity_rigid_body_mut(player_id).unwrap();
                body.set_linear_damping(0.);

                let collider = physics_state.get_entity_collider_mut(player_id).unwrap();
                collider.set_friction(collider.friction() - SLIPPERY_FRICTION_DECREASE);
            }
        }
        Ok(())
    }
}
