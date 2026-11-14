//! A debug log of every paddle hit and miss, for checking hit detection.
//!
//! A miss within `NEAR_MISS_ROWS` of the paddle is flagged, because those are
//! the ones that look like they should have been hits on screen.

use std::io::{self, Write};

use crate::physics::{Event, Game, PADDLE_REACH};

pub const NEAR_MISS_ROWS: f32 = 0.5;

pub struct EventLog {
    sink: Box<dyn Write + Send>,
    pub hits: u32,
    pub misses: u32,
    pub near_misses: u32,
}

/// One log line for an event. `game` is the state after the tick.
pub fn describe(event: &Event, game: &Game) -> String {
    let at = format!("[tick {:>6} {:>2}-{:<2}]", game.tick, game.p1points, game.p2points);
    match *event {
        Event::Hit { player, offset, speed } => format!(
            "{at} HIT   p{} offset={offset:+.2} of paddle, ball leaves at {speed:.0} cells/s",
            player.number()
        ),
        Event::Miss { player, ball_y, paddle_y, gap } => {
            let flag = if gap < NEAR_MISS_ROWS { "  <-- NEAR MISS, check hit detection" } else { "" };
            format!(
                "{at} MISS  p{} ball_y={ball_y:.2} paddle_y={paddle_y} (covers {:.1}..{:.1}) missed by {gap:.2} rows{flag}",
                player.number(),
                paddle_y as f32 - PADDLE_REACH,
                paddle_y as f32 + PADDLE_REACH,
            )
        }
    }
}

impl EventLog {
    pub fn new(sink: Box<dyn Write + Send>) -> EventLog {
        EventLog { sink, hits: 0, misses: 0, near_misses: 0 }
    }

    pub fn stdout() -> EventLog {
        EventLog::new(Box::new(io::stdout()))
    }

    pub fn file(path: &str) -> io::Result<EventLog> {
        let file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        Ok(EventLog::new(Box::new(file)))
    }

    pub fn note(&mut self, text: &str) {
        let _ = writeln!(self.sink, "{text}");
        let _ = self.sink.flush();
    }

    pub fn record(&mut self, events: &[Event], game: &Game) {
        for event in events {
            match event {
                Event::Hit { .. } => self.hits += 1,
                Event::Miss { gap, .. } => {
                    self.misses += 1;
                    if *gap < NEAR_MISS_ROWS {
                        self.near_misses += 1;
                    }
                }
            }
            self.note(&describe(event, game));
        }
    }

    pub fn summary(&mut self, game: &Game) {
        let line = format!(
            "Final score {}-{} after {} ticks: {} hits, {} misses, {} near misses",
            game.p1points, game.p2points, game.tick, self.hits, self.misses, self.near_misses
        );
        self.note(&line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::Player;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<Vec<u8>>>);
    impl Write for Shared {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn logs_hits_and_flags_only_near_misses() {
        let buffer = Shared::default();
        let mut log = EventLog::new(Box::new(buffer.clone()));
        let mut game = Game::new();
        game.tick = 42;
        log.record(
            &[
                Event::Hit { player: Player::One, offset: -0.5, speed: 30.0 },
                Event::Miss { player: Player::Two, ball_y: 11.7, paddle_y: 10, gap: 0.2 },
                Event::Miss { player: Player::One, ball_y: 2.0, paddle_y: 10, gap: 6.5 },
            ],
            &game,
        );
        log.summary(&game);

        let text = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("HIT   p1 offset=-0.50") && lines[0].contains("tick     42"));
        assert!(lines[1].contains("MISS  p2") && lines[1].contains("covers 8.5..11.5") && lines[1].contains("NEAR MISS"));
        assert!(lines[2].contains("missed by 6.50 rows") && !lines[2].contains("NEAR MISS"));
        assert!(lines[3].contains("1 hits, 2 misses, 1 near misses"));
    }
}
