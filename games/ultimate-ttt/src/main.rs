use std::time::Duration;

use common::Competition;
use common::search::baseline::HumanPlayer;
use common::search::minimax::Minimax;
use ultimate_ttt::UltimateTicTacToe;

/// Codingame gives ~100 ms per turn; iterative deepening spends that budget
/// and stops at whatever depth it reaches, instead of a fixed depth.
const TURN_BUDGET: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = UltimateTicTacToe::new();

    // let mut competition = Competition::new(game, Minimax::new(20).with_time_budget(TURN_BUDGET).with_status_bar(), HumanPlayer);
    let mut competition = Competition::new(
        game,
        Minimax::new(20).with_time_budget(TURN_BUDGET).with_status_bar(),
        Minimax::new(20).with_time_budget(TURN_BUDGET).with_status_bar(),
    );
    competition.start(true)?;

    Ok(())
}
