#[cfg(test)]
mod tests {
    use games::games::connect_four::ConnectFour;
    use games::games::tic_tac_toe::TicTacToe;
    // use games::strategy::common::{FirstPossibleMove, RandomMove};
    use games::strategy::minimax::Minimax;
    use games::{Competition, Game, PlayerType};

    #[test]
    fn minimax_tictactoe_first_two_moves() {
        let game = TicTacToe::new();
        let depths = 9;

        let first_player = PlayerType::AI(Minimax::new(depths));
        let second_player = PlayerType::AI(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);

        let mut player = competition.determine_player();
        let mut chosen_move = competition
            .get_move_for_player(player)
            .expect("Should be able to get a move");
        competition.game.apply_move(chosen_move);
        assert!(
            competition.game.board.x_board & (1 + 4 + 16 + 64 + 256) > 0,
            "First move of first player must be either a corner or the center."
        );

        player = competition.determine_player();
        chosen_move = competition.get_move_for_player(player).unwrap();
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

        let first_player = PlayerType::AI(Minimax::new(depths));
        let second_player = PlayerType::AI(Minimax::new(depths));

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
    fn minimax_connect_four_first_move() {
        let game = ConnectFour::new();
        let depths = 15;

        let first_player = PlayerType::AI(Minimax::new(depths));
        let second_player = PlayerType::AI(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);
        let mut player = competition.determine_player();
        let mut chosen_move = competition
            .get_move_for_player(player)
            .expect("Should be able to get a move");
        competition.game.apply_move(chosen_move);

        assert!(
            competition.game.board.both & 1 << 21 > 0,
            "First move of first player must be in the center (3) column."
        )
    }

    #[test]
    fn minimax_connect_four_draw() {
        let game = ConnectFour::new();
        let depths = 9;

        let first_player = PlayerType::AI(Minimax::new(depths));
        let second_player = PlayerType::AI(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(
            competition.game.get_winner().is_some(),
            "A Minimax duel must result in a draw."
        );
    }

    // TODO: Implement the possiblity of different strategies.
    //
    // #[test]
    // fn minimax_connect_four_beat_random() {
    //     let game = ConnectFour::new();
    //     let depths = 10;

    //     let first_player = PlayerType::AI(Minimax::new(depths));
    //     let second_player = PlayerType::AI(FirstPossibleMove);

    //     let mut competition = Competition::new(game, first_player, second_player);
    //     competition.start(false);

    //     assert!(competition.game.get_winner().is_none());
    // }
}
