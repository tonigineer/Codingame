#[cfg(test)]
mod tests {
    use games::games::tic_tac_toe::{self, TicTacToe};
    use games::strategy::common::{FirstPossibleMove, RandomMove};
    use games::{Competition, Game, PlayerType};

    #[test]
    fn test_tictactoe_always_first_move() {
        let game = TicTacToe::new();

        let first_player = PlayerType::FirstPossibleMove(FirstPossibleMove);
        let second_player = PlayerType::FirstPossibleMove(FirstPossibleMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(competition.game.get_winner().is_some());
        assert!(competition.game.get_winner().unwrap() == tic_tac_toe::PlayerMask::X);
    }

    #[test]
    fn test_tictactoe_random_moves() {
        let game = TicTacToe::new();

        let first_player = PlayerType::RandomMove(RandomMove);
        let second_player = PlayerType::RandomMove(RandomMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(competition.game.is_finished());
    }
}
