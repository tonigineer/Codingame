#[cfg(test)]
mod tests {
    use game_lib::games::connect_four::{self, ConnectFour};
    use game_lib::strategy::common::{FirstPossibleMove, RandomMove};
    use game_lib::{Competition, Game, PlayerType};

    #[test]
    fn test_connect_four_always_first_move() {
        let game: ConnectFour<7, 6> = ConnectFour::new();

        let first_player = PlayerType::FirstPossibleMove(FirstPossibleMove);
        let second_player = PlayerType::FirstPossibleMove(FirstPossibleMove);
        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(competition.game.get_winner().is_some());
        assert!(competition.game.get_winner().unwrap() == connect_four::PlayerMask::Red);
    }

    #[test]
    fn test_connect_four_random_moves() {
        let game: ConnectFour<7, 6> = ConnectFour::new();

        let first_player = PlayerType::RandomMove(RandomMove);
        let second_player = PlayerType::RandomMove(RandomMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        // Game should have finished with a winner or draw
        assert!(competition.game.is_finished());
    }
}
