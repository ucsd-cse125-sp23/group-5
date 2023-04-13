// using SystemTime instead?
use std::time::{Duration};

// server to client interaction: "response"
// potential improvement: replace data type of VirtualKey Code?

#[derive(Debug)]
pub struct Response(
    /* Response Format
         client_id | timestamp(Duration) | game_state(json) |
     */
    pub String);

impl Response {
    /// Create a new response with a given message
    pub fn new(message: String) -> Self {
        Self(message)
    }

    /// Get the response message value
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl Serialize for Response {
    /// Serialize Response to bytes (to send to client)
    ///
    /// Returns the number of bytes written
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        let resp_bytes = self.0.as_bytes();
        buf.write_u16::<NetworkEndian>(resp_bytes.len() as u16)?;
        buf.write_all(&resp_bytes)?;
        Ok(3 + resp_bytes.len()) // Type + len + bytes
    }
}

impl Deserialize for Response {
    type Output = Response;
    /// Deserialize Response to bytes (to receive from server)
    fn deserialize(mut buf: &mut impl Read) -> io::Result<Self::Output> {
        let value = extract_string(&mut buf)?;
        Ok(Response(value))
    }
}

}