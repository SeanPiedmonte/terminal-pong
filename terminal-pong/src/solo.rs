//! Single-player against the computer, with no server involved.

use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crate::ai::Ai;
use crate::eventlog::EventLog;
use crate::physics::{Game, Player, TICK_HZ};
use crate::render::Terminal;

pub fn run(mut ai: Ai, mut log: Option<EventLog>) -> io::Result<()> {
    let mut term = Terminal::enter()?;
    let mut game = Game::new();
    let help = format!("w/s or arrow keys to move, q to quit - computer: {}", ai.difficulty().name());
    let tick = Duration::from_secs(1) / TICK_HZ;
    let mut next = Instant::now();

    if let Some(log) = log.as_mut() {
        log.note(&format!("Solo match started against the {} computer", ai.difficulty().name()));
    }

    while game.winner().is_none() {
        let input = term.poll_input()?;
        if input.quit {
            break;
        }
        game.move_paddle(Player::One, input.dy);
        let ai_move = ai.decide(&game, Player::Two);
        game.move_paddle(Player::Two, ai_move);

        let events = game.step();
        if let Some(log) = log.as_mut() {
            log.record(&events, &game);
        }
        term.draw(&game, Some(Player::One), &help)?;

        next += tick;
        thread::sleep(next.saturating_duration_since(Instant::now()));
    }

    if let Some(log) = log.as_mut() {
        log.summary(&game);
    }
    if let Some(winner) = game.winner() {
        let text = if winner == Player::One { "You win!" } else { "The computer wins." };
        term.draw(&game, Some(Player::One), &format!("{text} Press any key to exit."))?;
        // Ignore keys still held from the last rally before waiting.
        thread::sleep(Duration::from_millis(600));
        term.poll_input()?;
        while !term.poll_input()?.any_key {
            thread::sleep(Duration::from_millis(30));
        }
    }
    Ok(())
}
