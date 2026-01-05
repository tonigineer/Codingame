#[cfg(test)]
mod tests {
    use games::games::ttt::{self, TicTacToe};
    use games::strategy::common::{FirstPossibleMove, RandomMove};
    use games::{Competition, Game, PlayerType};

    #[test]
    fn test_tictactoe_always_first_move() {
        let game = TicTacToe::new();

        let first_player = PlayerType::AI(FirstPossibleMove);
        let second_player = PlayerType::AI(FirstPossibleMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition.start(true);

        assert!(competition.game.get_winner().is_some());
        assert!(competition.game.get_winner().unwrap() == ttt::PlayerMask::X);
    }

    #[test]
    fn test_tictactoe_random_moves() {
        let game = TicTacToe::new();

        let first_player = PlayerType::AI(RandomMove);
        let second_player = PlayerType::AI(RandomMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition.start(true);

        assert!(competition.game.get_winner().is_some() || competition.game.get_winner().is_none());
    }
}
