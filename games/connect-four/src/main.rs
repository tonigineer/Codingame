use common::search::minimax::Minimax;
use common::{Competition, PlayerType};
use connect_four::ConnectFour;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = ConnectFour::<7, 6>::new();

    let first_player = PlayerType::Minimax(Minimax::new(15));
    let second_player = PlayerType::Human;

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true)?;

    Ok(())
}
