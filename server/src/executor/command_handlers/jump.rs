use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use common::configs::parameters::{JUMP_IMPULSE, MAX_JUMP_COUNT};
use common::core::action_states::ActionState;
use common::core::powerup_system::StatusEffect;
use common::core::states::GameState;
use derive_more::Constructor;
use itertools::Itertools;
use nalgebra::Vector3;
use std::f32::consts::PI;
use std::time::Duration;

extern crate nalgebra_glm as glm;

use rapier3d::prelude as rapier;
use common::configs::physics_config::ConfigPhysics;

#[derive(Constructor)]
pub struct JumpCommandHandler {
    player_id: u32,
    physics_config: ConfigPhysics,
}

impl CommandHandler for JumpCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_collider_handle = physics_state
            .get_entity_handles(self.player_id)
            .ok_or(HandlerError::new(format!(
                "Handlers for player {} not found",
                self.player_id
            )))?
            .collider
            .ok_or(HandlerError::new(format!(
                "Collider for player {} not found",
                self.player_id
            )))?;

        let _player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .ok_or(HandlerError::new(format!(
                "Rigid body for player {} not found",
                self.player_id
            )))?;

        let contact_pairs = physics_state
            .narrow_phase
            .contacts_with(player_collider_handle)
            .collect_vec();

        let mut should_reset_jump = false;
        for contact_pair in contact_pairs {
            if let Some((manifold, _)) = contact_pair.find_deepest_contact() {
                // see if player is above another collider by testing the normal angle
                if nalgebra_glm::angle(&manifold.data.normal, &Vector3::y()) < PI / 3. {
                    should_reset_jump = true;
                }
            }
        }

        let mut player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        if should_reset_jump {
            player_state.jump_count = 0;
        }

        let jump_limit = if player_state
            .status_effects
            .contains_key(&StatusEffect::TripleJump)
        {
            MAX_JUMP_COUNT + 1
        } else {
            MAX_JUMP_COUNT
        };

        if player_state.jump_count >= jump_limit {
            return Ok(());
        }

        player_state.jump_count += 1;

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();
        player_rigid_body.apply_impulse(rapier::vector![0.0, JUMP_IMPULSE, 0.0], true);

        player_state.active_action_states.insert((
            ActionState::Jumping,
            Duration::from_secs_f32(if player_state.jump_count == 2 {
                1.4
            } else {
                0.9
            }),
        ));

        super::handle_invincible_players(game_state, physics_state, self.player_id);

        Ok(())
    }
}
