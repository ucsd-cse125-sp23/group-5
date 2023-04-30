use derive_more::Constructor;
use common::communication::message::{HostRole, Message, Payload};
use common::core::events::GameEvent;
use common::core::states::GameState;
use crate::Recipients;

/// Outgoing events that are broadcast to the consumer threads.
#[derive(Debug, Clone, PartialEq)]
pub enum RequestKind {
    SyncGameState, // ask for a sync of game state
    SendGameEvent(GameEvent), // send a game event to the clients
}

#[derive(Constructor, Debug, Clone, PartialEq)]
pub struct OutgoingRequest {
    kind: RequestKind,
    recipients: Recipients,
}

impl OutgoingRequest {
    pub fn kind(&self) -> &RequestKind {
        &self.kind
    }
    pub fn recipients(&self) -> &Recipients {
        &self.recipients
    }

    pub fn make_message(&self, game_state: &GameState) -> Message {
        match &self.kind {
            RequestKind::SyncGameState => {
                Message::new(
                    HostRole::Server,
                    Payload::StateSync(game_state.clone()),
                )
            },
            RequestKind::SendGameEvent(event) => {
                Message::new(
                    HostRole::Server,
                    Payload::ServerEvent(event.clone()),
                )
            },
        }
    }
}