use clap::Parser;
use common::communication::commons::{Protocol, DEFAULT_SERVER_ADDR};
use common::communication::message::{HostRole, Message, Payload};
use common::core::command::{Command, MoveDirection};
use log::debug;
use std::io::Write;
use std::net::TcpListener;
use std::time::SystemTime;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    id: u8,
}

fn prompt(name: &str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    return line.trim().to_string();
}

/// parse token to command
fn parse_command(input: String) -> Option<Command> {
    match input.as_str() {
        "left" => Some(Command::Move(MoveDirection::Left)),
        "right" => Some(Command::Move(MoveDirection::Right)),
        "forward" => Some(Command::Move(MoveDirection::Forward)),
        "down" => Some(Command::Move(MoveDirection::Backward)),
        "spawn" => Some(Command::Spawn),
        _ => None,
    }
}

fn main() {
    let args = Cli::parse();

    let mut protocol = Protocol::connect(DEFAULT_SERVER_ADDR.parse().unwrap()).unwrap();

    // start a new thread to read messages from the server
    let mut protocol_clone = protocol.try_clone().unwrap();
    std::thread::spawn(move || loop {
        let msg = protocol_clone.read_message::<Message>().unwrap();
        println!("\n{:?}", msg);
        print!("> ");
        std::io::stdout().flush().unwrap();
    });

    loop {
        let input = prompt("> ");
        let mut tokens = Vec::from_iter(input.split_whitespace());
        if let Some(&command) = tokens.get(0) {
            if command == "quit" {
                break;
            }
            if command == "cmd" {
                let command = tokens.get(1).unwrap();
                let command = parse_command(command.to_string()).unwrap();
                protocol
                    .send_message(&Message::new(
                        HostRole::Client(args.id),
                        Payload::Command(command.clone()),
                    ))
                    .unwrap();
                println!("Sent command: {:?}", command)
            }
            if command == "ping" {
                protocol
                    .send_message(&Message::new(HostRole::Client(args.id), Payload::Ping))
                    .unwrap();
                println!("Sent ping")
            }
        }
    }
}
