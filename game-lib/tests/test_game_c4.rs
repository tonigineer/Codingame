#[cfg(test)]
mod tests {
    use games::games::c4::{self, ConnectFour};
    use games::strategy::common::{FirstPossibleMove, RandomMove};
    use games::{Competition, Game, PlayerType};

    #[test]
    fn test_connect_four_always_first_move() {
        let game = ConnectFour::new();

        let first_player = PlayerType::AI(FirstPossibleMove);
        let second_player = PlayerType::AI(FirstPossibleMove);
        let mut competition = Competition::new(game, first_player, second_player);
        competition.start(true);

        assert!(competition.game.get_winner().is_some());
        assert!(competition.game.get_winner().unwrap() == c4::PlayerMask::Red);
    }

    #[test]
    fn test_connect_four_random_moves() {
        let game = ConnectFour::new();

        let first_player = PlayerType::AI(RandomMove);
        let second_player = PlayerType::AI(RandomMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition.start(true);

        assert!(competition.game.get_winner().is_some() || competition.game.get_winner().is_none());
    }
}
