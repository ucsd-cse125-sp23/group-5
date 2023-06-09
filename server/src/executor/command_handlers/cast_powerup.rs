extern crate nalgebra_glm as glm;

use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::game_config::ConfigGame;
use common::core::action_states::ActionState;
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::powerup_system::OtherEffects::{Slippery, Stun};
use common::core::powerup_system::PowerUpEffects::Invincible;
use common::core::powerup_system::{
    OtherEffects, PowerUp, PowerUpEffects, PowerUpStatus, StatusEffect, POWER_UP_TO_EFFECT_MAP,
};
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::{zero, UnitQuaternion};
use nalgebra_glm::Vec3;
use rapier3d::math::Isometry;
use rapier3d::prelude as rapier;
use rapier3d::{geometry, pipeline};
use std::time::Duration;

#[derive(Constructor)]
pub struct CastPowerUpCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for CastPowerUpCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        let player_state = game_state
            .player(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?
            .clone();

        // if player is dead, don't do anything
        if player_state.is_dead {
            return Ok(());
        }

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Other(Stun))
        {
            return Ok(());
        } // Maybe Add Cleanse?

        // if powerup is on cooldown, or does not have a powerup, return
        if player_state.power_up.is_none() {
            return Ok(());
        }

        if let Some((x, PowerUpStatus::Active)) = player_state.power_up.clone() {
            {
                let player_state = game_state.player_mut(self.player_id).unwrap();
                // when dashing or flashing, remove invisibility
                super::remove_invisibility(player_state);
            }
            return match x {
                PowerUp::Flash => flash(
                    game_state,
                    self.player_id,
                    self.game_config.clone(),
                    physics_state,
                    game_events,
                ),
                PowerUp::Dash => dash(
                    game_state,
                    self.player_id,
                    self.game_config.clone(),
                    physics_state,
                    game_events,
                ),
                _ => Ok(()),
            };
        }

        let mut other_player_status_changes: Vec<(u32, StatusEffect, f32)> = vec![];

        if let Some((x, PowerUpStatus::Held)) = player_state.power_up.clone() {
            {
                let player_state = game_state.player_mut(self.player_id).unwrap();
                // when using a powerup, remove invisibility
                super::remove_invisibility(player_state);
                player_state
                    .active_action_states
                    .insert((ActionState::CastingPowerUp, Duration::from_secs_f32(1.666)));
            }
            match x {
                PowerUp::Blizzard => {
                    let player_pos = player_state.transform.translation;

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
                    let rotation =
                        UnitQuaternion::face_towards(&horizontal_camera_forward, &Vec3::y());
                    player_rigid_body.set_rotation(rotation, true);

                    // send game events for attack sound/particles
                    game_events.add(
                        GameEvent::ParticleEvent(ParticleSpec::new(
                            ParticleType::BLIZZARD,
                            player_pos,
                            horizontal_camera_forward,
                            //TODO: placeholder for player color
                            glm::vec3(0.0, 1.0, 0.0),
                            glm::vec4(1.0, 1.0, 1.0, 1.0),
                            format!("Blizzard from player {}", self.player_id),
                        )),
                        Recipients::All,
                    );

                    game_events.add(
                        GameEvent::SoundEvent(SoundSpec::new(
                            player_pos,
                            "ice".to_string(),
                            (self.player_id, false),
                            (false, false, false),
                            player_state.camera_forward,
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

                        // check dot product between direction to other player and attack direction
                        let angle = glm::angle(&horizontal_camera_forward, &vec_to_other);

                        let dist = glm::length(&(other_player_pos - player_pos));
                        // if object in attack range
                        if angle <= self.game_config.powerup_config.blizzard_max_attack_angle
                            && dist <= self.game_config.powerup_config.blizzard_max_attack_dist
                        {
                            other_player_status_changes.push((
                                *other_player_id,
                                StatusEffect::Other(Stun),
                                self.game_config.powerup_config.power_up_debuff_duration,
                            ));
                            other_player_status_changes.push((
                                *other_player_id,
                                StatusEffect::Other(Slippery),
                                self.game_config.powerup_config.power_up_debuff_duration,
                            ));

                            other_player_state.active_action_states.insert((
                                ActionState::Frozen,
                                Duration::from_secs_f32(
                                    self.game_config.powerup_config.power_up_debuff_duration,
                                ),
                            ));
                        }
                    }

                    let player_state = game_state.player_mut(self.player_id).unwrap();
                    // Clear blizzard power up after use
                    player_state.power_up = None;
                }
                x => {
                    if x == PowerUp::Invincible {
                        super::reset_weather(physics_state, self.player_id);
                    }
                    let player_state = game_state.player_mut(self.player_id).unwrap();
                    player_state.status_effects.insert(
                        *POWER_UP_TO_EFFECT_MAP.get(&(x.value())).unwrap(),
                        self.game_config.powerup_config.power_up_buff_duration,
                    );
                    // by now the player should have casted the powerup successfully, change powerup status
                    player_state.power_up = Some((
                        player_state.power_up.clone().unwrap().0,
                        PowerUpStatus::Active,
                    ));
                }
            }
        };

        // TODO: replace this example with actual implementation, with sound_id powerups etc.
        if let Some((x, PowerUpStatus::Held)) = player_state.power_up.clone() {
            if x != PowerUp::Blizzard {
                let player_pos = player_state.transform.translation;
                game_events.add(
                    GameEvent::SoundEvent(SoundSpec::new(
                        player_pos,
                        "powerup".to_string(),
                        (self.player_id, true),
                        (false, false, false),
                        player_state.camera_forward,
                    )),
                    Recipients::All // One(self.player_id as u8),
                );
            }
        }
        // End of TODO

        // apply effects to other players
        for (id, effect, duration) in other_player_status_changes.iter() {
            let other_player_state = game_state.player_mut(*id).unwrap();
            if !other_player_state
                .status_effects
                .contains_key(&StatusEffect::Power(Invincible))
            {
                other_player_state.status_effects.insert(*effect, *duration);
            }
        }

        Ok(())
    }
}

