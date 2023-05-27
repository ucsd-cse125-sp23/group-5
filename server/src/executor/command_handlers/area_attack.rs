extern crate nalgebra_glm as glm;

use common::configs::game_config::ConfigGame;
use derive_more::Constructor;
use rapier3d::prelude as rapier;
use rapier3d::{geometry, pipeline};

use common::configs::physics_config::ConfigPhysics;
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType};
use common::core::powerup_system::{OtherEffects, PowerUpEffects, StatusEffect};
use common::core::states::GameState;

use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;

use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};

#[derive(Constructor)]
pub struct AreaAttackCommandHandler {
    player_id: u32,
    physics_config: ConfigPhysics,
    game_config: ConfigGame,
}

impl CommandHandler for AreaAttackCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        // if player is stunned
        if player_state.holds_status_effect_mut(StatusEffect::Other(OtherEffects::Stun)) {
            return Ok(());
        }

        // if attack on cooldown, or cannot consume charge, do nothing for now
        if player_state.command_on_cooldown(Command::AreaAttack)
            || !player_state
                .try_consume_wind_charge(Some(self.physics_config.attack_config.area_attack_cost))
        {
            return Ok(());
        }

        // when attacking, remove invisibility
        if player_state.holds_status_effect_mut(StatusEffect::Power(PowerUpEffects::Invisible)) {
            player_state
                .status_effects
                .remove(&StatusEffect::Power(PowerUpEffects::Invisible));
            player_state.power_up = None;
        }

        let wind_enhanced = player_state
            .status_effects
            .contains_key(&StatusEffect::Power(PowerUpEffects::EnhancedWind));
        let scalar = if wind_enhanced {
            self.game_config.powerup_config.wind_enhancement_scalar
        } else {
            1.0
        };

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

        player_state.insert_cooldown(
            Command::AreaAttack,
            self.physics_config.attack_config.area_attack_cooldown,
        );

        // TODO: add sound/particles for area attack
        /*
        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_pos,
                "wind".to_string(),
                (self.player_id, false),
            )),
            Recipients::All,
        );
        */
        game_events.add(
            GameEvent::ParticleEvent(ParticleSpec::new(
                ParticleType::AREA_ATTACK,
                player_pos,
                glm::vec3(0.0, 0.0, 0.0),
                //TODO: placeholder for player color
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(0.4, 0.9, 0.7, 1.0),
                format!("Area Attack from player {}", self.player_id),
            )),
            Recipients::All,
        );
        // loop over all other players
        for (other_player_id, other_player_state) in game_state.players.iter_mut() {
            if &self.player_id == other_player_id {
                continue;
            }

            // other player not affected if invincible
            if other_player_state
                .holds_status_effect(StatusEffect::Power(PowerUpEffects::Invincible))
            {
                continue;
            }

            // get direction from this player to other player
            let other_player_pos = other_player_state.transform.translation;
            let vec_to_other = glm::normalize(&(other_player_pos - player_pos));

            // send ray to other player (may need multiple later)
            let solid = true;
            let filter = pipeline::QueryFilter::default().exclude_collider(player_collider_handle);

            let ray = geometry::Ray::new(
                rapier::point![player_pos.x, player_pos.y, player_pos.z],
                rapier::vector![vec_to_other.x, vec_to_other.y, vec_to_other.z],
            );
            if let Some((handle, toi)) = physics_state.query_pipeline.cast_ray(
                &physics_state.bodies,
                &physics_state.colliders,
                &ray,
                self.physics_config.attack_config.max_area_attack_dist,
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

                    let attack_strength = self.physics_config.attack_config.area_attack_impulse
                        - (self.physics_config.attack_config.area_attack_coeff * toi);

                    let impulse_vec = scalar * vec_to_other * attack_strength;

                    // clear velocity of target before applying impulse
                    other_player_rigid_body.set_linvel(rapier::vector![0.0, 0.0, 0.0], true);

                    // apply_stun
                    // super::apply_stun(
                    //     other_player_state,
                    //     attack_strength / self.physics_config.attack_config.area_attack_impulse
                    //         * self.physics_config.attack_config.max_attack_stun_duration,
                    // );

                    // TODO:
                    other_player_state.status_effects.insert(
                        StatusEffect::Other(OtherEffects::MovementDisabled),
                        attack_strength / self.physics_config.attack_config.area_attack_impulse
                            * self.physics_config.attack_config.max_attack_stun_duration,
                    );

                    // apply attack impulse
                    other_player_rigid_body.apply_impulse(
                        rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z],
                        true,
                    );
                }
            }
        }

        Ok(())
    }
}
