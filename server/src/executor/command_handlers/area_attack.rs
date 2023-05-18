use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::parameters::{
    AREA_ATTACK_COEFF, AREA_ATTACK_COOLDOWN, AREA_ATTACK_COST, AREA_ATTACK_IMPULSE,
    MAX_AREA_ATTACK_DIST,
};
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType};
use common::core::states::GameState;
use derive_more::Constructor;
use rapier3d::{geometry, pipeline};

extern crate nalgebra_glm as glm;
use rapier3d::prelude as rapier;

#[derive(Constructor)]
pub struct AreaAttackCommandHandler {
    player_id: u32,
}

impl CommandHandler for AreaAttackCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        // if attack on cooldown, or cannot consume charge, do nothing for now
        if player_state.command_on_cooldown(Command::AreaAttack)
            || !player_state.try_consume_wind_charge(Some(AREA_ATTACK_COST))
        {
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

        player_state.insert_cooldown(Command::AreaAttack, AREA_ATTACK_COOLDOWN);

        // TODO: add sound/particles for area attack
        /*
        game_events.add(
            GameEvent::SoundEvent(SoundSpec::new(
                player_pos,
                "wind".to_string(),
                (self.player_id, false),
            )),
            Recipients::All,
        );
        */
        game_events.add(
            GameEvent::ParticleEvent(ParticleSpec::new(
                ParticleType::AREA_ATTACK,
                player_pos,
                glm::vec3(0.0, 0.0, 0.0),
                //TODO: placeholder for player color
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(0.4, 0.9, 0.7, 1.0),
                format!("Area Attack from player {}", self.player_id),
            )),
            Recipients::All,
        );
        // loop over all other players
        for (other_player_id, other_player_state) in game_state.players.iter() {
            if &self.player_id == other_player_id {
                continue;
            }

            // get direction from this player to other player
            let other_player_pos = other_player_state.transform.translation;
            let vec_to_other = glm::normalize(&(other_player_pos - player_pos));

            // send ray to other player (may need multiple later)
            let solid = true;
            let filter = pipeline::QueryFilter::default().exclude_collider(player_collider_handle);

            let ray = geometry::Ray::new(
                rapier::point![player_pos.x, player_pos.y, player_pos.z],
                rapier::vector![vec_to_other.x, vec_to_other.y, vec_to_other.z],
            );
            if let Some((handle, toi)) = physics_state.query_pipeline.cast_ray(
                &physics_state.bodies,
                &physics_state.colliders,
                &ray,
                MAX_AREA_ATTACK_DIST,
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
                    let other_player_rigid_body = physics_state
                        .get_entity_rigid_body_mut(*other_player_id)
                        .unwrap();
                    let impulse_vec =
                        vec_to_other * (AREA_ATTACK_IMPULSE - (AREA_ATTACK_COEFF * toi));
                    other_player_rigid_body.apply_impulse(
                        rapier::vector![impulse_vec.x, impulse_vec.y, impulse_vec.z],
                        true,
                    );
                }
            }
        }

        Ok(())
    }
}
