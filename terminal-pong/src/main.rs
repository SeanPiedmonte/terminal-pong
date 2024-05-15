mod game;
mod client_handler;
use game::GameState;
mod server;

use std::io::{self, Write};


fn main() -> io::Result<()> {
    let game_state = GameState::new();
    server::start_server()?;
    game_state.run_game()?;
    client_handler::send_data(game_state)?;   
    Ok(())
}



