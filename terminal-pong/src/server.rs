use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::client_handler::Client;

fn handle_client(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1500];
        stream.read(&mut buffer).expect("Failed to read from client!");

        let request = String::from_utf8_lossy(&buffer[..]);
        println!("Recieved request: {}", request);
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
