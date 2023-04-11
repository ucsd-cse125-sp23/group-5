use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use server::thread_pool::ThreadPool;

fn main() {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // sleep to simulate a slow request
    std::thread::sleep(std::time::Duration::from_secs(5));

    // print the request
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}