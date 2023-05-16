use std::f32::consts::PI;
use std::fmt::{format, Debug};

use derive_more::{Constructor, Display, Error};
use itertools::Itertools;
use nalgebra::UnitQuaternion;
use nalgebra::{zero, Isometry3, Vector3};
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use rapier3d::geometry::InteractionGroups;
use rapier3d::math::Isometry;
use rapier3d::prelude as rapier;

use common::configs::constants::{
    DASH_IMPULSE, FLASH_DISTANCE_SCALAR, INVINCIBLE_EFFECTIVE_DISTANCE,
    INVINCIBLE_EFFECTIVE_IMPULSE, MAX_WIND_CHARGE, ONE_CHARGE, POWER_UP_BUFF_DURATION,
    POWER_UP_COOLDOWN, POWER_UP_DEBUFF_DURATION, WIND_ENHANCEMENT_SCALAR,
};
use common::configs::model_config::ConfigModels;
use common::configs::player_config::ConfigPlayer;
use common::configs::scene_config::ConfigSceneGraph;
use common::core::command::{Command, MoveDirection};
use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::powerup_system::{PowerUp, StatusEffect, POWER_UP_TO_EFFECT_MAP};
use common::core::states::{calculate_distance, GameState, PlayerState};

use crate::executor::GameEventCollector;
use crate::simulation::obj_collider::FromObject;
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;

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
    config_models: ConfigModels,
    config_scene_graph: ConfigSceneGraph,
}

impl CommandHandler for StartupCommandHandler {
    fn handle(
        &self,
        _: &mut GameState,
        physics_state: &mut PhysicsState,
        _game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let mut scene_entity_id = 0xBEEF; // TODO: set up a better convention for scene entities

        let mut nodes = self
            .config_scene_graph
            .nodes
            .iter()
            .map(|n| (n.clone(), Isometry::identity()))
            .collect_vec();

        while !nodes.is_empty() {
            let (node, parent_transform) = nodes.pop().unwrap();
            let model = node.model.clone().ok_or(HandlerError::new(
                "Config node not attaching model".to_string(),
            ))?;

            let model_config = self
                .config_models
                .model(model)
                .ok_or(HandlerError::new("Model not declared".to_string()))?
                .path
                .clone();

            let (models, _) = tobj::load_obj(model_config, &tobj::GPU_LOAD_OPTIONS)
                .map_err(|e| HandlerError::new(format!("Error loading model {:?}", e)))?;

            let local_transform = Isometry3::from_parts(
                node.transform.position.into(),
                UnitQuaternion::from_quaternion(node.transform.rotation),
            );

            let world_transform = parent_transform * local_transform;

            let body = rapier::RigidBodyBuilder::fixed()
                .position(world_transform)
                .build();

            let decompose = node.decompose.unwrap_or(false);

            let collider = rapier::ColliderBuilder::from_object_models(models, decompose).build();

            physics_state.insert_entity(scene_entity_id, Some(collider), Some(body)); // insert the collider into the physics world
            scene_entity_id += 1;

            // add children to nodes
            if let Some(children) = node.children.clone() {
                nodes.extend(
                    children
                        .iter()
                        .map(|child| (child.clone(), world_transform)),
                );
            }
        }

        Ok(())
    }
}

