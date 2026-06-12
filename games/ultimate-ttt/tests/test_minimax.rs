use common::search::baseline::FirstPossibleMove;
use common::search::minimax::Minimax;
use common::{Competition, Game};
use ultimate_ttt::{PlayerMask, UltimateTicTacToe};

// Ultimate tic-tac-toe is nowhere near solvable, so unlike the tic-tac-toe
// suite there is no perfect-play assertion — only deterministic duels
// against the fixed baseline, from both sides.

#[test]
fn minimax_uttt_beats_first_possible_move_as_x() {
    let game = UltimateTicTacToe::new();

    let mut competition = Competition::new(game, Minimax::new(7), FirstPossibleMove);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(
        competition.game.get_winner(),
        Some(PlayerMask::X),
        "Minimax must beat the bot that always plays the first possible move."
    );
}

#[test]
fn minimax_uttt_beats_first_possible_move_as_o() {
    let game = UltimateTicTacToe::new();

    let mut competition = Competition::new(game, FirstPossibleMove, Minimax::new(7));
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert_eq!(
        competition.game.get_winner(),
        Some(PlayerMask::O),
        "Minimax must beat the baseline even from the second seat."
    );
}
