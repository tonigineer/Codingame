use common::Competition;
use common::search::baseline::HumanPlayer;
use common::search::minimax::Minimax;
use ultimate_ttt::UltimateTicTacToe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = UltimateTicTacToe::new();

    let mut competition = Competition::new(game, Minimax::new(15), HumanPlayer);
    competition.start(true)?;

    Ok(())
}
