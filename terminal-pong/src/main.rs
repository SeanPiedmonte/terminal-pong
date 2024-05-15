mod game;
mod client_handler;
use game::GameState;
mod server;
use clap::{Command, Arg};

use std::io::{self, Write};


fn main() -> io::Result<()> {
    let app = Command::new("terminal-pong")
        .about("Terminal Pong Game")
        .arg(Arg::new("mode").required(true))
        .arg(Arg::new("server").required(false))
        .get_matches();

    let mode = app.get_one::<String>("mode").unwrap();
    let port = app.get_one::<String>("server").unwrap();
    if mode == "server" {
        let mut handler = server::start_server(port);
    } else if mode == "client" {
        let game_state = GameState::new();
        game_state.run_game(port);

    }
    Ok(())
}



