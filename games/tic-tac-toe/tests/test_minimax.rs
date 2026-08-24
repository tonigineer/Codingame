use common::search::Strategy;
use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::search::minimax::Minimax;
use common::{Competition, Game};
use tic_tac_toe::{PlayerMask, TicTacToe};

/// Scores are floats, and a drawn line is a sum of exact zeros, but compare
/// with a tolerance rather than betting on that staying true.
const DRAW_EPS: f32 = 1e-6;

/// Tic-tac-toe is a draw under perfect play, and that holds for *every*
/// opening square — corners, center and edges alike. Edges are only weaker
/// against a fallible opponent, which a full-depth search is not. So the
/// invariant is the score, not the move: the root and all nine replies to it
/// must evaluate to zero. The search breaks ties at random, so which of the
/// nine it actually plays is deliberately not fixed.
#[test]
fn minimax_tictactoe_opening_is_drawn() {
    let game = TicTacToe::new();
    let depth = 9;

    let mut root = Minimax::new(depth);
    root.compute_move(&game)
        .expect("Should be able to get a move");
    assert!(
        root.move_score.abs() <= DRAW_EPS,
        "The empty board must evaluate as a draw, got {}.",
        root.move_score
    );

    for opening in game.get_possible_moves() {
        let mut opened = game.clone();
        opened.apply_move(opening);

        let mut reply = Minimax::new(depth);
        reply
            .compute_move(&opened)
            .expect("Should be able to get a move");
        assert!(
            reply.move_score.abs() <= DRAW_EPS,
            "Opening on square {opening} must still be a draw, got {}.",
            reply.move_score
        );
    }
}

#[test]
fn minimax_tictactoe_draw() {
    let game = TicTacToe::new();
    let depth = 9;

    let first_player = Minimax::new(depth);
    let second_player = Minimax::new(depth);

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

    let first_player = Minimax::new(depth);
    let second_player = FirstPossibleMove;

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

    let first_player = Minimax::new(depth);
    let second_player = RandomMove;

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