#[derive(Constructor)]
pub struct SpawnCommandHandler {
    player_id: u32,
    config_player: ConfigPlayer,
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
        if let Some(player) = game_state.player_mut(self.player_id) {
            // if player died and has no spawn cooldown
            if player.is_dead && !player.on_cooldown.contains_key(&Command::Spawn) {
                if let Some(player_rigid_body) =
                    physics_state.get_entity_rigid_body_mut(self.player_id)
                {
                    player_rigid_body.set_enabled(true);
                }

                player.is_dead = false;
                player.refill_wind_charge(Some(MAX_WIND_CHARGE));
            }
        } else {
            // get spawn-locations with corresponding id
            let spawn_position = self.config_player.spawn_points[self.player_id as usize - 1];
            let ground_groups = InteractionGroups::new(1.into(), 1.into());
            let collider = rapier::ColliderBuilder::cuboid(1.0, 1.0, 1.0)
                .collision_groups(ground_groups)
                .build();

            let rigid_body = rapier3d::prelude::RigidBodyBuilder::dynamic()
                .translation(spawn_position)
                .ccd_enabled(true)
                .build();

            physics_state.insert_entity(self.player_id, Some(collider), Some(rigid_body));

            // Game state (needed because syncing is only for the physical properties of entities)
            game_state.players.insert(
                self.player_id,
                PlayerState {
                    id: self.player_id,
                    connected: true,
                    is_dead: false,
                    wind_charge: MAX_WIND_CHARGE,
                    on_flag_time: 0.0,
                    spawn_point: spawn_position,
                    power_up: None,
                    ..Default::default()
                },
            );
        }
        Ok(())
    }
}

#[derive(Constructor)]
pub struct DieCommandHandler {
    player_id: u32,
}

impl CommandHandler for DieCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        player_state.reset_status_effects();

        let spawn_position = player_state.spawn_point;

        // Teleport the player back to their spawn position and disable physics.
        let new_position = rapier3d::prelude::Isometry::new(spawn_position, zero());
        if let Some(player_rigid_body) = physics_state.get_entity_rigid_body_mut(self.player_id) {
            player_rigid_body.set_position(new_position, true);
            player_rigid_body.set_linvel(rapier::vector![0.0, 0.0, 0.0], true);
            player_rigid_body.set_enabled(false);
        }

        player_state.is_dead = true;
        player_state.insert_cooldown(Command::Spawn, 3.0);

        Ok(())
    }
}

#[derive(Constructor)]
pub struct UpdateCameraFacingCommandHandler {
    player_id: u32,
    forward: Vec3,
}

