use games::games::c4::ConnectFour;
use games::games::ttt::TicTacToe;
use games::strategy::common::RandomMove;
use games::strategy::minimax::Minimax;
use games::{Competition, PlayerType};

fn play_tictactoe() {
    let game = TicTacToe::new();

    let first_player = PlayerType::Human;
    let second_player = PlayerType::AI(Minimax::new(9));
    // let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true);
}

fn play_connect_four() {
    let game = ConnectFour::new();

    let first_player = PlayerType::Human;
    let second_player = PlayerType::AI(Minimax::new(9));
    // let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true);
}

fn main() {
    play_tictactoe();
    // play_connect_four();
}
