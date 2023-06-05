use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::powerup_system::{PowerUpEffects, StatusEffect};
use common::core::states::GameState;
use common::core::weather::Weather;
use derive_more::Constructor;
use nalgebra::vector;
use rand::prelude::*;
use rapier3d::math::Vector;

use crate::executor::command_handlers::{CommandHandler, GameEventCollector, HandlerResult};
use crate::game_loop::TICK_RATE;
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;

extern crate nalgebra_glm as glm;

pub trait MarkovState<T> {
    fn next(&self) -> T;
}

// Constants for fraction of visits in long term (derived from limiting distribution)
const RAIN_FRACTION: f64 = 0.3;
const WIND_FRACTION: f64 = 0.2;
const NONE_FRACTION: f64 = 0.5;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(RAIN_FRACTION + WIND_FRACTION + NONE_FRACTION == 1.0);

// ticks of the effects
const RAIN_TICKS: f64 = 20. * TICK_RATE as f64;
// on average, it will rain every 10 seconds
const WIND_TICKS: f64 = 10. * TICK_RATE as f64;

const WIND_FORCE_MAGNITUDE: f32 = 128.0;

const RAINY_FRICTION: f32 = -0.2;

/// Modeling weather as a Markov process
impl MarkovState<Option<Weather>> for Option<Weather> {
    fn next(&self) -> Option<Weather> {
        let mut rng = thread_rng();
        let random_number: f64 = rng.gen(); // Generate a random number between 0 and 1

        match self {
            Some(Weather::Rainy) => {
                if random_number > 1. / RAIN_TICKS {
                    *self
                } else {
                    None
                }
            }
            Some(Weather::Windy(_)) => {
                if random_number > 1. / WIND_TICKS {
                    *self
                } else {
                    None
                }
            }
            None => {
                const TO_RAINY: f64 = RAIN_FRACTION / (NONE_FRACTION * RAIN_TICKS);
                const TO_WINDY: f64 = WIND_FRACTION / (NONE_FRACTION * WIND_TICKS);

                if random_number < TO_RAINY {
                    Some(Weather::Rainy)
                } else if random_number < TO_RAINY + TO_WINDY {
                    let wind_dir = thread_rng().gen_range(0.0..2.0 * std::f32::consts::PI);
                    let wind_dir = vector![wind_dir.cos(), 0.0, wind_dir.sin()];

                    Some(Weather::Windy(wind_dir))
                } else {
                    // stay the same
                    None
                }
            }
        }
    }
}

const WEATHER_START_DELAY: u64 = 60 * TICK_RATE;

#[derive(Constructor)]
/// Handles the command to start the weather
pub struct UpdateWeatherCommandHandler {}

impl CommandHandler for UpdateWeatherCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        _: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // don't do anything for the fist 1 min
        if game_state.life_cycle_state.unwrap_running() < WEATHER_START_DELAY {
            return Ok(());
        }

        game_state.world.weather = game_state.world.weather.next();

        Ok(())
    }
}

#[derive(Constructor)]
pub struct WeatherEffectCommandHandler {}

impl CommandHandler for WeatherEffectCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        match game_state.world.weather {
            Some(Weather::Rainy) => {
                self.handle_rainy_weather(game_state, physics_state, game_events)
            }
            Some(Weather::Windy(_)) => {
                self.handle_windy_weather(game_state, physics_state, game_events)
            }
            None => self.handle_reset_weather(game_state, physics_state, game_events),
        }
    }
}

impl WeatherEffectCommandHandler {
    fn handle_rainy_weather(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // reduce friction for every player
        for (&player_id, player_state) in game_state.players.iter() {
            if player_state.holds_status_effect(StatusEffect::Power(PowerUpEffects::Invincible)) {
                super::reset_weather(physics_state, player_id);
                continue;
            }

            let body = physics_state.get_entity_rigid_body_mut(player_id).unwrap();
            body.set_linear_damping(0.);

            let collider = physics_state.get_entity_collider_mut(player_id).unwrap();
            collider.set_friction(RAINY_FRICTION);

            // add rain particles every one second
            if game_state.life_cycle_state.unwrap_running() % TICK_RATE == 0 {
                game_events.add(
                    GameEvent::ParticleEvent(ParticleSpec::new(
                        ParticleType::RAIN,
                        player_state.transform.translation,
                        -Vector::y(),
                        Vector::y(),
                        glm::vec4(1.0, 1.0, 1.0, 0.8),
                        "rain".to_string(),
                    )),
                    Recipients::One(player_id as u8),
                )
            }

            // TODO: change to actual sound event
            game_events.add(
                GameEvent::SoundEvent(SoundSpec::new(
                    player_state.transform.translation,
                    "rain".to_string(),
                    (0, false),
                    (true, true),
                )),
                Recipients::One(player_id as u8),
            )
        }
        Ok(())
    }

    fn handle_windy_weather(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let wind_dir = match game_state.world.weather {
            Some(Weather::Windy(wind_dir)) => wind_dir,
            _ => return Ok(()),
        };
        for (&player_id, player_state) in game_state.players.iter() {
            // apply a force to the player

            if player_state.holds_status_effect(StatusEffect::Power(PowerUpEffects::Invincible)) {
                super::reset_weather(physics_state, player_id);
                continue;
            }

            let body = physics_state.get_entity_rigid_body_mut(player_id).unwrap();

            body.reset_forces(false);
            body.add_force(wind_dir * WIND_FORCE_MAGNITUDE, true);

            // add wind particles every one second
            if game_state.life_cycle_state.unwrap_running() % TICK_RATE == 0 {
                game_events.add(
                    GameEvent::ParticleEvent(ParticleSpec::new(
                        ParticleType::WIND,
                        glm::vec3(0.0, 0.0, 0.0),
                        wind_dir,
                        Vector::y(),
                        glm::vec4(0.2, 0.25, 0.25, 0.8),
                        "wind".to_string(),
                    )),
                    Recipients::One(player_id as u8),
                )
            }
        }
        Ok(())
    }

    fn handle_reset_weather(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        // reset friction for every player
        for (&player_id, _) in game_state.players.iter() {
            super::reset_weather(physics_state, player_id);

            // TODO: change to actual sound event
            game_events.add(
                // to stop rain sound
                GameEvent::SoundEvent(SoundSpec::new(
                    glm::Vec3::new(0.0, 0.0, 0.0),
                    "rain".to_string(),
                    (0, false),
                    (true, false),
                )),
                Recipients::One(player_id as u8),
            )
        }
        Ok(())
    }
}