impl CommandHandler for UpdateCameraFacingCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        handle_invincible_players(game_state, physics_state, self.player_id);

        // Game state
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        player_state.camera_forward = self.forward;

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
        handle_invincible_players(game_state, physics_state, self.player_id);

        // Physics state
        if self.direction.eq(&MoveDirection::zeros()) {
            return Ok(());
        }

        // normalize the direction vector
        let dir_vec = self.direction.normalize();

        let player_state = game_state
            .player(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
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
                (self.player_id, true),
            )),
            Recipients::One(self.player_id as u8),
        );

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
        let player_collider_handle = physics_state
            .get_entity_handles(self.player_id)
            .ok_or(HandlerError::new(format!(
                "Handlers for player {} not found",
                self.player_id
            )))?
            .collider
            .ok_or(HandlerError::new(format!(
                "Collider for player {} not found",
                self.player_id
            )))?;

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .ok_or(HandlerError::new(format!(
                "Rigid body for player {} not found",
                self.player_id
            )))?;

        let contact_pairs = physics_state
            .narrow_phase
            .contacts_with(player_collider_handle)
            .collect_vec();

        let mut should_reset_jump = false;
        for contact_pair in contact_pairs {
            if let Some((manifold, _)) = contact_pair.find_deepest_contact() {
                // see if player is above another collider by testing the normal angle
                if nalgebra_glm::angle(&manifold.data.normal, &Vector3::y()) < PI / 3. {
                    should_reset_jump = true;
                }
            }
        }

        let mut player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        if should_reset_jump {
            player_state.jump_count = 0;
        }

        const MAX_JUMP_COUNT: u32 = 2; // allow double jump
        let jump_limit = if player_state
            .status_effects
            .contains_key(&StatusEffect::TripleJump)
        {
            MAX_JUMP_COUNT + 1
        } else {
            MAX_JUMP_COUNT
        };

        if player_state.jump_count >= jump_limit {
            return Ok(());
        }

        player_state.jump_count += 1;

        // apply upward impulse to the player's rigid body
        const JUMP_IMPULSE: f32 = 40.0; // parameter to tune

        let player_rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();
        player_rigid_body.apply_impulse(rapier::vector![0.0, JUMP_IMPULSE, 0.0], true);

        handle_invincible_players(game_state, physics_state, self.player_id);

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
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        handle_invincible_players(game_state, physics_state, self.player_id);

        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        // if attack on cooldown, or cannot consume charge, do nothing for now
        if player_state.command_on_cooldown(Command::Attack)
            || !player_state.try_consume_wind_charge(None)
        {
            return Ok(());
        }

        player_state.status_effects.remove(&StatusEffect::Invisible);

        let player_pos = player_state.transform.translation;

        // TODO: replace this example with actual implementation
        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_pos,
                "wind".to_string(),
                (self.player_id, false),
            )),
            Recipients::All,
        );

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

        player_state.insert_cooldown(Command::Attack, 1.0);
        game_events.add(
            GameEvent::ParticleEvent(ParticleSpec::new(
                ParticleType::ATTACK,
                player_pos.clone(),
                camera_forward.clone(),
                //TODO: placeholder for player color
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(0.4, 0.9, 0.7, 1.0),
                format!("Attack from player {}", self.player_id),
            )),
            Recipients::All,
        );
        let wind_enhanced = player_state
            .status_effects
            .contains_key(&StatusEffect::EnhancedWind);
        let scalar = if wind_enhanced {
            WIND_ENHANCEMENT_SCALAR
        } else {
            1.0
        };

        // loop over all other players
        for (other_player_id, other_player_state) in game_state.players.iter() {
            if &self.player_id == other_player_id {
                continue;
            }

            if game_state
                .player(*other_player_id)
                .unwrap()
                .status_effects
                .contains_key(&StatusEffect::Invincible)
            {
                continue;
            }

            // get direction from this player to other player
            let other_player_pos = other_player_state.transform.translation;
            let vec_to_other = glm::normalize(&(other_player_pos - player_pos));

            // check dot product between direction to other player and attack direction
            let angle = glm::angle(&camera_forward, &vec_to_other);

            // if object in attack range
            if angle <= std::f32::consts::FRAC_PI_6 * scalar {
                // send ray to other player (may need multiple later)
                let max_toi = 5.0 * scalar; // max attack distance
                let solid = true;
                let filter =
                    rapier::QueryFilter::default().exclude_collider(player_collider_handle);

                let ray = rapier::Ray::new(
                    rapier::point![player_pos.x, player_pos.y, player_pos.z],
                    rapier::vector![vec_to_other.x, vec_to_other.y, vec_to_other.z],
                );
                if let Some((handle, toi)) = physics_state.query_pipeline.cast_ray(
                    &physics_state.bodies,
                    &physics_state.colliders,
                    &ray,
                    max_toi,
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
                        const ATTACK_IMPULSE: f32 = 40.0; // parameter to tune
                        let other_player_rigid_body = physics_state
                            .get_entity_rigid_body_mut(*other_player_id)
                            .unwrap();
                        let impulse_vec = scalar * vec_to_other * ATTACK_IMPULSE * 2.0 / toi;
                        other_player_rigid_body.apply_impulse(
                            rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z],
                            true,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Constructor)]
pub struct RefillCommandHandler {
    player_id: u32,
}

impl CommandHandler for RefillCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        handle_invincible_players(game_state, physics_state, self.player_id);

        let spawn_position = game_state.player(self.player_id).unwrap().spawn_point;
        let player_state = game_state.player_mut(self.player_id).unwrap();

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        }

        if !player_state.is_in_circular_area(
            (spawn_position.x, spawn_position.z),
            2.0,
            (None, None),
        ) || player_state.command_on_cooldown(Command::Refill)
        {
            // signal player that he/she is not in refill area
            return Ok(());
        }
        player_state.refill_wind_charge(Some(ONE_CHARGE));
        player_state.insert_cooldown(Command::Refill, 0.5);
        Ok(())
    }
}

