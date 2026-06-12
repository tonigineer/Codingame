use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::{Competition, Game};
use connect_four::{ConnectFour, PlayerMask};

#[test]
fn test_connect_four_always_first_move() {
    let game: ConnectFour<7, 6> = ConnectFour::new();

    let first_player = FirstPossibleMove;
    let second_player = FirstPossibleMove;

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(competition.game.get_winner(), Some(PlayerMask::Red));
}

#[test]
fn test_connect_four_random_moves() {
    let game: ConnectFour<7, 6> = ConnectFour::new();

    let first_player = RandomMove;
    let second_player = RandomMove;

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert!(competition.game.is_finished());
}
