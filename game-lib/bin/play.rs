use game_lib::games::{connect_four::ConnectFour, tic_tac_toe::TicTacToe};
use game_lib::strategy::minimax::Minimax;
use game_lib::{Competition, PlayerType};

fn play_tic_tac_toe() -> Result<(), Box<dyn std::error::Error>> {
    let game = TicTacToe::new();

    let first_player = PlayerType::Minimax(Minimax::new(9));
    let second_player = PlayerType::Human;

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true)?;

    Ok(())
}

fn play_connect_four() -> Result<(), Box<dyn std::error::Error>> {
    let game = ConnectFour::<7, 6>::new();

    let first_player = PlayerType::Minimax(Minimax::new(15));
    let second_player = PlayerType::Human;

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let game = if args.len() > 2 && args[1] == "--game" {
        &args[2]
    } else {
        "tic-tac-toe"
    };

    match game {
        "connect-four" => play_connect_four()?,
        "tic-tac-toe" => play_tic_tac_toe()?,
        _ => {
            eprintln!("Usage: {} [--game <game>]", args[0]);
            eprintln!("Games: connect-four, tic-tac-toe");
            std::process::exit(1);
        }
    }

    Ok(())
}
