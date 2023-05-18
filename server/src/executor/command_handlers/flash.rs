use crate::simulation::physics_state::PhysicsState;
use common::configs::parameters::{FLASH_DISTANCE_SCALAR, SPECIAL_MOVEMENT_COOLDOWN};
use common::core::command::Command;
use common::core::powerup_system::StatusEffect;
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::{zero, UnitQuaternion};
use nalgebra_glm::Vec3;
use rapier3d::math::Isometry;

use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};

#[derive(Constructor)]
pub struct FlashCommandHandler {
    player_id: u32,
}

impl CommandHandler for FlashCommandHandler {
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
        if player_state.command_on_cooldown(Command::Flash)
            || !player_state
                .status_effects
                .contains_key(&StatusEffect::EnabledFlash)
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

        // turn player towards attack direction (camera_forward)
        let rotation = UnitQuaternion::face_towards(&camera_forward, &Vec3::y());
        player_rigid_body.set_rotation(rotation, true);

        player_state.insert_cooldown(Command::Flash, SPECIAL_MOVEMENT_COOLDOWN);

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
            .player_mut(self.player_id)
            .unwrap()
            .transform
            .translation;

        new_coordinates.x += FLASH_DISTANCE_SCALAR * x_dir;
        new_coordinates.z += FLASH_DISTANCE_SCALAR * z_dir;

        let new_position = Isometry::new(new_coordinates, zero());
        player_rigid_body.set_position(new_position, true);

        super::handle_invincible_players(game_state, physics_state, self.player_id);

        Ok(())
    }
}
