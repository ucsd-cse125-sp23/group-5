use std::convert::From;
use std::hash::Hasher;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

use crate::communication::commons::*;

// Request object (client -> server)
#[derive(Debug)]
pub struct Request (
    /* Request Format
         client_id(u8) |/ event \~|
         inter-msg delimiter: "|/"
         final delimiter: "\~|"
     */
    pub String
);

impl Request {
    /// Create a new response with a given message
    pub fn new(message: String) -> Self {
        Self(message)
    }

    /// Get the response message value
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl Serialize for Request {
    /// Serialize Request to bytes (to send to client)
    ///
    /// Returns the number of bytes written
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        let req_bytes = self.0.as_bytes();
        buf.write_all(&req_bytes)?;
        Ok(req_bytes.len()) // Type + len + bytes
    }
}

impl Deserialize for Request {
    type Output = Request;
    /// Deserialize Response to bytes (to receive from server)
    fn deserialize(mut buf: &mut impl Read) -> io::Result<Self::Output> {
        let value = extract_string(&mut buf)?;
        Ok(Request(value))
    }
}