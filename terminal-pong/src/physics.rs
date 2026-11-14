//! The rules of the game, with no I/O, so every rule can be unit tested.
//!
//! The ball has a float position and velocity. Its speed is never random:
//! it starts at `BASE_SPEED`, gains `SPEED_STEP` on every paddle hit and is
//! capped at `MAX_SPEED`. Its direction after a hit depends only on where it
//! struck the paddle: dead centre sends it back flat, the edges send it back
//! at up to `MAX_ANGLE_DEG`.

/// Playfield size in terminal cells. Columns run 0..WIDTH, rows 0..HEIGHT.
pub const WIDTH: f32 = 80.0;
pub const HEIGHT: f32 = 20.0;

/// The ball turns around at these columns; paddles are drawn just outside.
pub const LEFT_X: f32 = 1.0;
pub const RIGHT_X: f32 = WIDTH - 2.0;

/// A paddle is 3 rows tall. The ball counts as hit within this many rows of
/// the paddle's centre.
pub const PADDLE_REACH: f32 = 1.5;
pub const PADDLE_MIN: i32 = 1;
pub const PADDLE_MAX: i32 = HEIGHT as i32 - 2;

pub const TICK_HZ: u32 = 30;
pub const DT: f32 = 1.0 / TICK_HZ as f32;

/// Ball speed in cells per second.
pub const BASE_SPEED: f32 = 26.0;
pub const SPEED_STEP: f32 = 3.0;
pub const MAX_SPEED: f32 = 80.0;
pub const MAX_ANGLE_DEG: f32 = 50.0;
/// Terminal cells are about twice as tall as they are wide, so vertical
/// travel is halved to make angles look right on screen.
pub const VERTICAL_SCALE: f32 = 0.5;

pub const WIN_SCORE: u32 = 10;
/// Pause before each serve, so players can see where the ball starts.
pub const SERVE_PAUSE_TICKS: u32 = TICK_HZ;

const TOP: f32 = 0.0;
const BOTTOM: f32 = HEIGHT - 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}

impl Player {
    pub fn other(self) -> Player {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }

    pub fn number(self) -> u32 {
        match self {
            Player::One => 1,
            Player::Two => 2,
        }
    }

    pub fn from_number(n: u32) -> Option<Player> {
        match n {
            1 => Some(Player::One),
            2 => Some(Player::Two),
            _ => None,
        }
    }
}

