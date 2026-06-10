use common::search::minimax::Minimax;
use common::{Competition, PlayerType};
use tic_tac_toe::TicTacToe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = TicTacToe::new();

    let first_player = PlayerType::Minimax(Minimax::new(9));
    let second_player = PlayerType::Human;

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true)?;

    Ok(())
}
