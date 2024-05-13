mod game;

use game::GameState;
mod server;

use std::io::{self, Write};


fn main() -> io::Result<()> {
    let game_state = GameState::new();
    game_state.run_game()?;
    server::start_server() 
}



