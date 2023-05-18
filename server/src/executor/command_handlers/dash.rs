use common::configs::parameters::{DASH_IMPULSE, SPECIAL_MOVEMENT_COOLDOWN};
use common::core::command::Command;
use common::core::powerup_system::StatusEffect;
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::UnitQuaternion;
use nalgebra_glm::Vec3;

use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;

extern crate nalgebra_glm as glm;

use rapier3d::prelude as rapier;

#[derive(Constructor)]
pub struct DashCommandHandler {
    player_id: u32,
}

impl CommandHandler for DashCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        // if dash on cooldown, or should not be able to dash, do nothing for now
        if player_state.command_on_cooldown(Command::Dash)
            || !player_state
                .status_effects
                .contains_key(&StatusEffect::EnabledDash)
        {
            return Ok(());
        }

        player_state.status_effects.remove(&StatusEffect::Invisible);

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

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();

        let camera_forward = Vec3::new(
            player_state.camera_forward.x,
            0.0,
            player_state.camera_forward.z,
        );

        // turn player towards dash direction (camera_forward)
        let rotation = UnitQuaternion::face_towards(&camera_forward, &Vec3::y());
        player_rigid_body.set_rotation(rotation, true);

        player_state.insert_cooldown(Command::Dash, SPECIAL_MOVEMENT_COOLDOWN);

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

        player_rigid_body.apply_impulse(
            rapier::vector![
                player_state.camera_forward.x * DASH_IMPULSE,
                0.0,
                player_state.camera_forward.z * DASH_IMPULSE
            ],
            true,
        );
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        Ok(())
    }
}
