use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::search::minimax::Minimax;
use common::{Competition, Game, PlayerType};
use tic_tac_toe::{PlayerMask, TicTacToe};

/// The corners and the center: the only optimal opening squares.
const CORNERS_AND_CENTER: u16 = 0b1_0101_0101;
const CENTER: u16 = 0b0_0001_0000;

#[test]
fn minimax_tictactoe_first_two_moves() {
    let game = TicTacToe::new();
    let depth = 9;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::Minimax(Minimax::new(depth));

    let mut competition = Competition::new(game, first_player, second_player);

    competition
        .play_turn()
        .expect("Should be able to get a move");
    assert!(
        competition.game.board.x_board & CORNERS_AND_CENTER > 0,
        "First move of first player must be either a corner or the center."
    );

    competition
        .play_turn()
        .expect("Should be able to get a move");
    assert!(
        competition.game.board.o_board & CORNERS_AND_CENTER > 0,
        "First move of second player must be either a corner or the center."
    );

    assert!(
        (competition.game.board.x_board | competition.game.board.o_board) & CENTER > 0,
        "One of first two moves must be in the center."
    );
}

#[test]
fn minimax_tictactoe_draw() {
    let game = TicTacToe::new();
    let depth = 9;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::Minimax(Minimax::new(depth));

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert!(
        competition.game.get_winner().is_none(),
        "A Minimax duel must result in a draw."
    );
}

#[test]
fn minimax_tictactoe_beat_first_possible_move() {
    let game = TicTacToe::new();
    let depth = 9;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::FirstPossibleMove(FirstPossibleMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(
        competition.game.get_winner(),
        Some(PlayerMask::X),
        "Minimax must beat bot that always plays first possible move."
    );
}

#[test]
fn minimax_tictactoe_never_loses_to_random() {
    // Tic-tac-toe is a draw under perfect play, so against an (unseeded)
    // random opponent only a non-loss is guaranteed.
    let game = TicTacToe::new();
    let depth = 9;

    let first_player = PlayerType::Minimax(Minimax::new(depth));
    let second_player = PlayerType::RandomMove(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_ne!(
        competition.game.get_winner(),
        Some(PlayerMask::O),
        "Minimax must never lose to random play."
    );
}
