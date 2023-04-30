pub mod executor;
pub mod game_loop;
pub mod outgoing_request;
pub mod simulation;

#[derive(Debug, Clone, PartialEq)]
pub enum Recipients {
    All,
    One(u8),
    Multiple(Vec<u8>),
}

impl Recipients {
    pub fn matches(&self, client_id: u8) -> bool {
        match self {
            Recipients::All => true,
            Recipients::One(id) => *id == client_id,
            Recipients::Multiple(ids) => ids.contains(&client_id),
        }
    }
}
