

// client to server "request"
// potential improvement: replace data type of VirtualKey Code?
pub struct Request {
    pub client_id : u8,
    pub data : String
}

impl Request {
    pub fn new(client_id: u8, data: String) -> Request {
        Request { client_id, data }
    }
}