#[derive(Constructor)]
pub struct CastPowerUpCommandHandler {
    player_id: u32,
}

impl CommandHandler for CastPowerUpCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        handle_invincible_players(game_state, physics_state, self.player_id);

        let game_state_clone = game_state.clone();
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Stun)
        {
            return Ok(());
        } // Maybe Add Cleanse?

        // if powerup is on cooldown, or does not have a powerup, return
        if player_state.command_on_cooldown(Command::CastPowerUp) || player_state.power_up.is_none()
        {
            return Ok(());
        }

        let mut other_player_status_changes: Vec<(u32, StatusEffect, f32)> = vec![];

        match player_state.power_up.clone() {
            Some(x) => match x {
                PowerUp::Lightning => match game_state_clone.find_closest_player(self.player_id) {
                    Some(id) => {
                        other_player_status_changes.push((
                            id,
                            StatusEffect::Stun,
                            POWER_UP_DEBUFF_DURATION,
                        ));
                    }
                    _ => {
                        // TODO:
                        // cannot cast, should notify player
                        // perhaps Some Sound/UI event
                        return Ok(());
                    }
                },
                x => {
                    player_state.status_effects.insert(
                        POWER_UP_TO_EFFECT_MAP.get(&(x.value())).unwrap().clone(),
                        POWER_UP_BUFF_DURATION,
                    );
                }
            },
            None => {}
        };

        // by now the player should have casted the powerup successfully, resetting player powerup states
        player_state.power_up = None;
        player_state.insert_cooldown(Command::CastPowerUp, POWER_UP_COOLDOWN);

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
                .contains_key(&StatusEffect::Invincible)
            {
                other_player_state.status_effects.insert(*effect, *duration);
            }
        }

        Ok(())
    }
}

#[derive(Constructor)]
pub struct DashCommandHandler {
    player_id: u32,
}

impl CommandHandler for DashCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
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

        let player_pos = player_state.transform.translation;

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

        player_state.insert_cooldown(Command::Dash, 0.5);

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
        handle_invincible_players(game_state, physics_state, self.player_id);

        Ok(())
    }
}

#[derive(Constructor)]
pub struct FlashCommandHandler {
    player_id: u32,
}

impl CommandHandler for FlashCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
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

        let player_pos = player_state.transform.translation;

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

        player_state.insert_cooldown(Command::Flash, 0.5);

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
            .translation
            .clone();

        new_coordinates.x += FLASH_DISTANCE_SCALAR * x_dir;
        new_coordinates.z += FLASH_DISTANCE_SCALAR * z_dir;

        let new_position = Isometry::new(new_coordinates, zero());
        player_rigid_body.set_position(new_position, true);

        handle_invincible_players(game_state, physics_state, self.player_id);

        Ok(())
    }
}

fn handle_invincible_players(
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
        .status_effects
        .contains_key(&StatusEffect::Invincible)
    {
        return;
    }
    let game_state_clone = game_state.clone();
    for (id, player_state) in game_state.players.iter_mut() {
        if player_state
            .status_effects
            .contains_key(&StatusEffect::Invincible)
        {
            for (other_player_id, other_player_state) in game_state_clone.players.iter() {
                if !other_player_state
                    .status_effects
                    .contains_key(&StatusEffect::Invisible)
                    && *other_player_id != *id
                    && calculate_distance(
                        player_state.transform.translation,
                        other_player_state.transform.translation,
                    ) < INVINCIBLE_EFFECTIVE_DISTANCE
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

                    let player_collider_handle = physics_state
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

                    let player_rigid_body = physics_state
                        .get_entity_rigid_body_mut(command_casting_player_id)
                        .unwrap();

                    let camera_forward = Vec3::new(
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
                    let other_player_collider_handle = physics_state
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
                    let impulse_vec = vec_to_other * INVINCIBLE_EFFECTIVE_IMPULSE * 2.0;
                    other_player_rigid_body.apply_impulse(
                        rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z],
                        true,
                    );
                }
            }
        }
    }
}
