use common::Competition;
use common::search::baseline::HumanPlayer;
use common::search::minimax::Minimax;
use connect_four::ConnectFour;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = ConnectFour::<7, 6>::new();

    let mut competition = Competition::new(game, Minimax::new(15), HumanPlayer);
    competition.start(true)?;

    Ok(())
}
