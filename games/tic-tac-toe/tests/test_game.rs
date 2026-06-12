use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::{Competition, Game, PlayerType};
use tic_tac_toe::{PlayerMask, TicTacToe};

#[test]
fn test_tictactoe_always_first_move() {
    let game = TicTacToe::new();

    let first_player = PlayerType::FirstPossibleMove(FirstPossibleMove);
    let second_player = PlayerType::FirstPossibleMove(FirstPossibleMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(competition.game.get_winner(), Some(PlayerMask::X));
}

#[test]
fn test_tictactoe_random_moves() {
    let game = TicTacToe::new();

    let first_player = PlayerType::RandomMove(RandomMove);
    let second_player = PlayerType::RandomMove(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert!(competition.game.is_finished());
}
