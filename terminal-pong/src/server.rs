use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::client_handler::Client;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1500];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    println!("Client Disconnected");
                    break;
                }

                let request = String::from_utf8_lossy(&buffer[..]);
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
