use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read from client!");
    
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Recieved request: {}", request);
    let response = "Hello, Client!".as_bytes();
    stream.write(response).expect("Failed to write to the client!");
}


pub fn start_server() -> std::io::Result<()> {

    let listener = TcpListener::bind("127.0.0.1:8080").
    expect("Failed to bind to address");
    println!("Server listening on port 8080..."); 

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
    
    Ok(())

}


