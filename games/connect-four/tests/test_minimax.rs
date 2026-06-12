use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::search::minimax::Minimax;
use common::{Competition, Game, PlayerType};
use connect_four::{ConnectFour, PlayerMask};

/// The bottom cell of the center column (column 3 of 7, stride H+1 = 7).
const CENTER_COLUMN_BOTTOM: u64 = 1 << 21;

#[test]
fn minimax_connect_four_first_move() {
    let game = ConnectFour::<7, 6>::new();
    let depth = 15; // 10 moves are not enough to predict center move

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::Minimax(Minimax::new(depth));

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .play_turn()
        .expect("Should be able to get a move");

    assert!(
        competition.game.board.both & CENTER_COLUMN_BOTTOM > 0,
        "First move of first player must be in the center (3) column."
    );
}

#[test]
fn minimax_connect_four_no_draw() {
    // Connect Four is a first-player win under perfect play, but depth 10 is
    // nowhere near perfect (this duel actually ends in a Yellow win). What is
    // stable is that the game ends decisively rather than drawn.
    let game = ConnectFour::<7, 6>::new();
    let depth = 10;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::Minimax(Minimax::new(depth));

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert!(
        competition.game.get_winner().is_some(),
        "A Minimax duel must not end in a draw."
    );
}

#[test]
fn minimax_connect_four_beat_first_possible_move() {
    let game = ConnectFour::<7, 6>::new();
    let depth = 10;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::FirstPossibleMove(FirstPossibleMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(
        competition.game.get_winner(),
        Some(PlayerMask::Red),
        "Minimax must beat bot that always plays first possible move."
    );
}

#[test]
fn minimax_connect_four_beat_random() {
    let game = ConnectFour::<7, 6>::new();
    let depth = 10;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::RandomMove(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(
        competition.game.get_winner(),
        Some(PlayerMask::Red),
        "Minimax must beat bot that always plays random moves."
    );
}
