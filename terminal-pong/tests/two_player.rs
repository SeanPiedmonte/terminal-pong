//! Runs the real server on a local port and plays it with two raw TCP clients.

use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use terminal_pong::eventlog::EventLog;
use terminal_pong::net::{read_frame, write_frame};
use terminal_pong::proto::message::{ClientInput, ServerFrame};
use terminal_pong::server;

fn start_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    thread::spawn(move || server::serve(listener, EventLog::new(Box::new(io::sink()))));
    addr
}

fn connect(addr: &str) -> TcpStream {
    let stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    stream
}

fn next(stream: &mut TcpStream) -> ServerFrame {
    read_frame(stream).expect("a frame from the server")
}

fn send(stream: &mut TcpStream, dy: i32) {
    let mut input = ClientInput::new();
    input.dy = dy;
    write_frame(stream, &input).unwrap();
}

/// Read frames until one satisfies `done`, failing if 300 go by first.
fn wait_for(stream: &mut TcpStream, what: &str, done: impl Fn(&ServerFrame) -> bool) -> ServerFrame {
    for _ in 0..300 {
        let frame = next(stream);
        if done(&frame) {
            return frame;
        }
    }
    panic!("never saw: {what}");
}

#[test]
fn server_seats_two_players_applies_their_input_and_reports_a_disconnect() {
    let addr = start_server();

    let mut one = connect(&addr);
    let welcome = next(&mut one);
    assert_eq!((welcome.you, welcome.waiting), (1, true));
    assert!(welcome.state.is_none(), "no game until both players are in");

    let mut two = connect(&addr);
    let welcome = next(&mut two);
    assert_eq!((welcome.you, welcome.waiting), (2, true));

    // Both now receive the same game, each told which side they are.
    let a = wait_for(&mut one, "first state for player one", |f| f.state.is_some());
    let b = wait_for(&mut two, "first state for player two", |f| f.state.is_some());
    assert_eq!((a.you, b.you), (1, 2));
    assert!(!a.waiting && a.winner == 0);
    let (start_p1, start_p2) = (a.state.p1y, a.state.p2y);

    // Three presses up for player one. Messages are sent back to back, so
    // this also exercises framing over a real socket.
    for _ in 0..3 {
        send(&mut one, -1);
    }
    // One oversized move from player two counts as a single row.
    send(&mut two, 100);

    let moved = wait_for(&mut two, "both paddles to move", |f| {
        f.state.p1y == start_p1 - 3 && f.state.p2y == start_p2 + 1
    });
    assert!(moved.state.tick > a.state.tick, "the server is advancing the game");

    // The ball is served after the pause and moves on its own.
    let before = wait_for(&mut one, "the serve", |f| f.state.is_some() && f.state.serve_in == 0);
    let after = wait_for(&mut one, "the ball to move", |f| f.state.bx != before.state.bx);
    assert!(after.state.tick > before.state.tick);

    drop(two);
    let end = wait_for(&mut one, "opponent_left", |f| f.opponent_left);
    assert_eq!(end.you, 1);

    // The server is ready for the next pair.
    let mut again = connect(&addr);
    assert_eq!(next(&mut again).you, 1);
}

#[test]
fn a_player_who_leaves_the_lobby_frees_their_seat() {
    let addr = start_server();
    let mut early = connect(&addr);
    assert_eq!(next(&mut early).you, 1);
    drop(early);
    thread::sleep(Duration::from_millis(200));

    let mut one = connect(&addr);
    assert_eq!(next(&mut one).you, 1, "the abandoned seat is reused");
    let mut two = connect(&addr);
    assert_eq!(next(&mut two).you, 2);
    wait_for(&mut one, "the match to start", |f| f.state.is_some());
}
