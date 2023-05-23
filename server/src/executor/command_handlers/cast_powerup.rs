use super::{CommandHandler, GameEventCollector, HandlerError, HandlerResult};
use crate::simulation::physics_state::PhysicsState;
use crate::Recipients;
use common::configs::game_config::ConfigGame;
use common::core::command::Command;
use common::core::events::{GameEvent, SoundSpec};
use common::core::powerup_system::OtherEffects::Stun;
use common::core::powerup_system::PowerUpEffects::Invincible;
use common::core::powerup_system::{PowerUp, PowerUpStatus, StatusEffect, POWER_UP_TO_EFFECT_MAP};
use common::core::states::GameState;
use derive_more::Constructor;

#[derive(Constructor)]
pub struct CastPowerUpCommandHandler {
    player_id: u32,
    game_config: ConfigGame,
}

impl CommandHandler for CastPowerUpCommandHandler {
    fn handle(
        &self,
        game_state: &mut GameState,
        physics_state: &mut PhysicsState,
        game_events: &mut dyn GameEventCollector,
    ) -> HandlerResult {
        super::handle_invincible_players(game_state, physics_state, self.player_id);

        let game_state_clone = game_state.clone();
        let player_state = game_state
            .player_mut(self.player_id)
            .ok_or_else(|| HandlerError::new(format!("Player {} not found", self.player_id)))?;

        if player_state
            .status_effects
            .contains_key(&StatusEffect::Other(Stun))
            || !(player_state.power_up.clone().is_some()
                && player_state.power_up.clone().unwrap().1 == PowerUpStatus::Held)
        {
            return Ok(());
        } // Maybe Add Cleanse?

        // if powerup is on cooldown, or does not have a powerup, return
        if player_state.command_on_cooldown(Command::CastPowerUp) || player_state.power_up.is_none()
        {
            return Ok(());
        }

        let mut other_player_status_changes: Vec<(u32, StatusEffect, f32)> = vec![];

        if let Some((x, _)) = player_state.power_up.clone() {
            match x {
                PowerUp::Lightning => match game_state_clone.find_closest_player(self.player_id) {
                    Some(id) => {
                        other_player_status_changes.push((
                            id,
                            StatusEffect::Other(Stun),
                            self.game_config.powerup_config.power_up_debuff_duration,
                        ));
                        
                        // special case
                        player_state.power_up = None;
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
                        *POWER_UP_TO_EFFECT_MAP.get(&(x.value())).unwrap(),
                        self.game_config.powerup_config.power_up_buff_duration,
                    );
                    // by now the player should have casted the powerup successfully, change powerup status
                    player_state.power_up = Some((
                        player_state.power_up.clone().unwrap().0,
                        PowerUpStatus::Active,
                    ));
                }
            }
        };

        

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
                .contains_key(&StatusEffect::Power(Invincible))
            {
                other_player_state.status_effects.insert(*effect, *duration);
            }
        }

        Ok(())
    }
}
