use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::{Competition, Game};
use tic_tac_toe::{PlayerMask, TicTacToe};

#[test]
fn test_tictactoe_always_first_move() {
    let game = TicTacToe::new();

    let first_player = FirstPossibleMove;
    let second_player = FirstPossibleMove;

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(competition.game.get_winner(), Some(PlayerMask::X));
}

#[test]
fn test_tictactoe_random_moves() {
    let game = TicTacToe::new();

    let first_player = RandomMove;
    let second_player = RandomMove;

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert!(competition.game.is_finished());
}
