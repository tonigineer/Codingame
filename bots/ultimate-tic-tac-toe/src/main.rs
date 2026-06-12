pub mod game;
pub mod parse;
pub mod strat;
pub mod types;

fn rc2idx(row: i32, col: i32) -> Result<usize, String> {
    let idx = row * 3 + col;
    usize::try_from(idx).map_err(|_| format!("Invalid index calculation from ({row}, {col})"))
}

fn idx2rc(idx: usize) -> Result<(i32, i32), String> {
    let idx_i32 =
        i32::try_from(idx).map_err(|_| format!("Index {idx} fits in usize but not i32"))?;
    Ok((idx_i32 / 3, idx_i32 % 3))
}

fn main() {
    use crate::game::Game;

    let mut game = game::TicTacToe::new();

    let max_depths = 9;
    let mut minimax = strat::Minimax::new(max_depths);

    loop {
        let (opp_move, moves) = parse::read_input();

        // Apply opponents move to keep track in own game
        if opp_move != (-1, -1) {
            game.apply_move(rc2idx(opp_move.0, opp_move.1).unwrap());
        }

        // Sanity check
        eprintln!("{moves:?}");
        eprintln!(
            "{:?}",
            game.get_possible_moves()
                .map(|m| idx2rc(m).unwrap())
                .collect::<Vec<(i32, i32)>>()
        );

        // Bot
        let mut game_temp = game.clone();
        let player = game.get_current_player();
        let mv = minimax.get_move(&mut game_temp, &player).unwrap();

        // Apply own move in own game
        game.apply_move(mv);

        let (row, col) = idx2rc(mv).unwrap();
        println!("{row} {col}");
    }
}
