mod game;
mod client_handler;
use game::GameState;
mod server;
use clap::{Command, Arg};
use protobuf::Message;
include!(concat!(env!("OUT_DIR"), "/mod.rs"));
use message::Game_State;
use std::io;


fn main() -> io::Result<()> {
    let app = Command::new("terminal-pong")
        .about("Terminal Pong Game")
        .arg(Arg::new("mode").required(true))
        .arg(Arg::new("server").required(false))
        .get_matches();

    let mode = app.get_one::<String>("mode").unwrap();
    let port = app.get_one::<String>("server").unwrap();
    if mode == "server" {
        let _handler = server::start_server(port);
    } else if mode == "client" {
        let game_state = GameState::new();
        game_state.run_game(port).expect("Game was unable to run");

    } else {
        // test for if my protobuf thingy works
        let mut out_msg = Game_State::new();
        out_msg.bx = 5;
        out_msg.by = 5;
        out_msg.bdx = 5;
        out_msg.bdy = 5;
        out_msg.p1y = 5;
        out_msg.p2y = 5;
        out_msg.p1points = 5;
        out_msg.p2points = 5;

        let out_bytes: Vec<u8> = out_msg.write_to_bytes().unwrap();
        println!("Message request sent in bytes:\nout_bytes {:?}", out_bytes);

        let in_msg = Game_State::parse_from_bytes(&out_bytes).unwrap();
        println!("Message request received in bytes:\nin_msg {:?}", in_msg.bx);

        assert_eq!(in_msg.bx, out_msg.bx);
        assert_eq!(in_msg.by, out_msg.by);
        assert_eq!(in_msg.bdx, out_msg.bdx);
        assert_eq!(in_msg.bdy, out_msg.bdy);
        assert_eq!(in_msg.p1y, out_msg.p1y);
        assert_eq!(in_msg.p2y, out_msg.p2y);
        assert_eq!(in_msg.p1points, out_msg.p1points);
        assert_eq!(in_msg.p2points, out_msg.p2points);
    }        
    Ok(())
}



