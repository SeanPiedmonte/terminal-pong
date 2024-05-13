use std::io{Read, Write};
use std::net::TcpStream;
use game::GameState;


pub fn send_data(&game_state: GameState) {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to localhost");

    //stream.write(game_state);
}
