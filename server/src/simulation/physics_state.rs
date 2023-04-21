use crate::simulation::entity::{Entity, EntityHandles};
use rapier3d::data::Index;
use rapier3d::parry::utils::hashmap::HashMap;
use rapier3d::prelude::*;
use std::collections::BTreeMap;
use nalgebra_glm::Vec3;
use rapier3d::control::KinematicCharacterController;

#[derive(Default)]
pub struct PhysicsState {
    pub physics_pipeline: PhysicsPipeline,
    pub islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub integration_parameters: IntegrationParameters,
    pub gravity: Vector<f32>,
    pub entity_indices: HashMap<Entity, EntityHandles>,
    pub character_controller: KinematicCharacterController,
}

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            gravity: Vector::y() * -9.81,
            ..Default::default()
        }
    }

    pub fn set_delta_time(&mut self, time_step: f32) {
        self.integration_parameters.dt = time_step;
    }

    pub fn dt(&self) -> f32 {
        self.integration_parameters.dt
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        )
    }

    pub fn insert_entity(
        &mut self,
        entity: Entity,
        collider: Option<Collider>,
        rigid_body: Option<RigidBody>,
    ) -> Option<EntityHandles> {
        let (rigid_body_handle, collider_handle) = match (collider, rigid_body) {
            // if both are provided, insert the rigid body first, then the collider (with parent rigid body)
            (Some(collider), Some(rigid_body)) => {
                let rigid_body_handle = self.bodies.insert(rigid_body);
                let collider_handle = self.colliders.insert_with_parent(
                    collider,
                    rigid_body_handle,
                    &mut self.bodies,
                );
                (Some(rigid_body_handle), (Some(collider_handle)))
            }
            (Some(collider), None) => {
                // if only collider is provided, insert it
                let collider_handle = self.colliders.insert(collider);
                (None, Some(collider_handle))
            }
            (None, Some(rigid_body)) => {
                // if only rigid body is provided, insert it
                let rigid_body_handle = self.bodies.insert(rigid_body);
                (Some(rigid_body_handle), None)
            }
            _ => return None, // if neither are provided, return None
        };
        self.entity_indices.insert(
            entity,
            EntityHandles {
                rigid_body: rigid_body_handle,
                collider: collider_handle,
            },
        )
    }

    pub fn get_entity_handles(&self, entity: Entity) -> Option<&EntityHandles> {
        self.entity_indices.get(&entity)
    }

    pub fn get_entity_handles_mut(&mut self, entity: Entity) -> Option<&mut EntityHandles> {
        self.entity_indices.get_mut(&entity)
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<EntityHandles> {
        let entity_handles = self.entity_indices.remove(&entity)?;
        // remove rigid body and collider
        if let Some(rigid_body_handle) = entity_handles.rigid_body {
            self.bodies.remove(
                rigid_body_handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.joints,
                &mut self.multibody_joints,
                true,
            );
        }
        // else remove collider
        else if let Some(collider_handle) = entity_handles.collider {
            self.colliders
                .remove(collider_handle, &mut self.islands, &mut self.bodies, true);
        }
        Some(entity_handles)
    }

    pub fn get_entity_rigid_body(&self, entity: Entity) -> Option<&RigidBody> {
        self.bodies
            .get(self.get_entity_handles(entity)?.rigid_body?)
    }

    pub fn get_entity_rigid_body_mut(&mut self, entity: Entity) -> Option<&mut RigidBody> {
        self.bodies
            .get_mut(self.get_entity_handles(entity)?.rigid_body?)
    }

    pub fn get_entity_collider(&self, entity: Entity) -> Option<&Collider> {
        self.colliders
            .get(self.get_entity_handles(entity)?.collider?)
    }

    pub fn get_entity_collider_mut(&mut self, entity: Entity) -> Option<&mut Collider> {
        self.colliders
            .get_mut(self.get_entity_handles(entity)?.collider?)
    }


    pub fn move_character_with_velocity(&mut self, entity: Entity, desired_translation: Vec3) {
        let character_shape = self.get_entity_collider(entity).unwrap().shape();
        let character_handle = self.get_entity_handles(entity).unwrap().rigid_body.unwrap();
        let character_pos = self.get_entity_rigid_body(entity).unwrap().position();

        let dt = self.integration_parameters.dt;
        // Calculate the possible movement.
        let corrected_movement = self.character_controller.move_shape(
            dt,                 // The timestep length
            &self.bodies,                                   // The RigidBodySet.
            &self.colliders,                                // The ColliderSet.
            &self.query_pipeline,                   // The QueryPipeline.
            character_shape,                                // The character’s shape.
            character_pos,                                  // The character’s initial position.
            desired_translation,
            QueryFilter::default()
                // Make sure the the character we are trying to move isn’t considered an obstacle.
                .exclude_rigid_body(character_handle),
            |_| {},
        );

        let character_body = self.get_entity_rigid_body_mut(entity).unwrap();
        // set its velocity to the computed movement divided by the timestep length.
        character_body.set_linvel(corrected_movement.translation / dt, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[test]
    fn test_physics_state() {
        let mut physics_state = PhysicsState::new();
        physics_state.step();
    }

    #[test]
    fn test_adding_bodies_and_colliders() {
        let mut physics_state = PhysicsState::new();

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
        physics_state.insert_entity(0, Some(collider), None);

        /* Create the bounding ball. */
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 10.0, 0.0])
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();

        physics_state.insert_entity(1, Some(collider), Some(rigid_body));

        let tick_duration = 30; // 30ms per tick
        let mut last_instant = Instant::now();
        for _ in 0..100 {
            physics_state.set_delta_time((Instant::now() - last_instant).as_secs_f32());
            last_instant = Instant::now();
            physics_state.step();
            sleep(Duration::from_millis(tick_duration));
        }
        let ball_pos = physics_state
            .get_entity_rigid_body(1)
            .unwrap()
            .position()
            .translation
            .y;
        // should be around 0.59585696 += epsilon
        assert!(ball_pos > 0.5 && ball_pos < 0.7);
    }
}
