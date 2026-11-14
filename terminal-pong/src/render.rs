//! Drawing the game with crossterm.

use std::io::{self, Stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::physics::{Game, Player, HEIGHT, TICK_HZ, WIDTH};

const COLS: usize = WIDTH as usize;
const ROWS: usize = HEIGHT as usize;
/// Score line, top wall, playfield, bottom wall, status line.
pub const NEEDED_ROWS: u16 = ROWS as u16 + 4;
pub const NEEDED_COLS: u16 = COLS as u16;

/// What the player asked for this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    /// Net paddle movement in rows (-1 per "up" press, +1 per "down").
    pub dy: i32,
    pub quit: bool,
    pub any_key: bool,
}

/// Puts the terminal in raw mode on an alternate screen, and puts it back
/// when dropped, including when the game exits through an error.
pub struct Terminal {
    out: Stdout,
}

impl Terminal {
    pub fn enter() -> io::Result<Terminal> {
        let (cols, rows) = terminal::size()?;
        if cols < NEEDED_COLS || rows < NEEDED_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("terminal is {cols}x{rows}; terminal-pong needs at least {NEEDED_COLS}x{NEEDED_ROWS}"),
            ));
        }
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        if let Err(err) = execute!(out, EnterAlternateScreen, Hide, Clear(ClearType::All)) {
            let _ = terminal::disable_raw_mode();
            return Err(err);
        }
        Ok(Terminal { out })
    }

    /// Collect every key pressed since the last call, without blocking.
    pub fn poll_input(&mut self) -> io::Result<Input> {
        let mut input = Input::default();
        while event::poll(std::time::Duration::ZERO)? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            input.any_key = true;
            match key.code {
                KeyCode::Char('w') | KeyCode::Char('k') | KeyCode::Up => input.dy -= 1,
                KeyCode::Char('s') | KeyCode::Char('j') | KeyCode::Down => input.dy += 1,
                KeyCode::Esc | KeyCode::Char('q') => input.quit = true,
                // Raw mode delivers Ctrl-C as a key rather than a signal.
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => input.quit = true,
                _ => {}
            }
        }
        Ok(input)
    }

    pub fn draw(&mut self, game: &Game, you: Option<Player>, status: &str) -> io::Result<()> {
        for (row, line) in frame(game, you, status).iter().enumerate() {
            queue!(self.out, MoveTo(0, row as u16), Print(line))?;
        }
        self.out.flush()
    }

    /// Show a message in the middle of an otherwise empty screen.
    pub fn message(&mut self, lines: &[&str]) -> io::Result<()> {
        queue!(self.out, Clear(ClearType::All))?;
        let top = (NEEDED_ROWS as usize).saturating_sub(lines.len()) / 2;
        for (i, line) in lines.iter().enumerate() {
            let col = COLS.saturating_sub(line.chars().count()) / 2;
            queue!(self.out, MoveTo(col as u16, (top + i) as u16), Print(line))?;
        }
        self.out.flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(self.out, Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn pad(text: &str) -> String {
    let mut line: String = text.chars().take(COLS).collect();
    let len = line.chars().count();
    line.extend(std::iter::repeat(' ').take(COLS - len));
    line
}

/// The whole screen as fixed-width lines. Every line is redrawn in full each
/// frame, so nothing needs clearing and the picture does not flicker.
pub fn frame(game: &Game, you: Option<Player>, status: &str) -> Vec<String> {
    let mut grid = vec![vec![' '; COLS]; ROWS];

    for (y, row) in grid.iter_mut().enumerate() {
        if y % 2 == 0 {
            row[COLS / 2] = ':';
        }
    }
    for (x, centre) in [(0, game.p1y), (COLS - 1, game.p2y)] {
        for y in centre - 1..=centre + 1 {
            if (0..ROWS as i32).contains(&y) {
                grid[y as usize][x] = '|';
            }
        }
    }
    let bx = (game.bx.round() as i32).clamp(1, COLS as i32 - 2) as usize;
    let by = (game.by.round() as i32).clamp(0, ROWS as i32 - 1) as usize;
    grid[by][bx] = 'O';

    let tag = |p: Player| if you == Some(p) { " (you)" } else { "" };
    let left = format!("Player 1{}: {}", tag(Player::One), game.p1points);
    let right = format!("Player 2{}: {}", tag(Player::Two), game.p2points);
    let gap = COLS.saturating_sub(left.chars().count() + right.chars().count());
    let score = format!("{left}{}{right}", " ".repeat(gap));

    let centre_note = if game.serve_in > 0 && game.winner().is_none() {
        format!("serve in {}", game.serve_in / TICK_HZ + 1)
    } else if game.rally > 1 {
        format!("rally {}", game.rally)
    } else {
        String::new()
    };
    let mut wall: Vec<char> = "-".repeat(COLS).chars().collect();
    if !centre_note.is_empty() {
        let note: Vec<char> = format!(" {centre_note} ").chars().collect();
        let start = (COLS - note.len()) / 2;
        wall[start..start + note.len()].copy_from_slice(&note);
    }

    let mut lines = Vec::with_capacity(ROWS + 4);
    lines.push(pad(&score));
    lines.push(wall.into_iter().collect());
    lines.extend(grid.into_iter().map(|row| row.into_iter().collect::<String>()));
    lines.push("-".repeat(COLS));
    lines.push(pad(status));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_is_a_fixed_size_and_shows_ball_paddles_and_score() {
        let mut game = Game::new();
        game.serve_in = 0;
        game.bx = 10.2;
        game.by = 4.6;
        game.p1y = 5;
        game.p2y = 12;
        game.p1points = 3;
        game.p2points = 7;
        let lines = frame(&game, Some(Player::Two), "w/s to move");

        assert_eq!(lines.len(), NEEDED_ROWS as usize);
        assert!(lines.iter().all(|l| l.chars().count() == COLS));
        assert!(lines[0].starts_with("Player 1: 3") && lines[0].ends_with("Player 2 (you): 7"));

        let field = &lines[2..2 + ROWS];
        assert_eq!(field[5].chars().nth(10), Some('O'));
        for y in 4..=6 {
            assert_eq!(field[y].chars().next(), Some('|'));
        }
        for y in 11..=13 {
            assert_eq!(field[y].chars().last(), Some('|'));
        }
        assert!(lines.last().unwrap().starts_with("w/s to move"));
    }

    #[test]
    fn ball_is_never_drawn_over_a_paddle_or_off_screen() {
        let mut game = Game::new();
        for (bx, by) in [(-5.0, -5.0), (500.0, 500.0), (0.0, 10.0), (79.0, 10.0)] {
            game.bx = bx;
            game.by = by;
            let lines = frame(&game, None, "");
            let balls: Vec<usize> = lines.iter().flat_map(|l| l.chars().enumerate().filter(|(_, c)| *c == 'O').map(|(x, _)| x)).collect();
            assert_eq!(balls.len(), 1);
            assert!((1..COLS - 1).contains(&balls[0]));
        }
    }
}
