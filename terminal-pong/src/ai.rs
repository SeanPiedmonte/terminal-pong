//! The computer opponent. Nothing here is random: each difficulty differs in
//! how fast the paddle moves, how early it reacts and how well it predicts
//! where the ball will arrive.

use crate::physics::{Game, Player, DT, HEIGHT, LEFT_X, PADDLE_MAX, PADDLE_MIN, RIGHT_X, WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub const NAMES: [&'static str; 3] = ["easy", "medium", "hard"];

    pub fn parse(name: &str) -> Option<Difficulty> {
        match name {
            "easy" => Some(Difficulty::Easy),
            "medium" => Some(Difficulty::Medium),
            "hard" => Some(Difficulty::Hard),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }

    fn profile(self) -> Profile {
        match self {
            // Chases the ball's current row, slowly, once it is over halfway,
            // and returns it flat.
            Difficulty::Easy => Profile {
                paddle_speed: 7.0,
                react_within: 0.5,
                tracking: Tracking::FollowBall,
                aim_rows: 0.0,
                recentres: false,
            },
            // Works out where the ball will arrive, bounces included, but
            // starts moving late and angles its returns only a little.
            Difficulty::Medium => Profile {
                paddle_speed: 8.0,
                react_within: 0.6,
                tracking: Tracking::Predict,
                aim_rows: 0.5,
                recentres: false,
            },
            // Reacts early, strikes with the paddle edge to steer the ball
            // away from the opponent, and goes back to the middle between
            // shots. Beatable only by out-pacing its paddle late in a rally.
            Difficulty::Hard => Profile {
                paddle_speed: 9.0,
                react_within: 0.75,
                tracking: Tracking::Predict,
                aim_rows: 0.8,
                recentres: true,
            },
        }
    }
}

#[derive(Clone, Copy)]
enum Tracking {
    /// Go to the row the ball is on right now.
    FollowBall,
    /// Go to the row the ball will arrive on, allowing for wall bounces.
    Predict,
}

#[derive(Clone, Copy)]
struct Profile {
    /// Rows per second.
    paddle_speed: f32,
    /// React once the ball is within this fraction of the field's width.
    react_within: f32,
    tracking: Tracking,
    /// How far from the paddle's centre it tries to strike the ball, to
    /// angle the return away from the opponent. Zero returns it flat.
    aim_rows: f32,
    recentres: bool,
}

pub struct Ai {
    difficulty: Difficulty,
    /// Fractional rows of movement carried between ticks.
    budget: f32,
}

impl Ai {
    pub fn new(difficulty: Difficulty) -> Ai {
        Ai { difficulty, budget: 0.0 }
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// The row this AI wants its paddle centre on, or `None` to stay put.
    pub fn target(&self, game: &Game, me: Player) -> Option<f32> {
        let p = self.difficulty.profile();
        let (plane, incoming) = match me {
            Player::One => (LEFT_X, game.vx < 0.0),
            Player::Two => (RIGHT_X, game.vx > 0.0),
        };
        let centre = (HEIGHT - 1.0) / 2.0;

        // Everyone walks back to the middle while the ball waits to be served.
        if game.serve_in > 0 {
            return Some(centre);
        }
        if !incoming {
            return p.recentres.then_some(centre);
        }
        if (plane - game.bx).abs() > p.react_within * WIDTH {
            return None;
        }

        let arrival = match p.tracking {
            Tracking::FollowBall => game.by,
            Tracking::Predict => game.predict_y_at(plane)?,
        };

        // Striking above centre sends the ball up, below sends it down.
        // Send it to whichever half the opponent's paddle is not in.
        let opponent_is_low = game.paddle(me.other()) as f32 > centre;
        Some(if opponent_is_low { arrival + p.aim_rows } else { arrival - p.aim_rows })
    }

    /// Decide this tick's paddle movement: -1, 0 or 1 rows.
    pub fn decide(&mut self, game: &Game, me: Player) -> i32 {
        let p = self.difficulty.profile();
        // Carry fractional rows between ticks, but never bank more than one
        // spare row while idle, so the paddle cannot lunge after waiting.
        let gain = p.paddle_speed * DT;
        self.budget = (self.budget + gain).min(1.0 + gain);

        let Some(target) = self.target(game, me) else {
            return 0;
        };
        let target = (target.round() as i32).clamp(PADDLE_MIN, PADDLE_MAX);
        let delta = target - game.paddle(me);
        if delta == 0 || self.budget < 1.0 {
            return 0;
        }
        self.budget -= 1.0;
        delta.signum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::{Event, BASE_SPEED, TICK_HZ};

    /// Play AI against AI and return (player one points, player two points).
    fn play(one: Difficulty, two: Difficulty, seconds: u32) -> (u32, u32) {
        let mut game = Game::new();
        let (mut a, mut b) = (Ai::new(one), Ai::new(two));
        for _ in 0..seconds * TICK_HZ {
            let d1 = a.decide(&game, Player::One);
            let d2 = b.decide(&game, Player::Two);
            game.move_paddle(Player::One, d1);
            game.move_paddle(Player::Two, d2);
            game.step();
            if game.winner().is_some() {
                break;
            }
        }
        (game.p1points, game.p2points)
    }

    /// Stand-in for a strong human: player one is always exactly where the
    /// ball arrives and strikes it near the paddle edge, away from the AI.
    /// Returns how many balls the AI got back before each point it lost.
    fn returns_against_a_sharpshooter(difficulty: Difficulty) -> Vec<u32> {
        let mut game = Game::new();
        let mut ai = Ai::new(difficulty);
        let (mut rallies, mut returns) = (Vec::new(), 0);
        for _ in 0..600 * TICK_HZ {
            if let Some(y) = game.predict_y_at(LEFT_X) {
                let aim = if game.p2y as f32 > (HEIGHT - 1.0) / 2.0 { 0.9 } else { -0.9 };
                game.p1y = ((y + aim).round() as i32).clamp(PADDLE_MIN, PADDLE_MAX);
            }
            let dy = ai.decide(&game, Player::Two);
            game.move_paddle(Player::Two, dy);
            for event in game.step() {
                match event {
                    Event::Hit { player: Player::Two, .. } => returns += 1,
                    Event::Miss { player: Player::Two, .. } => {
                        rallies.push(returns);
                        returns = 0;
                    }
                    Event::Miss { player: Player::One, .. } => panic!("the sharpshooter should never miss"),
                    _ => {}
                }
            }
            if game.winner().is_some() {
                break;
            }
        }
        rallies
    }

    #[test]
    fn each_difficulty_survives_longer_against_a_strong_player_than_the_one_below() {
        let easy = returns_against_a_sharpshooter(Difficulty::Easy);
        let medium = returns_against_a_sharpshooter(Difficulty::Medium);
        let hard = returns_against_a_sharpshooter(Difficulty::Hard);

        let best = |r: &Vec<u32>| *r.iter().max().unwrap();
        let worst = |r: &Vec<u32>| *r.iter().min().unwrap();
        assert!(best(&easy) < worst(&medium), "easy {easy:?} vs medium {medium:?}");
        assert!(best(&medium) < worst(&hard), "medium {medium:?} vs hard {hard:?}");
        // Hard is meant to be beatable: it does lose points in the end.
        assert_eq!(hard.len(), 10, "hard should lose the match to a perfect player: {hard:?}");
    }

    #[test]
    fn hard_beats_the_other_difficulties_from_either_side() {
        for weak in [Difficulty::Easy, Difficulty::Medium] {
            let (hard, other) = play(Difficulty::Hard, weak, 600);
            assert!(hard == 10 && other < 10, "hard as player one: {hard}-{other} against {weak:?}");
            let (other, hard) = play(weak, Difficulty::Hard, 600);
            assert!(hard == 10 && other < 10, "hard as player two: {hard}-{other} against {weak:?}");
        }
    }

    #[test]
    fn the_ai_is_deterministic() {
        assert_eq!(play(Difficulty::Medium, Difficulty::Easy, 120), play(Difficulty::Medium, Difficulty::Easy, 120));
    }

    #[test]
    fn easy_follows_the_ball_while_medium_and_hard_read_a_bank_shot() {
        let mut game = Game::new();
        game.serve_in = 0;
        game.bx = 45.0;
        game.by = 4.0;
        game.vx = BASE_SPEED;
        game.vy = -14.0; // up into the top wall, then down across the field
        game.p1y = 2; // opponent is high, so an aiming AI strikes below centre

        let truth = game.predict_y_at(RIGHT_X).unwrap();
        assert!((truth - game.by).abs() > 5.0, "the shot should end far from where it is now");

        let target = |d| Ai::new(d).target(&game, Player::Two).unwrap();
        assert_eq!(target(Difficulty::Easy), game.by);
        assert!((target(Difficulty::Medium) - (truth - 0.5)).abs() < 1e-3);
        assert!((target(Difficulty::Hard) - (truth - 0.8)).abs() < 1e-3);
    }

    #[test]
    fn reaction_distance_grows_with_difficulty() {
        let mut game = Game::new();
        game.serve_in = 0;
        game.vx = BASE_SPEED;
        game.by = 3.0;
        let reacts = |d, bx: f32, game: &mut Game| {
            game.bx = bx;
            Ai::new(d).target(game, Player::Two).is_some()
        };
        // Ball a quarter of the way across: only hard has started moving.
        assert!(!reacts(Difficulty::Easy, 20.0, &mut game));
        assert!(!reacts(Difficulty::Medium, 20.0, &mut game));
        assert!(reacts(Difficulty::Hard, 20.0, &mut game));
        // Just under halfway: medium joins in.
        assert!(!reacts(Difficulty::Easy, 35.0, &mut game));
        assert!(reacts(Difficulty::Medium, 35.0, &mut game));
        // Well past halfway: everyone is tracking it.
        assert!(reacts(Difficulty::Easy, 60.0, &mut game));
    }

    #[test]
    fn everyone_recentres_for_a_serve_but_only_hard_recentres_mid_rally() {
        let mut game = Game::new();
        let centre = Some((HEIGHT - 1.0) / 2.0);
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            assert_eq!(Ai::new(d).target(&game, Player::Two), centre);
        }
        game.serve_in = 0;
        game.vx = -BASE_SPEED; // heading away from player two
        assert_eq!(Ai::new(Difficulty::Easy).target(&game, Player::Two), None);
        assert_eq!(Ai::new(Difficulty::Medium).target(&game, Player::Two), None);
        assert_eq!(Ai::new(Difficulty::Hard).target(&game, Player::Two), centre);
    }

    #[test]
    fn paddle_speed_is_limited_and_grows_with_difficulty() {
        // Ask each AI to cross the whole field and count rows moved in two seconds.
        let rows_moved = |d: Difficulty| {
            let mut game = Game::new();
            game.serve_in = 0;
            game.bx = 70.0;
            game.by = 0.0;
            game.vx = 0.001; // incoming, but effectively parked
            game.vy = 0.0;
            game.p2y = PADDLE_MAX;
            let mut ai = Ai::new(d);
            let mut moved = 0;
            for _ in 0..2 * TICK_HZ {
                let dy = ai.decide(&game, Player::Two);
                assert!(dy.abs() <= 1, "never more than one row per tick");
                game.move_paddle(Player::Two, dy);
                moved += dy.abs();
            }
            moved
        };
        assert_eq!(rows_moved(Difficulty::Easy), 14);
        assert_eq!(rows_moved(Difficulty::Medium), 16);
        assert_eq!(rows_moved(Difficulty::Hard), 17, "hard reaches the far wall and stops");
    }
}
