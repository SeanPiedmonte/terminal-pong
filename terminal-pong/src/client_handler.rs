use std::io::{Read, Write};
use std::net::TcpStream;
use game::GameState;
use serde::{Serialize, Deserialize};
use serde_protobuf;

pub fn send_data(&game_state: GameState) -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to localhost");
    
    let data = serde_protobuf::to_string(&game_state).unwrap();

    
    stream.write_all(data.as_bytes);

    Ok(())
}
