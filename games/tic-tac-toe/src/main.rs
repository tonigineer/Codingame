use common::Competition;
use common::search::baseline::HumanPlayer;
use common::search::minimax::Minimax;
use tic_tac_toe::TicTacToe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = TicTacToe::new();

    let mut competition = Competition::new(game, Minimax::new(9), HumanPlayer);
    competition.start(true)?;

    Ok(())
}