/// Something worth logging that happened during a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The ball struck a paddle. `offset` is -1.0 (top edge) to 1.0 (bottom edge).
    Hit { player: Player, offset: f32, speed: f32 },
    /// The ball got past a paddle. `gap` is how many rows beyond the paddle's
    /// reach it was, so a small gap is a near miss.
    Miss { player: Player, ball_y: f32, paddle_y: i32, gap: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub bx: f32,
    pub by: f32,
    pub vx: f32,
    pub vy: f32,
    pub p1y: i32,
    pub p2y: i32,
    pub p1points: u32,
    pub p2points: u32,
    pub rally: u32,
    pub tick: u32,
    pub serve_in: u32,
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

/// Reflect `y` back into the playfield as if it had bounced off the top and
/// bottom walls any number of times.
pub fn fold_into_field(y: f32) -> f32 {
    let span = BOTTOM - TOP;
    let m = (y - TOP).rem_euclid(2.0 * span);
    TOP + if m > span { 2.0 * span - m } else { m }
}

impl Game {
    pub fn new() -> Game {
        let mut game = Game {
            bx: 0.0,
            by: 0.0,
            vx: 0.0,
            vy: 0.0,
            p1y: HEIGHT as i32 / 2,
            p2y: HEIGHT as i32 / 2,
            p1points: 0,
            p2points: 0,
            rally: 0,
            tick: 0,
            serve_in: 0,
        };
        game.serve(Player::Two);
        game
    }

    pub fn paddle(&self, player: Player) -> i32 {
        match player {
            Player::One => self.p1y,
            Player::Two => self.p2y,
        }
    }

    pub fn points(&self, player: Player) -> u32 {
        match player {
            Player::One => self.p1points,
            Player::Two => self.p2points,
        }
    }

    pub fn speed(&self) -> f32 {
        (BASE_SPEED + SPEED_STEP * self.rally as f32).min(MAX_SPEED)
    }

    pub fn winner(&self) -> Option<Player> {
        if self.p1points >= WIN_SCORE {
            Some(Player::One)
        } else if self.p2points >= WIN_SCORE {
            Some(Player::Two)
        } else {
            None
        }
    }

    /// Move a paddle by `dy` rows, clamped to the playfield.
    pub fn move_paddle(&mut self, player: Player, dy: i32) {
        let y = match player {
            Player::One => &mut self.p1y,
            Player::Two => &mut self.p2y,
        };
        *y = (*y + dy).clamp(PADDLE_MIN, PADDLE_MAX);
    }

    /// Put the ball on the centre spot, heading for `toward` after a pause.
    /// The serve angle alternates up and down with the score, so serves are
    /// varied but never random.
    fn serve(&mut self, toward: Player) {
        self.bx = (WIDTH - 1.0) / 2.0;
        self.by = BOTTOM / 2.0;
        self.rally = 0;
        self.serve_in = SERVE_PAUSE_TICKS;
        let up = (self.p1points + self.p2points) % 2 == 0;
        let angle = if up { -15.0_f32 } else { 15.0_f32 }.to_radians();
        let dir = if toward == Player::Two { 1.0 } else { -1.0 };
        self.vx = dir * BASE_SPEED * angle.cos();
        self.vy = BASE_SPEED * angle.sin() * VERTICAL_SCALE;
    }

    /// Where the ball will be (row) when it reaches column `x`, allowing for
    /// wall bounces. `None` if the ball is not travelling toward `x`.
    pub fn predict_y_at(&self, x: f32) -> Option<f32> {
        if self.vx == 0.0 {
            return None;
        }
        let t = (x - self.bx) / self.vx;
        if t < 0.0 {
            return None;
        }
        Some(fold_into_field(self.by + self.vy * t))
    }

    /// Advance the game by one tick.
    pub fn step(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        if self.winner().is_some() {
            return events;
        }
        self.tick += 1;
        if self.serve_in > 0 {
            self.serve_in -= 1;
            return events;
        }

        self.bx += self.vx * DT;
        self.by += self.vy * DT;

        if self.by < TOP {
            self.by = TOP + (TOP - self.by);
            self.vy = -self.vy;
        } else if self.by > BOTTOM {
            self.by = BOTTOM - (self.by - BOTTOM);
            self.vy = -self.vy;
        }

        if self.vx < 0.0 && self.bx <= LEFT_X {
            events.push(self.arrive(Player::One, LEFT_X));
        } else if self.vx > 0.0 && self.bx >= RIGHT_X {
            events.push(self.arrive(Player::Two, RIGHT_X));
        }
        events
    }

    /// The ball has reached `player`'s side: either they return it or they
    /// concede the point.
    fn arrive(&mut self, player: Player, plane: f32) -> Event {
        // A tick moves the ball up to a couple of cells, so it is already
        // past the paddle's column when we notice. Rewind to the row it was
        // on at the moment it crossed, and judge the hit there.
        let overshoot = (self.bx - plane) / self.vx;
        self.by = fold_into_field(self.by - self.vy * overshoot);

        let paddle_y = self.paddle(player);
        let distance = self.by - paddle_y as f32;

        if distance.abs() > PADDLE_REACH {
            let event = Event::Miss {
                player,
                ball_y: self.by,
                paddle_y,
                gap: distance.abs() - PADDLE_REACH,
            };
            match player.other() {
                Player::One => self.p1points += 1,
                Player::Two => self.p2points += 1,
            }
            self.serve(player);
            return event;
        }

        self.rally += 1;
        let speed = self.speed();
        let offset = (distance / PADDLE_REACH).clamp(-1.0, 1.0);
        let angle = (offset * MAX_ANGLE_DEG).to_radians();
        let dir = if player == Player::One { 1.0 } else { -1.0 };
        self.vx = dir * speed * angle.cos();
        self.vy = speed * angle.sin() * VERTICAL_SCALE;
        self.bx = plane + (plane - self.bx);
        Event::Hit { player, offset, speed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A game with the serve pause skipped and the ball one step from the
    /// left paddle, `offset_rows` below its centre.
    fn about_to_reach_left(offset_rows: f32) -> Game {
        let mut g = Game::new();
        g.serve_in = 0;
        g.p1y = 10;
        g.bx = LEFT_X + 0.5;
        g.by = 10.0 + offset_rows;
        g.vx = -BASE_SPEED;
        g.vy = 0.0;
        g
    }

    fn ball_speed(g: &Game) -> f32 {
        (g.vx.powi(2) + (g.vy / VERTICAL_SCALE).powi(2)).sqrt()
    }

    #[test]
    fn ball_waits_for_the_serve_pause() {
        let mut g = Game::new();
        let start = (g.bx, g.by);
        for _ in 0..SERVE_PAUSE_TICKS {
            g.step();
            assert_eq!((g.bx, g.by), start);
        }
        g.step();
        assert_ne!(g.bx, start.0);
    }

    #[test]
    fn centre_hit_returns_flat() {
        let mut g = about_to_reach_left(0.0);
        let events = g.step();
        assert!(matches!(events[0], Event::Hit { player: Player::One, .. }));
        assert!(g.vx > 0.0);
        assert_eq!(g.vy, 0.0);
    }

    #[test]
    fn edge_hits_return_at_an_angle_away_from_the_centre() {
        let mut low = about_to_reach_left(1.4);
        low.step();
        assert!(low.vx > 0.0 && low.vy > 0.0, "a low hit should send the ball down");

        let mut high = about_to_reach_left(-1.4);
        high.step();
        assert!(high.vx > 0.0 && high.vy < 0.0, "a high hit should send the ball up");

        let mut nearer_centre = about_to_reach_left(0.5);
        nearer_centre.step();
        assert!(low.vy > nearer_centre.vy, "further from centre means a steeper return");
    }

    #[test]
    fn speed_depends_only_on_rally_length_and_is_capped() {
        // The same hit twice gives the same result: nothing is random.
        let mut a = about_to_reach_left(0.7);
        let mut b = about_to_reach_left(0.7);
        a.step();
        b.step();
        assert_eq!(a, b);
        assert!((ball_speed(&a) - (BASE_SPEED + SPEED_STEP)).abs() < 1e-3);

        // Wherever the ball strikes the paddle, it leaves at the same speed.
        let mut edge = about_to_reach_left(1.4);
        edge.step();
        assert!((ball_speed(&edge) - ball_speed(&a)).abs() < 1e-3);

        let mut long = about_to_reach_left(0.0);
        long.rally = 1000;
        long.step();
        assert!((ball_speed(&long) - MAX_SPEED).abs() < 1e-3);
    }

    #[test]
    fn a_miss_scores_for_the_opponent_and_serves_to_the_loser() {
        let mut g = about_to_reach_left(4.0);
        g.rally = 6;
        let events = g.step();
        match events[0] {
            Event::Miss { player, paddle_y, gap, .. } => {
                assert_eq!(player, Player::One);
                assert_eq!(paddle_y, 10);
                assert!((gap - 2.5).abs() < 1e-3);
            }
            other => panic!("expected a miss, got {other:?}"),
        }
        assert_eq!((g.p1points, g.p2points), (0, 1));
        assert_eq!(g.rally, 0);
        assert_eq!(g.serve_in, SERVE_PAUSE_TICKS);
        assert!(g.vx < 0.0, "the next serve goes to the player who missed");
    }

    #[test]
    fn hit_box_edge_is_exactly_the_paddle_reach() {
        let mut inside = about_to_reach_left(PADDLE_REACH);
        assert!(matches!(inside.step()[0], Event::Hit { .. }));
        let mut outside = about_to_reach_left(PADDLE_REACH + 0.01);
        assert!(matches!(outside.step()[0], Event::Miss { .. }));
    }

    #[test]
    fn ball_bounces_off_the_walls_and_stays_in_the_field() {
        let mut g = Game::new();
        g.serve_in = 0;
        g.bx = 40.0;
        g.by = 0.2;
        g.vx = 5.0;
        g.vy = -20.0;
        g.step();
        assert!(g.by >= 0.0 && g.vy > 0.0);

        g.by = HEIGHT - 1.2;
        g.vy = 20.0;
        g.step();
        assert!(g.by <= HEIGHT - 1.0 && g.vy < 0.0);
    }

    #[test]
    fn prediction_matches_the_simulation_through_wall_bounces() {
        let mut g = Game::new();
        g.serve_in = 0;
        g.bx = 5.0;
        g.by = 3.0;
        g.vx = 30.0;
        g.vy = 21.0; // steep enough to bounce more than once on the way
        let predicted = g.predict_y_at(RIGHT_X).unwrap();
        g.p2y = -100; // out of the way, so arrival is recorded as a miss
        let arrived = loop {
            if let Some(Event::Miss { ball_y, .. }) = g.step().first().copied() {
                break ball_y;
            }
        };
        assert!((predicted - arrived).abs() < 0.01, "{predicted} vs {arrived}");
        assert_eq!(g.predict_y_at(0.0), None, "the ball is heading the other way");
    }

    #[test]
    fn paddles_are_clamped_to_the_field() {
        let mut g = Game::new();
        g.move_paddle(Player::One, -100);
        g.move_paddle(Player::Two, 100);
        assert_eq!((g.p1y, g.p2y), (PADDLE_MIN, PADDLE_MAX));
    }

    #[test]
    fn first_to_the_win_score_wins_and_the_game_stops() {
        let mut g = Game::new();
        g.p2points = WIN_SCORE - 1;
        g.serve_in = 0;
        g.p1y = 2;
        g.bx = LEFT_X + 0.5;
        g.by = 15.0;
        g.vx = -BASE_SPEED;
        g.step();
        assert_eq!(g.winner(), Some(Player::Two));
        let frozen = g.clone();
        g.step();
        assert_eq!(g, frozen);
    }

    #[test]
    fn fold_handles_several_bounces() {
        assert_eq!(fold_into_field(5.0), 5.0);
        assert_eq!(fold_into_field(-3.0), 3.0);
        assert_eq!(fold_into_field(HEIGHT - 1.0 + 4.0), HEIGHT - 1.0 - 4.0);
        assert_eq!(fold_into_field(2.0 * (HEIGHT - 1.0) + 6.0), 6.0);
    }
}
