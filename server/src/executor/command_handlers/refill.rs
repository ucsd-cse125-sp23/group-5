use super::{CommandHandler, GameEventCollector, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::game_config::ConfigGame;
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType};
use common::core::powerup_system::{OtherEffects, StatusEffect};
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;

#[derive(Constructor)]
pub struct RefillCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for RefillCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        let player_state = game_state.player_mut(self.player_id).unwrap();
        // if player is dead, don't do anything
        if player_state.is_dead {
            return Ok(());
        }

        // if player is stunned
        if player_state.holds_status_effect_mut(StatusEffect::Other(OtherEffects::Stun)) {
            return Ok(());
        }

        if !player_state.is_need_refill(self.game_config.clone()) {
            return Ok(());
        }

        let refill_point_result = player_state.is_in_refill_area(self.game_config.clone());

        if refill_point_result == None || player_state.command_on_cooldown(Command::Refill) {
            // signal player that he/she is not in refill area
            return Ok(());
        }

        player_state.refill_wind_charge(
            Some(self.game_config.one_charge),
            self.game_config.max_wind_charge,
        );

        player_state.insert_cooldown(Command::Refill, self.game_config.refill_rate_limit);

        let player_pos = player_state.transform.translation;
        let refill_point = match refill_point_result {
            Some(point) => point,
            None => return Ok(()),
        };

        let player_customization = game_state.players_customization.get(&self.player_id);
        let leaf_color = match player_customization {
            Some(c) => {
                c.color
                    .get(common::core::choices::LEAF_MESH)
                    .unwrap()
                    .rgb_color
            }
            None => [0.0, 1.0, 1.0],
        };
        let atk_particle: String;
        {
            let def = String::from("korok_1");
            let model_id = match player_customization {
                Some(c) => &c.model[..],
                None => &def[..],
            };

            atk_particle = match model_id {
                "korok_1" => String::from(common::configs::particle_config::MODEL_1),
                "korok_2" => String::from(common::configs::particle_config::MODEL_2),
                "korok_3" => String::from(common::configs::particle_config::MODEL_3),
                "korok_4" => String::from(common::configs::particle_config::MODEL_4),
                _ => String::from(common::configs::particle_config::MODEL_1),
            }
        }
        game_events.add(
            GameEvent::ParticleEvent(ParticleSpec::new(
                ParticleType::REFILL_ATTACK,
                refill_point,
                (player_pos - refill_point).normalize(),
                glm::vec3(0.0, 1.0, 0.0),
                glm::vec4(leaf_color[0], leaf_color[1], leaf_color[2], 0.85),
                atk_particle,
            )),
            Recipients::All,
        );
        Ok(())
    }
}
