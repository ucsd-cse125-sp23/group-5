use crate::simulation::obj_collider::FromObject;
use crate::simulation::physics_state::PhysicsState;
use common::core::command::MoveDirection;
use common::core::states::{GameState, PlayerState};
use derive_more::{Constructor, Display, Error};
use nalgebra::UnitQuaternion;
use nalgebra_glm::Vec3;
use rapier3d::prelude as rapier;
use std::fmt::Debug;

#[derive(Error, Debug, Display)]
pub struct HandlerError {
    pub message: String,
}

type HandlerResult = Result<(), HandlerError>;

pub trait CommandHandler {
    fn handle(&self, game_state: &mut GameState, physics_state: &mut PhysicsState)
        -> HandlerResult;
}

#[derive(Constructor)]
/// Handles the startup command that initializes the games state and physics world
pub struct StartupCommandHandler {
    map_obj_path: String,
}

impl CommandHandler for StartupCommandHandler {
    fn handle(&self, _: &mut GameState, physics_state: &mut PhysicsState) -> HandlerResult {
        // loading the object model
        let map = tobj::load_obj("assets/island.obj", &tobj::GPU_LOAD_OPTIONS);

        let (models, _) = map.unwrap();

        // Physics state
        let collider = rapier::ColliderBuilder::from_object_models(models).translation(rapier::vector![0.0, -9.7, 0.0]).build();

        physics_state.insert_entity(0, Some(collider), None); // insert the collider into the physics world
        Ok(())
    }
}

#[derive(Constructor)]
pub struct SpawnCommandHandler {
    player_id: u32,
}

impl CommandHandler for SpawnCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
    ) -> HandlerResult {
        // Physics state

        // let player_model = tobj::load_obj("assets/cube.obj", &tobj::GPU_LOAD_OPTIONS);
        //
        // let (models, materials) = player_model.unwrap();
        //
        // // Physics state
        // let collider = rapier::ColliderBuilder::from_object_models(models)
        //     .translation(rapier::vector![0.0, 2.0, 0.0])
        //     .build();

        if physics_state.get_entity_handles(self.player_id).is_some() {
            return Err(HandlerError {
                message: "Player already spawned".to_string(),
            });
        }

        let collider = rapier::ColliderBuilder::round_cuboid(1.0, 1.0, 1.0, 0.01).build();

        let rigid_body = rapier3d::prelude::RigidBodyBuilder::dynamic()
            .translation(rapier::vector![0.0, 4.0, 0.0])
            .build();
        physics_state.insert_entity(self.player_id, Some(collider), Some(rigid_body));

        // Game state (needed because syncing is only for the physical properties of entities)
        game_state.players.push(PlayerState {
            id: self.player_id,
            ..Default::default()
        });
        Ok(())
    }
}

#[derive(Constructor)]
pub struct UpdateCameraFacingCommandHandler {
    player_id: u32,
    forward: Vec3,
}

impl CommandHandler for UpdateCameraFacingCommandHandler {
    fn handle(&self, game_state: &mut GameState, _: &mut PhysicsState) -> HandlerResult {
        // Game state
        let player = game_state
            .players
            .iter_mut()
            .find(|p| p.id == self.player_id)
            .ok_or(HandlerError {
                message: "Player not found".to_string(),
            })?;

        player.camera_forward = self.forward;
        Ok(())
    }
}

#[derive(Constructor)]
pub struct MoveCommandHandler {
    player_id: u32,
    direction: MoveDirection,
}

impl CommandHandler for MoveCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
    ) -> HandlerResult {
        // Physics state
        let dir_vec = match self.direction {
            MoveDirection::Forward => rapier::vector![0.0, 0.0, 1.0],
            MoveDirection::Backward => rapier::vector![0.0, 0.0, -1.0],
            MoveDirection::Left => rapier::vector![1.0, 0.0, 0.0],
            MoveDirection::Right => rapier::vector![-1.0, 0.0, 0.0],
        };

        let player_state = game_state
            .players
            .iter_mut()
            .find(|p| p.id == self.player_id)
            .ok_or(HandlerError {
                message: "Player not found".to_string(),
            })?;

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
        let angle = rotation_difference.angle();
        let axis = rotation_difference.axis().unwrap();

        // Step 2: Divide the angular displacement by dt to get the desired angular velocity
        let desired_angular_velocity = axis.scale(angle) / dt;

        // Step 3: Calculate the difference between the current and desired angular velocities

        // rotation parameters to tune (balance them to get the best results)
        const DAMPING: f32 = 10.0;
        const GAIN: f32 = 0.8;

        player_rigid_body.set_angular_damping(DAMPING);
        let current_angular_velocity = player_rigid_body.angvel();
        let angular_velocity_difference = desired_angular_velocity - current_angular_velocity;

        // Step 4: Calculate the required torque using the gain factor
        let required_torque = angular_velocity_difference * GAIN;

        // Step 5: Apply the torque to the player's rigid body
        player_rigid_body.apply_torque_impulse(required_torque, true);

        // movement parameter
        const STEP_SIZE: f32 = 0.1;

        let dir_vec = rotation * dir_vec;
        physics_state.move_character_with_velocity(self.player_id, dir_vec * STEP_SIZE);
        // Game state (not needed since the physics state is synced at the end of the tick)
        Ok(())
    }
}

#[derive(Constructor)]
pub struct JumpCommandHandler {
    player_id: u32,
}

impl CommandHandler for JumpCommandHandler {
    fn handle(&self, _: &mut GameState, physics_state: &mut PhysicsState) -> HandlerResult {
        // apply upward impulse to the player's rigid body
        const JUMP_IMPULSE: f32 = 20.0; // parameter to tune

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();
        player_rigid_body.apply_impulse(rapier::vector![0.0, JUMP_IMPULSE, 0.0], true);

        Ok(())
    }
}