fn flash(
    game_state: &mut GameState,
    player_id: u32,
    game_config: ConfigGame,
    physics_state: &mut PhysicsState,
    game_events: &mut dyn GameEventCollector,
) -> HandlerResult {
    let player_state = game_state
        .player_mut(player_id)
        .ok_or_else(|| HandlerError::new(format!("Player {} not found", player_id)))?;
    // if player is stunned
    if player_state.holds_status_effect_mut(StatusEffect::Other(OtherEffects::Stun)) {
        return Ok(());
    }

    // if flash on cooldown, or should not be able to dash, do nothing for now
    if player_state.command_on_cooldown(Command::Flash)
        || !player_state.holds_status_effect_mut(StatusEffect::Power(PowerUpEffects::EnabledFlash))
    {
        return Ok(());
    }

    let player_pos = player_state.transform.translation;

    game_events.add(
        GameEvent::SoundEvent(SoundSpec::new(
            player_pos,
            "flash".to_string(),
            (player_id, true),
            (false, false, false),
            player_state.camera_forward,
        )),
        Recipients::All,
    );

    let player_rigid_body = physics_state.get_entity_rigid_body_mut(player_id).unwrap();

    let camera_forward = Vec3::new(
        player_state.camera_forward.x,
        0.0,
        player_state.camera_forward.z,
    );

    // turn player towards attack direction (camera_forward)
    let rotation = UnitQuaternion::face_towards(&camera_forward, &Vec3::y());
    player_rigid_body.set_rotation(rotation, true);

    player_state.insert_cooldown(Command::Flash, game_config.powerup_config.flash_cooldown);

    // TODO::
    // Flashy particle effect would be cool here
    // game_events.add(
    //     GameEvent::ParticleEvent(ParticleSpec::new(
    //         ParticleType::ATTACK,
    //         player_pos.clone(),
    //         camera_forward.clone(),
    //         glm::vec3(0.0, 1.0, 0.0),
    //         glm::vec4(0.4, 0.9, 0.7, 1.0),
    //         format!("Attack from player {}", self.player_id),
    //     )),
    //     Recipients::All,
    // );

    let x_dir = player_state.camera_forward.x;
    let z_dir = player_state.camera_forward.z;

    let mut new_coordinates = game_state
        .player_mut(player_id)
        .unwrap()
        .transform
        .translation;

    new_coordinates.x += game_config.powerup_config.flash_distance_scalar * x_dir;
    new_coordinates.z += game_config.powerup_config.flash_distance_scalar * z_dir;

    let new_position = Isometry::new(new_coordinates, zero());
    player_rigid_body.set_position(new_position, true);
    Ok(())
}

fn dash(
    game_state: &mut GameState,
    player_id: u32,
    game_config: ConfigGame,
    physics_state: &mut PhysicsState,
    game_events: &mut dyn GameEventCollector,
) -> HandlerResult {
    let player_state = game_state
        .player_mut(player_id)
        .ok_or_else(|| HandlerError::new(format!("Player {} not found", player_id)))?;

    // if player is stunned
    if player_state.holds_status_effect_mut(StatusEffect::Other(OtherEffects::Stun)) {
        return Ok(());
    }

    // if dash on cooldown, or should not be able to dash, do nothing for now
    if player_state.command_on_cooldown(Command::Dash)
        || !player_state.holds_status_effect_mut(StatusEffect::Power(PowerUpEffects::EnabledDash))
    {
        return Ok(());
    }

    player_state.status_effects.insert(
        StatusEffect::Other(OtherEffects::MovementDisabled),
        game_config.powerup_config.dash_blocking_duration,
    );

    let player_pos = player_state.transform.translation;

    game_events.add(
        GameEvent::SoundEvent(SoundSpec::new(
            player_pos,
            "dash".to_string(),
            (player_id, true),
            (false, false, false),
            player_state.camera_forward,
        )),
        Recipients::All,
    );

    let player_rigid_body = physics_state.get_entity_rigid_body_mut(player_id).unwrap();

    let camera_forward = Vec3::new(
        player_state.camera_forward.x,
        0.0,
        player_state.camera_forward.z,
    );

    // turn player towards dash direction (camera_forward)
    let rotation = UnitQuaternion::face_towards(&camera_forward, &Vec3::y());
    player_rigid_body.set_rotation(rotation, true);

    player_state.insert_cooldown(Command::Dash, game_config.powerup_config.dash_cooldown);

    // TODO::
    // some particle at the end would be cool, but probably different
    // game_events.add(
    //     GameEvent::ParticleEvent(ParticleSpec::new(
    //         ParticleType::ATTACK,
    //         player_pos.clone(),
    //         camera_forward.clone(),
    //         glm::vec3(0.0, 1.0, 0.0),
    //         glm::vec4(0.4, 0.9, 0.7, 1.0),
    //         format!("Attack from player {}", self.player_id),
    //     )),
    //     Recipients::All,
    // );

    // clear velocity of target before applying attack
    player_rigid_body.set_linvel(rapier::vector![0.0, 0.0, 0.0], true);

    player_rigid_body.apply_impulse(
        rapier::vector![
            player_state.camera_forward.x * game_config.powerup_config.dash_impulse,
            0.0,
            player_state.camera_forward.z * game_config.powerup_config.dash_impulse
        ],
        true,
    );
    super::handle_invincible_players(game_state, physics_state, player_id);

    Ok(())
}
