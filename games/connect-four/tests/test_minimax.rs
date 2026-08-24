use common::search::Strategy;
use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::search::minimax::Minimax;
use common::{Competition, Game};
use connect_four::{ConnectFour, PlayerMask};

/// The bottom cell of the center column (column 3 of 7, stride H+1 = 7).
const CENTER_COLUMN_BOTTOM: u64 = 1 << 21;

#[test]
fn minimax_connect_four_first_move() {
    let game = ConnectFour::<7, 6>::new();
    let depth = 15; // 10 moves are not enough to predict center move

    let first_player = Minimax::new(depth);
    let second_player = Minimax::new(depth);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .play_turn()
        .expect("Should be able to get a move");

    assert!(
        competition.game.board.both & CENTER_COLUMN_BOTTOM > 0,
        "First move of first player must be in the center (3) column."
    );
}

/// Replay `columns` from an empty board, alternating players.
fn position(columns: &[usize]) -> ConnectFour<7, 6> {
    let mut game = ConnectFour::<7, 6>::new();
    for &col in columns {
        game.apply_move(col);
    }
    game
}

/// Red owns the bottom of columns 0-2 and Yellow has stacked three in column
/// 6, so both sides are one move from winning and it is Red's turn. Taking
/// the win outscores blocking, and every other move hands Yellow the game,
/// which makes column 3 the unique best move — no tie set, nothing for the
/// random tie-break to pick between.
#[test]
fn minimax_connect_four_takes_the_win() {
    let mut game = position(&[0, 6, 1, 6, 2, 6]);
    assert_eq!(game.get_current_player(), PlayerMask::Red);

    let mut red = Minimax::new(8);
    let chosen = red
        .compute_move(&game)
        .expect("Should be able to get a move");
    assert_eq!(chosen, 3, "Minimax must complete its own four in a row.");

    game.apply_move(chosen);
    assert_eq!(game.get_winner(), Some(PlayerMask::Red));
}

/// Yellow holds the bottom of columns 0-2, whose only completion is column 3.
/// Red has no win of its own, so every move but the block loses at once —
/// again a single best move rather than a tie set.
#[test]
fn minimax_connect_four_blocks_the_threat() {
    let game = position(&[4, 0, 5, 1, 4, 2]);
    assert_eq!(game.get_current_player(), PlayerMask::Red);

    let mut red = Minimax::new(8);
    let chosen = red
        .compute_move(&game)
        .expect("Should be able to get a move");
    assert_eq!(chosen, 3, "Minimax must block Yellow's four in a row.");
}

#[test]
fn minimax_connect_four_beat_first_possible_move() {
    let game = ConnectFour::<7, 6>::new();
    let depth = 10;

    let first_player = Minimax::new(depth);
    let second_player = FirstPossibleMove;

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

    let first_player = Minimax::new(depth);
    let second_player = RandomMove;

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
