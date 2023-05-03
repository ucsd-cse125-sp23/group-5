use crate::executor::GameEventCollector;
use crate::simulation::obj_collider::FromObject;
use crate::simulation::physics_state::PhysicsState;

use crate::Recipients;
use common::core::command::{Command, MoveDirection};
use common::core::events::{GameEvent, SoundSpec};

use common::core::states::{GameState, PlayerState};
use derive_more::{Constructor, Display, Error};
use nalgebra::zero;
use nalgebra::UnitQuaternion;
use nalgebra_glm::Vec3;
use nalgebra_glm as glm;
use rapier3d::geometry::InteractionGroups;
use rapier3d::prelude as rapier;
use std::fmt::Debug;

#[derive(Constructor, Error, Debug, Display)]
pub struct HandlerError {
    pub message: String,
}

type HandlerResult = Result<(), HandlerError>;

pub trait CommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult;
}

#[derive(Constructor)]
/// Handles the startup command that initializes the games state and physics world
pub struct StartupCommandHandler {
    map_obj_path: String,
}

impl CommandHandler for StartupCommandHandler {
    fn handle(
        &self,
        _: &mut GameState,
        physics_state: &mut PhysicsState,
        _game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // loading the object model
        let map = tobj::load_obj("assets/island.obj", &tobj::GPU_LOAD_OPTIONS);

        let (models, _) = map.unwrap();

        // Physics state
        let collider = rapier::ColliderBuilder::from_object_models(models)
            .translation(rapier::vector![0.0, -9.7, 0.0])
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
        _game_events: &mut dyn GameEventCollector,
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

        // if player already spawned 
        let starting_ammo = 5; 

        if let Some(player) = game_state.player_mut(self.player_id) {
            // if player died and has no spawn cooldown
            if player.is_dead && !player.on_cooldown.contains_key(&Command::Spawn) {
                // Teleport the player to the desired position.
                let new_position =
                rapier3d::prelude::Isometry::new(rapier::vector![0.0, 3.0, 0.0], zero());
                if let Some(player_rigid_body) =
                    physics_state.get_entity_rigid_body_mut(self.player_id)
                {
                    player_rigid_body.set_position(new_position, true);
                    player_rigid_body.set_linvel(rapier::vector![0.0, 0.0, 0.0], true);
                }
                player.is_dead = false;
                player.ammo_count = starting_ammo;
            }
        } else {
            let ground_groups = InteractionGroups::new(1.into(), 1.into());
            let collider = rapier::ColliderBuilder::round_cuboid(1.0, 1.0, 1.0, 0.01)
                .collision_groups(ground_groups)
                .build();
    
            let rigid_body = rapier3d::prelude::RigidBodyBuilder::dynamic()
                .translation(rapier::vector![0.0, 3.0, 0.0])
                .build();
            physics_state.insert_entity(self.player_id, Some(collider), Some(rigid_body));
    
            // Game state (needed because syncing is only for the physical properties of entities)
            game_state.players.insert(
                self.player_id,
                PlayerState {
                    id: self.player_id,
                    connected: true,
                    is_dead: false, 
                    ammo_count: starting_ammo,
                    ..Default::default()
                },
            );
        }
        Ok(())
    }
}

/*
#[derive(Constructor)]
pub struct RespawnCommandHandler {
    player_id: u32,
}

impl CommandHandler for RespawnCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // TODO: remove debug code, example usage
        // if !command_on_cooldown(game_state, self.player_id, Command::Respawn) {
        //     return Ok(());
        // }

        // if dead player is not on the respawn map, insert it into it
        if let Some(player) = game_state.players.get(&self.player_id) {
            if !player.on_cooldown.contains_key(&Command::Respawn) {
                game_state.insert_cooldown(self.player_id, Command::Respawn, 3);
            }
        }
        // Update all cooldowns in the game state
        //game_state.update_cooldowns();
        // If the player's respawn cooldown has ended, create a new RespawnCommandHandler and handle the respawn command
        if let Some(player) = game_state.players.get(&self.player_id) {
            if !player.on_cooldown.contains_key(&Command::Respawn) {

            }
        }

        Ok(())
    }
}
*/

#[derive(Constructor)]
pub struct UpdateCameraFacingCommandHandler {
    player_id: u32,
    forward: Vec3,
}

