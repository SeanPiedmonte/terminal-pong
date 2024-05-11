use std::io::{self, Write};
use crossterm::{execute, cursor::MoveTo, terminal::{Clear, ClearType}, cursor::{Hide, Show}, 
    event::{self, KeyCode, KeyEvent, KeyModifiers, KeyCode::Char, read, Event}};
use std::time::Duration;
use std::thread;
use std::sync::mpsc;

const WIDTH: u16 = 80;
const HEIGHT: u16 = 20;

struct GameState {
    bx: u16,
    by: u16,
    p1y: u16,
    p2y: u16,
    bdx: i16,
    bdy: i16,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            bx: WIDTH / 2,
            by: HEIGHT / 2,
            p1y: HEIGHT / 2,
            p2y: HEIGHT / 2 + 1,
            bdx: 1,
            bdy: 1,
        }
    }

    fn update_ball(&mut self) {
        self.bx = (self.bx as i16 + self.bdx) as u16;
        self.by = (self.by as i16 + self.bdy) as u16;

        if (self.bx == 0 && (self.by == self.p1y + 1 || self.by == self.p1y - 1)) || 
           (self.bx == WIDTH && (self.by == self.p2y + 1 || self.by == self.p2y - 1)) {
            self.bdx *= -1;
            self.bdy = if self.bdx > 0 {2} else {-2};
        } else if (self.by == self.p1y && self.bx == 0) || (self.bx == WIDTH && self.by == self.p2y) {
            self.bdx = if self.bdx > 0 {1} else {-1};
            self.bdx *= -1;
            self.bdy *= -1;
        } else if self.bx >= WIDTH || self.bx <= 0 {
            self.bdx *= -1;
        }

        if self.by >= HEIGHT || self.by <= 0 {
            self.bdy *= -1;
        }

    }

    fn update_paddles(&mut self, input_rx: &mpsc::Receiver<KeyEvent>) {
        while let Ok(event) = input_rx.try_recv() {
            match event.code {
                KeyCode::Char('w') => self.p1y = self.p1y.saturating_sub(1),
                KeyCode::Char('s') => self.p1y = (self.p1y + 1).min(HEIGHT - 1),
                _ => {}
            }
        }
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        execute!(stdout, Hide)?;

        execute!(stdout, Clear(ClearType::All))?;

        execute!(stdout, MoveTo(0, self.p1y + 1))?;
        let _ = stdout.write_all(b"|");
        execute!(stdout, MoveTo(0, self.p1y))?;
        let _ = stdout.write_all(b"|");
        execute!(stdout, MoveTo(0, self.p1y - 1))?;
        let _ = stdout.write_all(b"|");
        execute!(stdout, MoveTo(WIDTH, self.p2y + 1))?;
        let _ = stdout.write_all(b"|");
        execute!(stdout, MoveTo(WIDTH, self.p2y))?;
        let _ = stdout.write_all(b"|");
        execute!(stdout, MoveTo(WIDTH, self.p2y - 1))?;
        let _ = stdout.write_all(b"|");

        execute!(stdout, MoveTo(self.bx, self.by))?;
        let _ = stdout.write_all(b"O");
        
        stdout.flush()?;

        Ok(())
    }

}

fn main() -> io::Result<()> {
    let mut game_state = GameState::new();
    let mut i = 0;

    let (input_tx, input_rx) = mpsc::channel();

    std::thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap() {
                if let event::Event::Key(key_event) = event::read().unwrap() {
                    let _ = input_tx.send(key_event);
                }
            }
        }
    });

    loop {
        game_state.update_paddles(&input_rx);
        game_state.update_ball();
        game_state.render()?;

        if i == 1000 {
            break;
        }
        i += 1;
        thread::sleep(Duration::from_millis(100));
    }
    
    {
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All))?; 
        if let Err(err) = execute!(stdout, Show) {
            eprintln!("Failed to show cursor: {}", err);
        }
    }

    println!("");

    Ok(())

}



