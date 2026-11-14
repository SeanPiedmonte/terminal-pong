//! The network client: sends key presses, draws what the server sends back.

use std::io;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::net::{from_wire, read_frame, write_frame};
use crate::physics::{Game, Player};
use crate::proto::message::{ClientInput, ServerFrame};
use crate::render::Terminal;

const HELP: &str = "w/s or arrow keys to move, q to quit";

pub fn run(addr: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;

    let mut reader = stream.try_clone()?;
    let (tx, rx) = mpsc::channel::<Option<ServerFrame>>();
    thread::spawn(move || loop {
        match read_frame::<_, ServerFrame>(&mut reader) {
            Ok(frame) => {
                if tx.send(Some(frame)).is_err() {
                    break;
                }
            }
            Err(_) => {
                let _ = tx.send(None);
                break;
            }
        }
    });

    let mut term = Terminal::enter()?;
    let mut you: Option<Player> = None;
    let mut game: Option<Game> = None;
    let mut ending: Option<String> = None;

    loop {
        // Keep only the newest frame; older ones are already out of date.
        let mut dirty = false;
        for msg in rx.try_iter() {
            dirty = true;
            match msg {
                None => {
                    ending.get_or_insert_with(|| "Lost connection to the server.".into());
                }
                Some(frame) => {
                    you = Player::from_number(frame.you).or(you);
                    if let Some(state) = frame.state.as_ref() {
                        game = Some(from_wire(state));
                    }
                    if frame.opponent_left {
                        ending = Some("Your opponent left the game.".into());
                    } else if let Some(winner) = Player::from_number(frame.winner) {
                        ending = Some(if Some(winner) == you { "You win!".into() } else { "You lose.".into() });
                    }
                }
            }
        }

        let input = term.poll_input()?;
        if input.quit {
            return Ok(());
        }
        if ending.is_none() && input.dy != 0 {
            // One message per key press; the server moves one row for each.
            let mut msg = ClientInput::new();
            msg.dy = input.dy.signum();
            for _ in 0..input.dy.abs() {
                if write_frame(&mut stream, &msg).is_err() {
                    ending = Some("Lost connection to the server.".into());
                    dirty = true;
                    break;
                }
            }
        }

        if dirty {
            match (&game, &ending) {
                (Some(game), Some(text)) => term.draw(game, you, &format!("{text} Press q to quit."))?,
                (Some(game), None) => term.draw(game, you, HELP)?,
                (None, Some(text)) => term.message(&[text.as_str(), "", "Press q to quit."])?,
                (None, None) => {
                    let seat = you.map_or(String::new(), |p| format!("You are player {}.", p.number()));
                    term.message(&[seat.as_str(), "Waiting for another player to join...", "", "Press q to quit."])?
                }
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
}
