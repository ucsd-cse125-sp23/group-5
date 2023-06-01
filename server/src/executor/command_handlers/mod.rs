use std::cell::RefMut;
use std::fmt::Debug;

use derive_more::{Constructor, Display, Error};
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use rapier3d::prelude as rapier;

use common::configs::ConfigurationManager;
use common::core::events::GameEvent;
use common::core::powerup_system::OtherEffects::Stun;
use common::core::powerup_system::{PowerUp, PowerUpEffects, PowerUpStatus, StatusEffect};
use common::core::states::{calculate_distance, GameState, PlayerState};

use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;

mod area_attack;
mod attack;
mod cast_powerup;
mod cheat_code;
mod die;
mod give_powerup;
mod jump;
mod movement;
mod refill;
mod spawn;
mod startup;
mod update_camera_facing;
mod weather;

mod cheat_code_control;
pub mod prelude;
mod weather_cheat_key;

#[derive(Constructor, Error, Debug, Display)]
pub struct HandlerError {
    pub message: String,
}

pub type HandlerResult = Result<(), HandlerError>;

pub trait CommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult;
}

type GameEventWithRecipients = (GameEvent, Recipients);

pub trait GameEventCollector {
    fn add(&mut self, event: GameEvent, recipients: Recipients);
}

impl GameEventCollector for RefMut<'_, Vec<GameEventWithRecipients>> {
    fn add(&mut self, event: GameEvent, recipients: Recipients) {
        self.push((event, recipients));
    }
}

pub fn handle_invincible_players(
    game_state: &mut GameState,
    physics_state: &mut PhysicsState,
    command_casting_player_id: u32,
) {
    if game_state.players.get(&command_casting_player_id).is_none() {
        return;
    }
    if !game_state
        .players
        .get(&command_casting_player_id)
        .unwrap()
        .holds_status_effect(StatusEffect::Power(PowerUpEffects::Invincible))
    {
        return;
    }
    let config_instance = ConfigurationManager::get_configuration();
    let game_config = config_instance.game.clone();
    let game_state_clone = game_state.clone();
    for (id, player_state) in game_state.players.iter_mut() {
        if player_state.holds_status_effect(StatusEffect::Power(PowerUpEffects::Invincible)) {
            for (other_player_id, other_player_state) in game_state_clone.players.iter() {
                if !other_player_state
                    .holds_status_effect(StatusEffect::Power(PowerUpEffects::Invincible))
                    && *other_player_id != *id
                    && calculate_distance(
                        player_state.transform.translation,
                        other_player_state.transform.translation,
                    ) < game_config.powerup_config.invincible_effective_distance
                {
                    // get launched
                    let player_pos = player_state.transform.translation;

                    // Bling bling sound?
                    // TODO: replace this example with actual implementation of collision
                    // game_events.add(
                    //     GameEvent::SoundEvent(SoundSpec::new(
                    //         player_pos,
                    //         "wind".to_string(),
                    //         (self.player_id, false),
                    //     )),
                    //     Recipients::All,
                    // );

                    let _player_collider_handle = physics_state
                        .get_entity_handles(command_casting_player_id)
                        .ok_or(HandlerError::new(format!(
                            "Player {} not found",
                            command_casting_player_id
                        )))
                        .unwrap()
                        .collider
                        .ok_or(HandlerError::new(format!(
                            "Player {} does not have a collider",
                            command_casting_player_id
                        )))
                        .unwrap();

                    let _player_rigid_body = physics_state
                        .get_entity_rigid_body_mut(command_casting_player_id)
                        .unwrap();

                    let _camera_forward = Vec3::new(
                        player_state.camera_forward.x,
                        0.0,
                        player_state.camera_forward.z,
                    );

                    // collision/launch sound
                    // game_events.add(
                    //     GameEvent::ParticleEvent(ParticleSpec::new(
                    //         ParticleType::ATTACK,
                    //         player_pos.clone(),
                    //         camera_forward.clone(),
                    //         //TODO: placeholder for player color
                    //         glm::vec3(0.0, 1.0, 0.0),
                    //         glm::vec4(0.4, 0.9, 0.7, 1.0),
                    //         format!("Attack from player {}", self.player_id),
                    //     )),
                    //     Recipients::All,
                    // );

                    // get direction from this player to other player
                    let other_player_pos = other_player_state.transform.translation;
                    let vec_to_other = glm::normalize(&(other_player_pos - player_pos));

                    // check dot product between direction to other player and attack direction

                    // if object in attack range
                    let _other_player_collider_handle = physics_state
                        .get_entity_handles(*other_player_id)
                        .ok_or(HandlerError::new(format!(
                            "Player {} not found",
                            command_casting_player_id
                        )))
                        .unwrap()
                        .collider
                        .ok_or(HandlerError::new(format!(
                            "Player {} does not have a collider",
                            command_casting_player_id
                        )))
                        .unwrap();

                    let other_player_rigid_body = physics_state
                        .get_entity_rigid_body_mut(*other_player_id)
                        .unwrap();
                    let impulse_vec =
                        vec_to_other * game_config.powerup_config.invincible_effective_impulse;
                    other_player_rigid_body.apply_impulse(
                        rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z],
                        true,
                    );
                }
            }
        }
    }
}

pub fn reset_weather(physics_state: &mut PhysicsState, player_id: u32) {
    physics_state
        .get_entity_collider_mut(player_id)
        .unwrap()
        .set_friction(1.0);

    let body = physics_state.get_entity_rigid_body_mut(player_id).unwrap();

    body.reset_forces(false);
    body.set_linear_damping(0.5);
}

pub fn apply_stun(player_state: &mut PlayerState, duration: f32) {
    player_state.status_effects.insert(
        StatusEffect::Other(Stun),
        duration.max(
            *player_state
                .status_effects
                .get(&StatusEffect::Other(Stun))
                .unwrap_or(&0.0),
        ),
    );
}

pub fn remove_invisibility(player_state: &mut PlayerState) {
    if player_state.holds_status_effect_mut(StatusEffect::Power(PowerUpEffects::Invisible)) {
        player_state
            .status_effects
            .remove(&StatusEffect::Power(PowerUpEffects::Invisible));
        if let Some((PowerUp::Invisible, PowerUpStatus::Active)) = player_state.power_up.clone() {
            player_state.power_up = None;
        }
    }
}
