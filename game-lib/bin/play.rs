use games::games::c4::ConnectFour;
use games::games::ttt::TicTacToe;
use games::strategy::minimax::Minimax;
use games::{Competition, PlayerType};

fn play_tictactoe() -> Result<(), Box<dyn std::error::Error>> {
    let game = TicTacToe::new();

    let first_player = PlayerType::AI(Minimax::new(9));
    let second_player = PlayerType::Human;
    // let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true)?;
    Ok(())
}

fn play_connect_four() -> Result<(), Box<dyn std::error::Error>> {
    let game = ConnectFour::new();

    let first_player = PlayerType::AI(Minimax::new(15));
    let second_player = PlayerType::Human;
    // let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let game = if args.len() > 2 && args[1] == "--game" {
        &args[2]
    } else {
        "tictactoe"
    };

    match game {
        "connect-four" => play_connect_four()?,
        "tictactoe" => play_tictactoe()?,
        _ => {
            eprintln!("Usage: {} [--game <game>]", args[0]);
            eprintln!("Games: connect-four, tictactoe");
            std::process::exit(1);
        }
    }
    Ok(())
}