impl CommandHandler for UpdateCameraFacingCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        _: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // Game state
        let player = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

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
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // Physics state
        if self.direction.eq(&MoveDirection::zeros()) {
            return Ok(());
        }

        // TODO: remove debug code, example usage
        // if command_on_cooldown(game_state, self.player_id, Command::Move(self.direction)) {
        //     return Ok(());
        // }

        // normalize the direction vector
        let dir_vec = self.direction.normalize();

        let player_state = game_state
            .player(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

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

        // TODO: replace this example with actual implementation
        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_state.transform.translation,
                "foot_step".to_string(),
            )),
            Recipients::One(self.player_id as u8),
        );

        // TODO: remove debug code, example usage
        // game_state.insert_cooldown(self.player_id, Command::Move(self.direction), 5);

        Ok(())
    }
}

#[derive(Constructor)]
pub struct JumpCommandHandler {
    player_id: u32,
}

impl CommandHandler for JumpCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // TODO: remove debug code, example usage
        // if command_on_cooldown(game_state, self.player_id, Command::Jump) {
        //     return Ok(());
        // }

        // check if player is touching the ground
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

        let ground_collider_handle = physics_state
            .get_entity_handles(0)
            .ok_or(HandlerError::new("Ground not found".to_string()))?
            .collider
            .ok_or(HandlerError::new(
                "Ground does not have a collider".to_string(),
            ))?;

        let pair = physics_state
            .narrow_phase
            .contact_pair(player_collider_handle, ground_collider_handle);

        let mut player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if pair.is_some() {
            player_state.jump_count = 0;
        }

        const MAX_JUMP_COUNT: u32 = 2; // allow double jump

        if player_state.jump_count >= MAX_JUMP_COUNT {
            return Ok(());
        }

        player_state.jump_count += 1;

        // apply upward impulse to the player's rigid body
        const JUMP_IMPULSE: f32 = 40.0; // parameter to tune

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();
        player_rigid_body.apply_impulse(rapier::vector![0.0, JUMP_IMPULSE, 0.0], true);

        // TODO: remove debug code, example usage
        // game_state.insert_cooldown(self.player_id, Command::Jump, 5);

        Ok(())
    }
}

#[derive(Constructor)]
pub struct AttackCommandHandler {
    player_id: u32,
}

impl CommandHandler for AttackCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        
        let player_state = game_state
                .player(self.player_id)
                .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        // if attack on cooldown, do nothing for now
        if game_state.command_on_cooldown(self.player_id, Command::Attack) || player_state.ammo_count == 0 {
            return Ok(());
        }

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

        let camera_forward = Vec3::new(
            player_state.camera_forward.x,
            0.0,
            player_state.camera_forward.z,
        );

        // turn player towards attack direction (camera_forward)
        let rotation = UnitQuaternion::face_towards(&camera_forward, &Vec3::y());
        player_rigid_body.set_rotation(rotation, true);

        // loop over all other players
        for (other_player_id, other_player_state) in &game_state.players {
            if &self.player_id == other_player_id {
                continue;
            }

            // get direction from this player to other player 
            let other_player_pos = other_player_state.transform.translation;
            let vec_to_other = glm::normalize(&(other_player_pos - player_pos));
            
            // check dot product between direction to other player and attack direction
            let angle = glm::angle(&camera_forward, &vec_to_other);

            // if object in attack range
            if angle <= std::f32::consts::FRAC_PI_6 {
                // send ray to other player (may need multiple later)
                let max_toi = 5.0; // max attack distance
                let solid = true;
                let filter = rapier::QueryFilter::default().exclude_collider(player_collider_handle);

                let ray = rapier::Ray::new(rapier::point![player_pos.x, player_pos.y, player_pos.z], rapier::vector![vec_to_other.x, vec_to_other.y, vec_to_other.z]);
                if let Some((handle, toi)) = physics_state.query_pipeline.cast_ray(
                    &physics_state.bodies, &physics_state.colliders, &ray, max_toi, solid, filter
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
                        const ATTACK_IMPULSE: f32 = 40.0; // parameter to tune
                        let other_player_rigid_body = physics_state
                        .get_entity_rigid_body_mut(*other_player_id)
                        .unwrap();
                        let impulse_vec = vec_to_other * ATTACK_IMPULSE * 2.0 / toi;
                        other_player_rigid_body.apply_impulse(rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z], true);
                    }
                }
            }
            
        }
        game_state.player_mut(self.player_id).unwrap().ammo_count -= 1;

        game_state.insert_cooldown(self.player_id, Command::Attack, 5);


        Ok(())
    }
}