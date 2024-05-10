use std::io::{self, Write};
use crossterm::{execute, cursor::MoveTo, terminal::{Clear, ClearType}, cursor::{Hide, Show}};
use std::time::Duration;
use std::thread;

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
            p2y: HEIGHT / 2,
            bdx: 1,
            bdy: 0,
        }
    }

    fn update_ball(&mut self) {
        self.bx = (self.bx as i16 + self.bdx) as u16;
        self.by = (self.by as i16 + self.bdy) as u16;

        if self.bx >= WIDTH || self.bx <= 0 {
            self.bdx *= -1;
        }

        if self.by >= HEIGHT || self.by <= 0 {
            self.bdy *= -1;
        }
    }

    fn update_paddles(&mut self) {
        // change paddles based on what player inputs
        // some ai implementation
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        execute!(stdout, Hide)?;

        execute!(stdout, Clear(ClearType::All))?;

        execute!(stdout, MoveTo(0, self.p1y))?;
        let _ = stdout.write_all(b"|");
        execute!(stdout, MoveTo(WIDTH, self.p2y))?;
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

    loop {
        game_state.update_ball();
        game_state.render()?;

        if i == 1000 {
            break;
        }
        i += 1;
        thread::sleep(Duration::from_millis(40));
    }
    
    {
        let mut stdout = io::stdout();
        if let Err(err) = execute!(stdout, Show) {
            eprintln!("Failed to show cursor: {}", err);
        }
    }
    
    println!();
    Ok(())

}



