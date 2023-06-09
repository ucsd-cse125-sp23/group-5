use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::executor::command_handlers::jump::JumpResetCommandHandler;
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::physics_config::ConfigPhysics;
use common::core::action_states::ActionState;
use common::core::command::MoveDirection;
use common::core::events::{GameEvent, SoundSpec};
use common::core::powerup_system::{OtherEffects, StatusEffect};
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::UnitQuaternion;
use nalgebra_glm::Vec3;
use std::time::Duration;

#[derive(Constructor)]
pub struct MoveCommandHandler {
    player_id: u32,
    direction: MoveDirection,
    physics_config: ConfigPhysics,
}

impl CommandHandler for MoveCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        // Physics state
        if self.direction.eq(&MoveDirection::zeros()) {
            return Ok(());
        }

        // normalize the direction vector
        let dir_vec = self.direction.normalize();
        // let gs_clone = game_state.clone();
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        // if player is dead, don't do anything
        if player_state.is_dead {
            return Ok(());
        }

        // if player is stunned
        if player_state.holds_status_effect_mut(StatusEffect::Other(OtherEffects::Stun))
            || player_state
                .holds_status_effect_mut(StatusEffect::Other(OtherEffects::MovementDisabled))
        {
            return Ok(());
        }

        // TODO: Need to figure out how invincibility would fit in here

        // rotate the direction vector to face the camera (only take the x and z components)
        let dt = physics_state.dt();
        let camera_forward = Vec3::new(
            player_state.camera_forward.x,
            0.0,
            player_state.camera_forward.z,
        );

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();

        let rotation = UnitQuaternion::face_towards(&camera_forward, &Vec3::y());
        let dir_rotation = UnitQuaternion::face_towards(&dir_vec, &Vec3::y());

        let player_rotation = rotation * dir_rotation;

        // apply the rotation to the direction vector

        // rotate by just setting the rotation
        // player_rigid_body.set_rotation(rotation, true);

        // rotate by applying a torque impulse
        // (does not guarantee the rotation will be reached, but it will eventually converge to the desired rotation)

        // Step 1: Calculate the angular displacement required to reach the desired rotation
        let rotation_difference = player_rotation * player_rigid_body.rotation().inverse();

        // Step 2: Divide the angular displacement by dt to get the desired angular velocity
        let desired_angular_velocity = rotation_difference.scaled_axis() / dt;

        // Step 3: Calculate the difference between the current and desired angular velocities

        player_rigid_body.set_angular_damping(self.physics_config.movement_config.damping);
        let current_angular_velocity = player_rigid_body.angvel();
        let angular_velocity_difference = desired_angular_velocity - current_angular_velocity;

        // Step 4: Calculate the required torque using the gain factor
        let required_torque =
            angular_velocity_difference * self.physics_config.movement_config.gain;

        // Step 5: Apply the torque to the player's rigid body
        player_rigid_body.apply_torque_impulse(required_torque, true);

        let dir_vec = rotation * dir_vec;
        physics_state.move_character_with_velocity(
            self.player_id,
            dir_vec * self.physics_config.movement_config.step_size,
        );

        // let action_state = player_state
        //         .active_action_states
        //         .iter()
        //         .map(|(action_state, _)| action_state)
        //         .max_by_key(|action_state| action_state.priority());

        // if let Some(a_s) = action_state {
        //     let curr_time = gs_clone.life_cycle_state.unwrap_running();
        //     if *a_s == ActionState::Walking && (curr_time - player_state.last_step) > 9 {
        //         // TODO: replace this example with actual implementation
        //         game_events.add(
        //             GameEvent::SoundEvent(SoundSpec::new(
        //                 player_state.transform.translation,
        //                 "foot_step".to_string(),
        //                 (self.player_id, true),
        //                 (false, false, false),
        //                 player_state.camera_forward,
        //             )),
        //             Recipients::All, // One(self.player_id as u8),
        //         );
        //         player_state.last_step = curr_time;
        //     }
        // }

        player_state.active_action_states.insert((
            ActionState::Walking,
            Duration::from_secs_f32(self.physics_config.movement_config.walking_cooldown),
        ));

        // reset jump if player is on the ground
        JumpResetCommandHandler::new(self.player_id).handle(
            game_state,
            physics_state,
            game_events,
        )?;

        Ok(())
    }
}
