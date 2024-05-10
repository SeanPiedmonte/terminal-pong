use std::io::{self, Write};
use crossterm::{execute, cursor::MoveTo, terminal::{Clear, ClearType}};
use std::time::Duration;
use std::thread;

const WIDTH: u16 = 80;
const HEIGHT: u16 = 20;

struct Game_State {
    bx: u16,
    by: u16,
    p1y: u16,
    p2y: u16,
    bdx: i16,
    bdy: i16,
}

fn main() -> io::Result<(), Error> {
    let mut game_state = Game_State {
        bx : WIDTH / 2,
        by : HEIGHT / 2,
        p1y : HEIGHT / 2,
        p2y : HEIGHT / 2,
        bdx : 1,
        bdy : 0,
    };
    


    loop {
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;


    }

    Ok(());

}



