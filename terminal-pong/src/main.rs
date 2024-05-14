mod game;
mod client_handler;
use game::GameState;
mod server;

use std::io::{self, Write};


fn main() -> io::Result<()> {
    server::start_server()?;
    let game_state = GameState::new();
    game_state.run_game()?;
    client_handler::send_data(game_state)?;   
    Ok(())
}



