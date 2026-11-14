//! The two-player server. It owns the game: clients send paddle movements,
//! the server runs the physics and sends every frame back to both of them.

use std::io;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use protobuf::MessageField;

use crate::eventlog::EventLog;
use crate::net::{read_frame, to_wire, write_frame};
use crate::physics::{Game, Player, TICK_HZ};
use crate::proto::message::{ClientInput, ServerFrame};

/// What a client's reader thread reports to the game loop.
enum FromClient {
    Move(Player, i32),
    Gone(Player),
}

const SEATS: [Player; 2] = [Player::One, Player::Two];

fn welcome(player: Player) -> ServerFrame {
    let mut frame = ServerFrame::new();
    frame.you = player.number();
    frame.waiting = true;
    frame
}

fn listen(stream: &TcpStream, player: Player, tx: mpsc::Sender<FromClient>) -> io::Result<()> {
    let mut reader = stream.try_clone()?;
    thread::spawn(move || loop {
        match read_frame::<_, ClientInput>(&mut reader) {
            // One message moves the paddle at most one row, so a client
            // cannot teleport by sending a large dy.
            Ok(input) => {
                if tx.send(FromClient::Move(player, input.dy.signum())).is_err() {
                    break;
                }
            }
            Err(_) => {
                let _ = tx.send(FromClient::Gone(player));
                break;
            }
        }
    });
    Ok(())
}

/// Run one match to completion between two connected clients.
fn play_match(mut streams: [TcpStream; 2], rx: Receiver<FromClient>, log: &mut EventLog) {
    let mut game = Game::new();
    let tick = Duration::from_secs(1) / TICK_HZ;
    let mut next = Instant::now();
    let mut left: Option<Player> = None;

    while game.winner().is_none() && left.is_none() {
        for msg in rx.try_iter() {
            match msg {
                FromClient::Move(player, dy) => game.move_paddle(player, dy),
                FromClient::Gone(player) => left = Some(player),
            }
        }
        if left.is_some() {
            break;
        }

        let events = game.step();
        log.record(&events, &game);

        let mut frame = ServerFrame::new();
        frame.state = MessageField::some(to_wire(&game));
        frame.winner = game.winner().map_or(0, Player::number);
        for (stream, player) in streams.iter_mut().zip(SEATS) {
            frame.you = player.number();
            if write_frame(stream, &frame).is_err() {
                left = Some(player);
            }
        }

        next += tick;
        thread::sleep(next.saturating_duration_since(Instant::now()));
    }

    if let Some(player) = left {
        log.note(&format!("Player {} disconnected", player.number()));
        let mut frame = ServerFrame::new();
        frame.state = MessageField::some(to_wire(&game));
        frame.opponent_left = true;
        for (stream, seat) in streams.iter_mut().zip(SEATS) {
            if seat != player {
                frame.you = seat.number();
                let _ = write_frame(stream, &frame);
            }
        }
    }
    log.summary(&game);
    for stream in &streams {
        let _ = stream.shutdown(Shutdown::Both);
    }
}

/// Accept players two at a time and run their matches, forever.
pub fn run(addr: &str, log: EventLog) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("terminal-pong server listening on {}", listener.local_addr()?);
    serve(listener, log)
}

/// Like `run`, on a listener the caller has already bound.
pub fn serve(listener: TcpListener, mut log: EventLog) -> io::Result<()> {
    loop {
        println!("Waiting for two players...");
        let (tx, rx) = mpsc::channel();
        let mut seated: Vec<TcpStream> = Vec::new();

        while seated.len() < 2 {
            // If the first player gave up while waiting, free their seat.
            if let Ok(FromClient::Gone(_)) = rx.try_recv() {
                println!("Player 1 left before the match started");
                seated.clear();
            }
            let (mut stream, peer) = listener.accept()?;
            if let Ok(FromClient::Gone(_)) = rx.try_recv() {
                println!("Player 1 left before the match started");
                seated.clear();
            }
            let player = SEATS[seated.len()];
            let ready = stream
                .set_nodelay(true)
                .and_then(|_| write_frame(&mut stream, &welcome(player)))
                .and_then(|_| listen(&stream, player, tx.clone()));
            match ready {
                Ok(()) => {
                    println!("Player {} connected from {peer}", player.number());
                    seated.push(stream);
                }
                Err(err) => println!("Dropped a connection from {peer}: {err}"),
            }
        }

        let two = seated.pop().expect("two players are seated");
        let one = seated.pop().expect("two players are seated");
        log.note("Match started");
        play_match([one, two], rx, &mut log);
    }
}
