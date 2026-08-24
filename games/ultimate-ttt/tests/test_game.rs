use common::search::baseline::{FirstPossibleMove, RandomMove};
use common::{Competition, Game, Player};
use ultimate_ttt::{PlayerMask, UltimateTicTacToe};

/// X wins small board 4 with cells 0/1/2 while O answers with cell 4 each
/// time (which keeps sending X back to board 4); O's last reply then sends
/// X to the now-decided board 4.
const SCRIPT: [usize; 6] = [36, 4, 37, 13, 38, 22];

#[test]
fn test_uttt_always_first_move() {
    let game = UltimateTicTacToe::new();

    let first_player = FirstPossibleMove;
    let second_player = FirstPossibleMove;

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    // With center-first move ordering the deterministic duel plays out to
    // a full board with no main-board line: a draw.
    assert!(competition.game.is_finished());
    assert_eq!(competition.game.get_winner(), None);
}

#[test]
fn test_uttt_random_moves() {
    let game = UltimateTicTacToe::new();

    let first_player = RandomMove;
    let second_player = RandomMove;

    let mut competition = Competition::new(game, first_player, second_player);
    competition
        .start(false)
        .expect("Game should complete without errors");

    assert!(competition.game.is_finished());
}

#[test]
fn test_uttt_move_redirects_to_matching_board() {
    let mut game = UltimateTicTacToe::new();

    // X plays board 4, cell 7 — O must answer somewhere on board 7.
    game.apply_move(4 * 9 + 7);

    assert_eq!(game.constraint, Some(7));
    assert!(game.get_possible_moves().all(|m| m / 9 == 7));
    assert_eq!(game.get_possible_moves().count(), 9);
}

#[test]
fn test_uttt_won_board_closes_and_frees_the_reply() {
    let mut game = UltimateTicTacToe::new();

    for chosen_move in SCRIPT {
        game.apply_move(chosen_move);
    }

    // X owns board 4 on the main board, nothing else is decided.
    assert_eq!(game.macro_board.x_board, 1 << 4);
    assert_eq!(game.macro_board.o_board, 0);
    assert_eq!(game.get_winner(), None);

    // O's last cell pointed at board 4, which is won: X plays anywhere
    // open — every empty cell except the six left on board 4.
    assert_eq!(game.constraint, None);
    let moves: Vec<usize> = game.get_possible_moves().collect();
    assert!(moves.iter().all(|&m| m / 9 != 4));
    assert_eq!(moves.len(), 81 - SCRIPT.len() - 6);
}

#[test]
fn test_uttt_apply_undo_restores_position() {
    let mut game = UltimateTicTacToe::new();

    let mut hashes = vec![game.get_game_state_hash()];
    for chosen_move in SCRIPT {
        game.apply_move(chosen_move);
        hashes.push(game.get_game_state_hash());
    }

    // Undoing X's board-4 win must reopen the board on the main board too.
    for chosen_move in SCRIPT.iter().rev() {
        hashes.pop();
        game.undo_move(*chosen_move);
        assert_eq!(game.get_game_state_hash(), *hashes.last().unwrap());
    }

    assert_eq!(game.macro_board.occupied(), 0);
    assert_eq!(game.constraint, None);
    assert_eq!(game.current_player, PlayerMask::X);
    assert_eq!(game.get_possible_moves().count(), 81);
}

#[test]
fn test_uttt_small_board_win_marks_main_board() {
    let mut game = UltimateTicTacToe::new();

    // X takes small board 3 down its top row; O's replies (cell 3 each
    // time) keep sending X right back to board 3.
    let moves = [
        27, // X b3c0 -> board 0
        3,  // O b0c3 -> board 3
        28, // X b3c1 -> board 1
        12, // O b1c3 -> board 3
        29, // X b3c2 -> top row complete
    ];
    for chosen_move in moves {
        assert!(
            game.get_possible_moves().any(|m| m == chosen_move),
            "scripted move {chosen_move} must be legal"
        );
        game.apply_move(chosen_move);
    }

    // X owns board 3 on the main board; the game itself keeps running and
    // O is sent to board 2 (the cell X just played).
    assert_eq!(game.macro_board.x_board, 1 << 3);
    assert!(!game.is_finished());
    assert_eq!(game.constraint, Some(2));
}

#[test]
#[allow(clippy::float_cmp)] // heuristic values are exact multiples of 1/4096
fn test_uttt_evaluate_is_zero_sum() {
    let mut game = UltimateTicTacToe::new();
    assert_eq!(game.evaluate(), 0.0, "the empty position is symmetric");

    for chosen_move in SCRIPT {
        game.apply_move(chosen_move);
    }

    let for_mover = game.evaluate();
    game.current_player = game.current_player.other();
    let for_other = game.evaluate();

    assert_eq!(for_mover, -for_other);
}

#[test]
fn test_uttt_evaluate_rewards_won_boards() {
    let mut game = UltimateTicTacToe::new();

    for chosen_move in SCRIPT {
        game.apply_move(chosen_move);
    }

    // X owns small board 4 and it is X's turn: the position must score
    // positive for the side to move, and inside the heuristic band.
    let score = game.evaluate();
    assert!(score > 0.0);
    assert!(score < 1.0);
}

#[test]
fn test_uttt_macro_line_ends_the_game() {
    let mut game = UltimateTicTacToe::new();

    // Hand X small boards 0 and 1; the game must still be running, then a
    // played-out win of board 2 completes the main-board top row.
    game.boards[0].x_board = 0b000_000_111;
    game.macro_board.x_board = 0b011;
    assert!(!game.is_finished());

    game.boards[2].x_board = 0b000_000_011;
    game.constraint = Some(2);
    game.apply_move(2 * 9 + 2);

    assert_eq!(game.get_winner(), Some(PlayerMask::X));
    assert!(game.is_finished());
}
