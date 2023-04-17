use crate::communication::commons::*;
use crate::core::command::Command;
use crate::core::states::GameState;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::convert::From;
use std::hash::Hasher;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct Message {
    pub host_role: HostRole,
    pub timestamp: u64,
    pub payload: Payload,
}

impl Message {
    pub fn new(host_role: HostRole, payload: Payload) -> Self {
        // timestamp is the time when the message is created
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            host_role,
            timestamp,
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HostRole {
    Server,
    Client(u8), // client id in the range of 1-255
}

#[derive(Debug)]
pub enum Payload {
    Ping,
    StateSync(GameState),
    Command(Command),
    Init(u8),
}

/// message kind to u8
impl From<&Payload> for u8 {
    fn from(msg: &Payload) -> Self {
        match msg {
            Payload::Ping => 0,
            Payload::StateSync(_) => 1,
            Payload::Command(_) => 2,
            Payload::Init(_) => 3,
        }
    }
}

impl From<&HostRole> for u8 {
    fn from(role: &HostRole) -> Self {
        match role {
            HostRole::Server => 0, // server always has id 0
            HostRole::Client(id) => {
                assert!(*id > 0); // client id must greater than 0
                *id
            }
        }
    }
}

impl From<Payload> for u8 {
    fn from(msg: Payload) -> Self {
        u8::from(&msg)
    }
}

impl From<HostRole> for u8 {
    fn from(role: HostRole) -> Self {
        u8::from(&role)
    }
}

impl Serialize for Message {
    /// Serialize Request to bytes (to send to client)
    ///
    /// Returns the number of bytes written
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u8(self.host_role.into())?; // write host role
        buf.write_u64::<NetworkEndian>(self.timestamp)?; // write timestamp
        buf.write_u8((&self.payload).into())?; // write payload kind

        match &self.payload {
            Payload::Ping => {}
            Payload::StateSync(state) => {
                prefix_len::write_json(buf, state)?;
            }
            Payload::Command(cmd) => {
                prefix_len::write_json(buf, cmd)?;
            }
            Payload::Init(client_id) => {
                prefix_len::write_json(buf, client_id)?;
            }
        }
        Ok(())
    }
}

impl Deserialize for Message {
    type Output = Message;
    /// Deserialize Response to bytes (to receive from server)
    fn deserialize(mut buf: &mut impl Read) -> io::Result<Self::Output> {
        let host_role = buf.read_u8()?;
        let timestamp = buf.read_u64::<NetworkEndian>()?;
        let payload_kind = buf.read_u8()?;

        let payload = match payload_kind {
            0 => Payload::Ping,
            1 => Payload::StateSync(prefix_len::extract_json(&mut buf)?),
            2 => Payload::Command(prefix_len::extract_json(&mut buf)?),
            3 => Payload::Init(prefix_len::extract_json(&mut buf)?),
            _ => panic!("Invalid payload kind {}", payload_kind),
        };

        Ok(Message {
            host_role: match host_role {
                0 => HostRole::Server,
                _ => HostRole::Client(host_role),
            },
            timestamp,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::command::MoveDirection;

    #[test]
    fn test_message_serialize_pin() {
        let msg = Message::new(HostRole::Server, Payload::Ping);
        let mut buf = Vec::new();
        msg.serialize(&mut buf).unwrap();
        assert_eq!(buf.len(), 10);
    }

    #[test]
    fn test_message_deserialize_pin() {
        let msg = Message::new(HostRole::Server, Payload::Ping);
        let mut buf = Vec::new();
        msg.serialize(&mut buf).unwrap();
        let msg2 = Message::deserialize(&mut buf.as_slice()).unwrap();
        assert!(matches!(msg2.payload, Payload::Ping));
    }

    #[test]
    fn test_message_round_trip_state_sync() {
        let msg = Message::new(HostRole::Server, Payload::StateSync(GameState::default()));
        let mut buf = Vec::new();
        msg.serialize(&mut buf).unwrap();

        let msg2 = Message::deserialize(&mut buf.as_slice()).unwrap();
        // cannot directly compare, compare debug string instead
        assert_eq!(format!("{:?}", msg), format!("{:?}", msg2));
    }

    #[test]
    fn test_message_round_trip_command() {
        let msg = Message::new(
            HostRole::Server,
            Payload::Command(Command::Move(MoveDirection::Left)),
        );
        let mut buf = Vec::new();
        msg.serialize(&mut buf).unwrap();

        let msg2 = Message::deserialize(&mut buf.as_slice()).unwrap();
        // cannot directly compare, compare debug string instead
        assert_eq!(format!("{:?}", msg), format!("{:?}", msg2));
    }
}
