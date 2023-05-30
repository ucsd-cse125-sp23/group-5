use crate::executor::command_handlers::{
    CommandHandler, GameEventCollector, HandlerError, HandlerResult,
};
use crate::simulation::physics_state::PhysicsState;
use common::core::command::{CheatCodeControl, CheatKeyWeather};
use common::core::states::GameState;
use common::core::weather::Weather;
use derive_more::Constructor;
use rand::{thread_rng, Rng};
use rapier3d::prelude::vector;

#[derive(Constructor)]
pub struct WeatherCheatKeyCommandHandler {
    player_id: u32,
    weather: CheatKeyWeather,
}

impl CommandHandler for WeatherCheatKeyCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        let player_state = game_state
            .player(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if !player_state.cheat_keys_enabled {
            return Ok(());
        }

        for (id, _) in game_state.players.iter() {
            super::reset_weather(physics_state, *id);
        }

        match self.weather {
            CheatKeyWeather::Rain => {
                game_state.world.weather = Some(Weather::Rainy);
            }
            CheatKeyWeather::Wind => {
                let wind_dir = thread_rng().gen_range(0.0..2.0 * std::f32::consts::PI);
                let wind_dir = vector![wind_dir.cos(), 0.0, wind_dir.sin()];
                game_state.world.weather = Some(Weather::Windy(wind_dir));
            }
            CheatKeyWeather::Reset => {
                game_state.world.weather = None;
            }
        }
        Ok(())
    }
}
