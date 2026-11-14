mod game;
mod client_handler;
use game::GameState;
mod server;
use clap::{Command, Arg};
use protobuf::Message;
include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
use message::Game_State;
use std::io;


fn main() -> io::Result<()> {
    let app = Command::new("terminal-pong")
        .about("Terminal Pong Game")
        .arg(
            Arg::new("mode")
                .required(true)
                .value_parser(["server", "client", "selftest"])
                .help("server: print game state received from a client; client: play the game; selftest: protobuf round-trip check"),
        )
        .arg(
            Arg::new("address")
                .required(false)
                .default_value("127.0.0.1:7878")
                .help("host:port the server listens on / the client connects to"),
        )
        .get_matches();

    let mode = app.get_one::<String>("mode").unwrap();
    let port = app.get_one::<String>("address").unwrap();
    if mode == "server" {
        println!("Listening on {port}");
        server::start_server(port)?;
    } else if mode == "client" {
        let game_state = GameState::new();
        if let Err(err) = game_state.run_game(port) {
            eprintln!("Could not start the game against {port}: {err}");
            eprintln!("Start a server first: terminal-pong server {port}");
            std::process::exit(1);
        }

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
        println!("Protobuf round-trip OK");
    }        
    Ok(())
}
