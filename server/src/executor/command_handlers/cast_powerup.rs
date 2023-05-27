use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::game_config::ConfigGame;
use common::core::command::Command;
use common::core::events::{GameEvent, SoundSpec};
use common::core::powerup_system::OtherEffects::Stun;
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

        let game_state_clone = game_state.clone();
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

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

        match player_state.power_up.clone() {
            Some((x, PowerUpStatus::Active)) => {
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
                }
            }
            _ => {}
        }

        let mut other_player_status_changes: Vec<(u32, StatusEffect, f32)> = vec![];

        if let Some((x, _)) = player_state.power_up.clone() {
            match x {
                PowerUp::Lightning => match game_state_clone.find_closest_player(self.player_id) {
                    Some(id) => {
                        other_player_status_changes.push((
                            id,
                            StatusEffect::Other(Stun),
                            self.game_config.powerup_config.power_up_debuff_duration,
                        ));

                        // special case
                        player_state.power_up = None;
                    }
                    _ => {
                        // TODO:
                        // cannot cast, should notify player
                        // perhaps Some Sound/UI event
                        return Ok(());
                    }
                },
                x => {
                    if x == PowerUp::Invincible {
                        super::reset_weather(physics_state, self.player_id);
                    }
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
        let player_pos = player_state.transform.translation;
        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_pos,
                "wind".to_string(),
                (self.player_id, false),
            )),
            Recipients::All,
        );
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
    _game_events: &mut dyn GameEventCollector,
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

    // when flashing, remove invisibility
    if player_state.holds_status_effect_mut(StatusEffect::Power(PowerUpEffects::Invisible)) {
        player_state
            .status_effects
            .remove(&StatusEffect::Power(PowerUpEffects::Invisible));
        player_state.power_up = None;
    }

    let _player_pos = player_state.transform.translation;

    // TODO: replace this example with actual implementation
    // game_events.add(
    //     GameEvent::SoundEvent(SoundSpec::new(
    //         player_pos,
    //         "wind".to_string(),
    //         (self.player_id, false),
    //     )),
    //     Recipients::All,
    // );
    // End TODO

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
    _game_events: &mut dyn GameEventCollector,
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

    // when dashing, remove invisibility
    if player_state.holds_status_effect_mut(StatusEffect::Power(PowerUpEffects::Invisible)) {
        player_state
            .status_effects
            .remove(&StatusEffect::Power(PowerUpEffects::Invisible));
        player_state.power_up = None;
    }

    player_state.status_effects.insert(
        StatusEffect::Other(OtherEffects::MovementDisabled),
        game_config.powerup_config.dash_blocking_duration,
    );

    let _player_pos = player_state.transform.translation;

    // TODO: replace this example with actual implementation
    // game_events.add(
    //     GameEvent::SoundEvent(SoundSpec::new(
    //         player_pos,
    //         "wind".to_string(),
    //         (self.player_id, false),
    //     )),
    //     Recipients::All,
    // );
    // End TODO

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
