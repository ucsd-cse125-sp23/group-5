use crate::communication::request::Request;
use crate::communication::response::Response;

const BUF_SIZE : usize = 1024;

pub fn req_to_byte(req: Request, buffer: &mut [u8]) {
    buffer[0] = req.client_id;

    let byte_array = req.data.as_bytes();
    let size = byte_array.len();
    for i in 1..size {
        buffer[i] = byte_array[i-1];
    };
    buffer[1+size] = b'\\';
    buffer[2+size] = b'~';
    buffer[3+size] = b'|';
}

pub fn resp_to_byte(resp: Response, buffer: &mut [u8]) {
    buffer[0] = resp.client_id;

    let mut count = 1;

    let byte_ts_arr:[u8; 16] = resp.timestamp.as_nanos().to_be_bytes();
    for i in 0..15 {
        buffer[count] = byte_ts_arr[i];
        count += 1;
    }

    let byte_array = resp.data.as_bytes();
    let size = byte_array.len();
    for i in 0..size-1 {
        buffer[count] = byte_array[i];
        count += 1;
    };
    buffer[count] = b'\\';
    buffer[count+1] = b'~';
    buffer[count+2] = b'|';
}

pub fn byte_to_req(buffer: &[u8], req: &mut Request) {
    (*req).client_id = buffer[0];
    let byte_array: &[u8; ] = [0; buffer.len()];
    for i in 1..buffer.len() {
        (*req).data
    }

}

pub fn byte_to_resp() {

}