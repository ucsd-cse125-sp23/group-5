use crate::simulation::physics_state::PhysicsState;
use common::core::command::{Command, MoveDirection};
use common::core::states::{GameState, PlayerState};
use derive_more::{Constructor, Display, Error};
use rapier3d::parry::transformation::utils::transform;
use rapier3d::prelude as rapier;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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
        let collider = rapier::ColliderBuilder::capsule_y(0.5, 0.5)
            .translation(rapier::vector![0.0, 0.0, 0.0])
            .build();

        let rigid_body = rapier3d::prelude::RigidBodyBuilder::dynamic()
            .translation(rapier::vector![0.0, 0.0, 0.0])
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
        let delta_vec = match self.direction {
            MoveDirection::Forward => rapier::vector![0.0, 0.0, 1.0],
            MoveDirection::Backward => rapier::vector![0.0, 0.0, -1.0],
            MoveDirection::Left => rapier::vector![-1.0, 0.0, 0.0],
            MoveDirection::Right => rapier::vector![1.0, 0.0, 0.0],
        };
        let rigid_body = physics_state
            .get_entity_rigid_body_mut(self.player_id)
            .unwrap();
        rigid_body.set_linvel(delta_vec, true);

        // Game state (not needed since the physics state is synced at the end of the tick)
        Ok(())
    }
}
