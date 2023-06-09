use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::core::action_states::ActionState;
use common::core::choices::FinalChoices;
use common::core::command::Command;
use common::core::events::{GameEvent, ParticleSpec, ParticleType, SoundSpec};
use common::core::powerup_system::{OtherEffects, PowerUpEffects, StatusEffect};
use common::core::states::GameState;
use derive_more::Constructor;
use nalgebra::UnitQuaternion;
use nalgebra_glm::Vec3;
use rapier3d::{geometry, pipeline};
use std::time::Duration;

extern crate nalgebra_glm as glm;
use crate::executor::command_handlers::{
    CommandHandler, GameEventCollector, HandlerError, HandlerResult,
};
use common::configs::game_config::ConfigGame;
use common::configs::physics_config::ConfigPhysics;
use rapier3d::prelude as rapier;

#[derive(Constructor)]
pub struct WaveCommandHandler {
    player_id: u32,
}

impl CommandHandler for WaveCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState, 
        _: &mut PhysicsState,
        _: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        
        let player_state = game_state.player_mut(self.player_id).unwrap();
        player_state.active_action_states.insert((ActionState::Wave, Duration::from_secs(3)));


        Ok(())
    }
}
