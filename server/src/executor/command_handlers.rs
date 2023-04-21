use crate::simulation::physics_state::PhysicsState;
use common::core::command::{Command, MoveDirection};
use common::core::states::{GameState, PlayerState};
use derive_more::{Constructor, Display, Error};
use rapier3d::parry::transformation::utils::transform;
use rapier3d::prelude as rapier;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use rapier3d::control::KinematicCharacterController;
use crate::simulation::obj_collider::FromObject;

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
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
    ) -> HandlerResult {
        // loading the object model
        let map = tobj::load_obj("assets/island.obj", &tobj::GPU_LOAD_OPTIONS);

        let (models, materials) = map.unwrap();

        // Physics state
        let collider = rapier::ColliderBuilder::from_object_models(models)
            .build();

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

        let collider = rapier::ColliderBuilder::round_cuboid( 1.0, 1.0, 1.0, 0.01)
            .build();

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
    camera_position: nalgebra_glm::Vec3,
    camera_spherical_coords: nalgebra_glm::Vec3,
}

impl CommandHandler for UpdateCameraFacingCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        _: &mut PhysicsState,
    ) -> HandlerResult {
        // Game state
        let player = game_state
            .players
            .iter_mut()
            .find(|p| p.id == self.player_id)
            .ok_or(HandlerError {
                message: "Player not found".to_string(),
            })?;

        player.camera_facing = self.camera_spherical_coords;
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
            MoveDirection::Left => rapier::vector![-1.0, 0.0, 0.0],
            MoveDirection::Right => rapier::vector![1.0, 0.0, 0.0],
        };

        let step_size = 0.1;

        physics_state.move_character_with_velocity(self.player_id, dir_vec * step_size);
        // Game state (not needed since the physics state is synced at the end of the tick)
        Ok(())
    }
}
