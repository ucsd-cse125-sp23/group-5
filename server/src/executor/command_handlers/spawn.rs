use super::{CommandHandler, GameEventCollector, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use common::configs::game_config::ConfigGame;
use common::core::command::Command;
use common::core::powerup_system::{PowerUpEffects, StatusEffect};
use common::core::states::{GameState, PlayerState};
use derive_more::Constructor;
use nalgebra::Point;
use rapier3d::dynamics::MassProperties;
use rapier3d::geometry;
use rapier3d::math::AngVector;

#[derive(Constructor)]
pub struct SpawnCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for SpawnCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // get spawn-locations with corresponding id
        let spawn_position = self.game_config.spawn_points[self.player_id as usize - 1];

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
                player.refill_wind_charge(
                    Some(self.game_config.max_wind_charge),
                    self.game_config.max_wind_charge,
                );
            }
        } else {
            let collider = geometry::ColliderBuilder::capsule_y(0.5, 0.25)
                .mass(0.0)
                .build();

            let rigid_body = rapier3d::prelude::RigidBodyBuilder::dynamic()
                .translation(spawn_position)
                .ccd_enabled(true)
                // add additional mass to the lower half of the player so that it doesn't tip over
                .additional_mass_properties(MassProperties::new(
                    Point::from_slice(&[0.0, -0.7, 0.0]),
                    15.0,
                    AngVector::new(1.425, 1.425, 0.45),
                ))
                .build();

            physics_state.insert_entity(self.player_id, Some(collider), Some(rigid_body));

            // Game state (needed because syncing is only for the physical properties of entities)
            game_state.players.insert(
                self.player_id,
                PlayerState {
                    id: self.player_id,
                    is_dead: false,
                    wind_charge: self.game_config.max_wind_charge,
                    on_flag_time: 0.0,
                    spawn_point: spawn_position,
                    power_up: None,
                    ..Default::default()
                },
            );
        }

        // give just spawned player some invincibility
        super::reset_weather(physics_state, self.player_id);
        game_state
            .player_mut(self.player_id)
            .unwrap()
            .status_effects
            .insert(
                StatusEffect::Power(PowerUpEffects::Invincible),
                self.game_config.powerup_config.spawn_invincible_duration,
            );
        Ok(())
    }
}
