// using SystemTime instead?
use std::time::{Duration};

// server to client "response"
// potential improvement: replace data type of VirtualKey Code?
pub struct Response {
    pub client_id : u8,
    pub data : String,
    pub timestamp : Duration
}

impl Response {
    pub fn new(client_id: u8, data: String, timestamp: Duration) -> Response {
        Response { client_id, data, timestamp }
    }
}