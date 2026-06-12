#[cfg(test)]
mod tests {
    use common::search::minimax::Minimax;
    use common::{Competition, Game, PlayerType};
    use tic_tac_toe::TicTacToe;

    #[test]
    fn minimax_tictactoe_first_two_moves() {
        let game = TicTacToe::new();
        let depths = 9;

        let first_player = PlayerType::Minimax(Minimax::new(depths));
        let second_player = PlayerType::Minimax(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);

        let player_index = competition.determine_player_index();
        let player = if player_index == 0 {
            &mut competition.first_player
        } else {
            &mut competition.second_player
        };
        let mut chosen_move = Competition::get_move_for_player(player, &competition.game)
            .expect("Should be able to get a move");
        competition.game.apply_move(chosen_move);

        assert!(
            competition.game.board.x_board & (1 + 4 + 16 + 64 + 256) > 0,
            "First move of first player must be either a corner or the center."
        );

        let player_index = competition.determine_player_index();
        let player = if player_index == 0 {
            &mut competition.first_player
        } else {
            &mut competition.second_player
        };
        chosen_move = Competition::get_move_for_player(player, &competition.game).unwrap();
        competition.game.apply_move(chosen_move);

        assert!(
            competition.game.board.o_board & (1 + 4 + 16 + 64 + 256) > 0,
            "First move of second player must be either a corner or the center."
        );

        assert!(
            competition.game.board.x_board & 16 > 0 || competition.game.board.o_board & 16 > 0,
            "One of first two moves must be in the center."
        );
    }

    #[test]
    fn minimax_tictactoe_draw() {
        let game = TicTacToe::new();
        let depths = 9;

        let first_player = PlayerType::Minimax(Minimax::new(depths));
        let second_player = PlayerType::Minimax(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(
            competition.game.get_winner().is_none(),
            "A Minimax duel must result in a draw."
        );
    }
}
