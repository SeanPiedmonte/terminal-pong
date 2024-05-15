use std::io::{Read, Write};
use std::net::TcpStream;
use crate::game;
use game::GameState;
use protobuf::{EnumOrUnknown, Message};
include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
use message::Game_State;


pub fn send_data(game_state: GameState) -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to localhost");
    let mut out_msg = Game_State::new();
    out_msg.bx = game_state.get_bx() as u32;
    out_msg.by = game_state.get_by() as u32;
    out_msg.bdx = game_state.get_bdx() as i32;
    out_msg.bdy = game_state.get_bdy() as i32;
    out_msg.p1y = game_state.get_p1y() as u32;
    out_msg.p2y = game_state.get_p2y() as u32;
    out_msg.p1points = game_state.get_p1points() as u32;
    out_msg.p2points = game_state.get_p2points() as u32;

    let out_bytes: Vec<u8> = out_msg.write_to_bytes().unwrap();
    
    stream.write(&out_bytes)?;

    Ok(())
}
