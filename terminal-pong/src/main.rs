fn main() {
    const WIDTH: u32 = 80;
    const HEIGHT: u32 = 40;
    let mut game_board : [[char; 80]; 40] = [[' '; 80]; 40];
    let mut p1_pos: (u32, u32) = (HEIGHT / 20, 0);
    let mut p2_pos: (u32, u32) = (HEIGHT / 20, WIDTH - 1);
    game_board[p1_pos.0 as usize][p1_pos.1 as usize] = '|';
    game_board[p2_pos.0 as usize][p2_pos.1 as usize] = '|';
    let mut bx: i32;
    let mut by: i32;
    let mut bdx: i32;
    let mut bdy: i32;
    draw_game_board(&mut game_board);
    while true {


    }
}


fn draw_game_board(game_board: &mut [&mut [char]]) {
    for elm in 0..80 {
        println!("-");
    }

}

fn update_game_board() {
}

fn update_ball_position() {
}

fn update_p1_paddle_position() {
}

fn ai_paddle() {
}

fn win_condition() {
}


