use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::client_handler::Client;
use protobuf::{EnumOrUnknown, Message};
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
use message::Game_State;

fn handle_client(mut stream: TcpStream) {
    let mut buffer : Vec<u8> = vec![0; 1500];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Client Disconnected");
                    break;
                }

                let request = Game_State::parse_from_bytes(&buffer).unwrap();
                println!("Recieved request: {}", request);
            },
            Err(err) => {
                println!("Error reading from client: {:?}", err);
            }
        }
    }
}


pub fn start_server(port: &str) -> std::io::Result<()> {

    let listener = TcpListener::bind(port).unwrap();
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_client(stream);
    }
    Ok(())

}
