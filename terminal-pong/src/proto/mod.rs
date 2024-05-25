/*mod message;
pub use message::Game_State;

use std::fmt;

impl fmt::Display for Game_State {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "----- GAME STATE -----")?;
        writeln!(fmt, "Bx: {}", self.get_bx())?;
        writeln!(fmt, "By: {}", self.get_by())?;
        writeln!(fmt, "Bdx: {}", self.get_bdx())?;
        writeln!(fmt, "Bdy: {}", self.get_bdy())?;
        writeln!(fmt, "p1y: {}", self.get_p1y())?;
        writeln!(fmt, "p2y: {}", self.get_p2y())?;
        writeln!(fmt, "p1points: {}", self.get_p1points())?;
        writeln!(fmt, "p2points: {}", self.get_p2points())?;
        write!(fmt, "-----------------------")
    }
}

mod test {
    use super::*;

    #[test]
    fn test_display_gamestate() {
        let gamestate = Game_State {
            bx: 1,
            by: 5,
            bdx: 5,
            bdy: 5,
            p1y: 5,
            p2y: 5,
            p1points: 0,
            p2points: 2,
        };
        let formatted = format!("{}", gamestate);
        assert_eq!(
            formatted,
            r#"----- GAME STATE -----
Bx: 1
By: 5
Bdx: 5
Bdy: 5
p1y: 5
p2y: 5
p1points: 0
p2points; 2
-----------------------"#
    );
    }
}*/
