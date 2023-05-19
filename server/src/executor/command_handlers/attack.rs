use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::parameters::{
    ATTACKING_COOLDOWN, ATTACK_COEFF, ATTACK_COOLDOWN, ATTACK_COST, ATTACK_IMPULSE,
    MAX_ATTACK_ANGLE, MAX_ATTACK_DIST, WIND_ENHANCEMENT_SCALAR,
};
use common::core::action_states::ActionState;
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::powerup_system::StatusEffect;
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::UnitQuaternion;
use nalgebra_glm::Vec3;
use rapier3d::{geometry, pipeline};
use std::time::Duration;

extern crate nalgebra_glm as glm;
use crate::executor::command_handlers::{
    CommandHandler, GameEventCollector, HandlerError, HandlerResult,
};
use rapier3d::prelude as rapier;
use common::configs::game_config::ConfigGame;
use common::configs::physics_config::ConfigPhysics;

#[derive(Constructor)]
pub struct AttackCommandHandler {
    player_id: u32,
    physics_config: ConfigPhysics,
    game_config: ConfigGame,
}

impl CommandHandler for AttackCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        // if attack on cooldown, or cannot consume charge, do nothing for now
        if player_state.command_on_cooldown(Command::Attack)
            || !player_state.try_consume_wind_charge(Some(ATTACK_COST))
        {
            return Ok(());
        }

        player_state.status_effects.remove(&StatusEffect::Invisible);

        let player_pos = player_state.transform.translation;

        let player_collider_handle = physics_state
            .get_entity_handles(self.player_id)
            .ok_or(HandlerError::new(format!(
                "Player {} not found",
                self.player_id
            )))?
            .collider
            .ok_or(HandlerError::new(format!(
                "Player {} does not have a collider",
                self.player_id
            )))?;

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();

        let camera_forward = player_state.camera_forward;
        let horizontal_camera_forward = Vec3::new(
            player_state.camera_forward.x,
            0.0,
            player_state.camera_forward.z,
        );

        // turn player towards attack direction (camera_forward)
        let rotation = UnitQuaternion::face_towards(&horizontal_camera_forward, &Vec3::y());
        player_rigid_body.set_rotation(rotation, true);

        player_state.insert_cooldown(Command::Attack, ATTACK_COOLDOWN);

        // send game events for attack sound/particles
        // TODO: replace this example with actual implementation
        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_pos,
                "wind".to_string(),
                (self.player_id, false),
            )),
            Recipients::All,
        );
        game_events.add(
            GameEvent::ParticleEvent(ParticleSpec::new(
                ParticleType::ATTACK,
                player_pos,
                camera_forward,
                //TODO: placeholder for player color
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(0.4, 0.9, 0.7, 1.0),
                format!("Attack from player {}", self.player_id),
            )),
            Recipients::All,
        );

        let wind_enhanced = player_state
            .status_effects
            .contains_key(&StatusEffect::EnhancedWind);
        let scalar = if wind_enhanced {
            WIND_ENHANCEMENT_SCALAR
        } else {
            1.0
        };

        player_state.active_action_states.insert((
            ActionState::Attacking,
            Duration::from_secs_f32(ATTACKING_COOLDOWN),
        ));

        // loop over all other players
        for (other_player_id, other_player_state) in game_state.players.iter() {
            if &self.player_id == other_player_id {
                continue;
            }

            if game_state
                .player(*other_player_id)
                .unwrap()
                .status_effects
                .contains_key(&StatusEffect::Invincible)
            {
                continue;
            }

            // get direction from this player to other player
            let other_player_pos = other_player_state.transform.translation;
            let vec_to_other = glm::normalize(&(other_player_pos - player_pos));

            // check dot product between direction to other player and attack direction
            let angle = glm::angle(&camera_forward, &vec_to_other);

            // if object in attack range
            if angle <= MAX_ATTACK_ANGLE * scalar {
                // send ray to other player (may need multiple later)
                let solid = true;
                let filter =
                    pipeline::QueryFilter::default().exclude_collider(player_collider_handle);

                let ray = geometry::Ray::new(
                    rapier::point![player_pos.x, player_pos.y, player_pos.z],
                    rapier::vector![vec_to_other.x, vec_to_other.y, vec_to_other.z],
                );
                if let Some((handle, toi)) = physics_state.query_pipeline.cast_ray(
                    &physics_state.bodies,
                    &physics_state.colliders,
                    &ray,
                    MAX_ATTACK_DIST,
                    solid,
                    filter,
                ) {
                    let other_player_collider_handle = physics_state
                        .get_entity_handles(*other_player_id)
                        .ok_or(HandlerError::new(format!(
                            "Player {} not found",
                            self.player_id
                        )))?
                        .collider
                        .ok_or(HandlerError::new(format!(
                            "Player {} does not have a collider",
                            self.player_id
                        )))?;

                    // if ray hit the correct target (the other player), apply force
                    if handle == other_player_collider_handle {
                        let other_player_rigid_body = physics_state
                            .get_entity_rigid_body_mut(*other_player_id)
                            .unwrap();

                        let impulse_vec =
                            scalar * vec_to_other * (ATTACK_IMPULSE - (ATTACK_COEFF * toi));

                        other_player_rigid_body.apply_impulse(
                            rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z],
                            true,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
