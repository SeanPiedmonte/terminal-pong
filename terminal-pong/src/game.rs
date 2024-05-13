use std::io::{self, Write};
use crossterm::{cursor::{Hide, MoveTo, Show}, event::{self, KeyCode::{self, Char}}, execute, 
terminal::{self, Clear, ClearType, LeaveAlternateScreen, }};
use std::time::Duration;
use std::thread;
use rand::Rng;
const WIDTH: u16 = 80;
const HEIGHT: u16 = 20;

pub struct GameState {
    bx: u16,
    by: u16,
    p1y: u16,
    p2y: u16,
    bdx: i16,
    bdy: i16,
    p1points: u16,
    p2points: u16,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            bx: WIDTH / 2,
            by: HEIGHT / 2,
            p1y: HEIGHT / 2,
            p2y: HEIGHT / 2 + 1,
            bdx: 1,
            bdy: 1,
            p1points: 0,
            p2points: 0,
        }
    }

    fn update_ball(&mut self) {
        self.bx = (self.bx as i16 + self.bdx) as u16;
        if self.bx > 100 {
            self.bx = 0;
        } else if self.bx >= 80 { 
            self.bx = WIDTH;
        }
        
        self.by = (self.by as i16 + self.bdy) as u16;
        if self.by > 100 {
            self.by = 0;
        } else if self.by >= 20 {
            self.by = HEIGHT;
        }
        let mut rng = rand::thread_rng();
        let x_speed = rng.gen_range(1..5);
        let y_speed = rng.gen_range(1..5); 

        if (self.bx == 0 && (self.by == self.p1y + 1 || self.by == self.p1y - 1)) || 
           (self.bx == WIDTH && (self.by == self.p2y + 1 || self.by == self.p2y - 1)) {
            //self.bdx *= -1;
            self.bdx = if self.bdx < 0 {x_speed} else {x_speed * -1};
            self.bdy = if self.bdy < 0 {y_speed} else {y_speed * -1};
        } else if (self.by == self.p1y && self.bx == 0) || (self.bx == WIDTH && self.by == self.p2y) {
            self.bdx = if self.bdx > 1 {1} else {-1};
            self.bdx *= -1;
            self.bdy *= -1;
        } else if self.bx >= WIDTH || self.bx <= 0 {
            self.bdx *= -1;
        }

        if self.by >= HEIGHT || self.by <= 0 {
            self.bdy *= -1;
        }

        if self.scored() {
            self.bx = WIDTH / 2;
            self.by = HEIGHT / 2;
            self.bdx = 1;
            self.bdy = 1;
            self.p1y = HEIGHT / 2;
            self.p2y = HEIGHT / 2;
        }

    }


    fn update_paddles(&mut self) -> bool {
        while event::poll(Duration::from_millis(0)).unwrap() {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char('w') => self.p1y = if self.p1y - 1 <= 1 {1} else {self.p1y - 1},
                    KeyCode::Char('s') => self.p1y = (self.p1y + 1).min(HEIGHT - 1),
                    KeyCode::Esc => return false, 
                    _ => {}
                }
            }
        }
        true
    }

    fn ai_paddles(&mut self) {
        if self.bdx > 0 && self.bx > WIDTH / 2 {
            let future_ball = self.calc_future_ball_pos();
            if future_ball < self.p2y as i16 {
                self.p2y = if self.p2y - 1 <= 1 {1} else {self.p2y - 1};
            } else if future_ball > self.p2y as i16 {
                self.p2y = (self.p2y + 1).min(HEIGHT - 1);
            }
        }
    }

    fn calc_future_ball_pos(&self) -> i16 {
        let mut y_increment = self.bdy;
        let mut ball_pos : i16 = self.by as i16;
        let range = WIDTH - self.bx + 1;
        for _elm in 0..range {
            if ball_pos + 1 == HEIGHT as i16 || ball_pos - 1 == 0 {
                y_increment *= -1; 
            }
            ball_pos += y_increment;
        }
        
       ball_pos 
    }

    fn win_condition(&self) -> bool {
        let mut stdout = io::stdout();
        if self.p1points == 10 {
            let _ = stdout.write_all(b"Winner: PLAYER 1");
            return false;
        } else if self.p2points == 10 {
            let _ = stdout.write_all(b"YOU SUCK LOSER");
            return false;
        }
        true
    }

    fn scored(&mut self) -> bool {
        if self.bx == WIDTH && self.by != self.p2y && self.by != self.p2y + 1 && self.by != self.p2y - 1 {
            self.p1points += 1;
            return true;
        } else if self.bx == 0 && self.by != self.p1y && self.by != self.p1y + 1 && self.by != self.p1y - 1 {
            self.p2points += 1;
            return true;
        }
        false
    }


    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        execute!(stdout, Hide)?;

        execute!(stdout, Clear(ClearType::All))?;
        execute!(stdout, MoveTo(0, HEIGHT + 1))?;
        println!("SCORE: {} : {}", self.p1points, self.p2points);
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

    pub fn run_game(&self) -> io::Result<()> {
        let mut game_state = GameState::new();
        let mut stdout = io::stdout();
        terminal::enable_raw_mode().expect("Could not turn on raw mode");
        let mut running : bool = true;
        while running {
            if !game_state.win_condition() {
                break;
            }
            game_state.ai_paddles();
            running = game_state.update_paddles();
            game_state.update_ball();
            game_state.render()?;

            thread::sleep(Duration::from_millis(200));
        }

        terminal::disable_raw_mode().expect("Unable to exit raw mode"); 

        execute!(stdout, Show).expect("Unable to show cursor"); 
        execute!(stdout, Clear(ClearType::All))?;

        stdout.flush()?;
        println!();    
        Ok(())
    }

}

