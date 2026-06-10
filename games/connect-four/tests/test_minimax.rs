#[cfg(test)]
mod tests {
    use common::search::baseline::{FirstPossibleMove, RandomMove};
    use common::search::minimax::Minimax;
    use common::{Competition, Game, PlayerType};
    use connect_four::{ConnectFour, PlayerMask};

    #[test]
    fn minimax_connect_four_first_move() {
        let game = ConnectFour::<7, 6>::new();
        let depths = 15; // 10 moves are not enough to predict center move

        let first_player = PlayerType::Minimax(Minimax::new(depths));
        let second_player = PlayerType::Minimax(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);
        let player_index = competition.determine_player_index();
        let player = if player_index == 0 {
            &mut competition.first_player
        } else {
            &mut competition.second_player
        };
        let chosen_move = Competition::get_move_for_player(player, &competition.game)
            .expect("Should be able to get a move");
        competition.game.apply_move(chosen_move);

        assert!(
            competition.game.board.both & 1 << 21 > 0,
            "First move of first player must be in the center (3) column."
        )
    }

    #[test]
    fn minimax_connect_four_draw() {
        let game = ConnectFour::<7, 6>::new();
        let depths = 10;

        let first_player = PlayerType::Minimax(Minimax::new(depths));
        let second_player = PlayerType::Minimax(Minimax::new(depths));

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(
            competition.game.get_winner().is_some(),
            "A Minimax duel must result in a draw."
        );
    }

    #[test]
    fn minimax_connect_four_beat_first_possible_move() {
        let game = ConnectFour::<7, 6>::new();
        let depths = 10;

        let first_player = PlayerType::Minimax(Minimax::new(depths));
        let second_player = PlayerType::FirstPossibleMove(FirstPossibleMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(
            competition.game.get_winner().unwrap() == PlayerMask::Red,
            "Minimax must beat bot that always plays first possible move."
        );
    }

    #[test]
    fn minimax_connect_four_beat_random() {
        let game = ConnectFour::<7, 6>::new();
        let depths = 10;

        let first_player = PlayerType::Minimax(Minimax::new(depths));
        let second_player = PlayerType::RandomMove(RandomMove);

        let mut competition = Competition::new(game, first_player, second_player);
        competition
            .start(false)
            .expect("Game should complete without errors");

        assert!(
            competition.game.get_winner().unwrap() == PlayerMask::Red,
            "Minimax must beat bot that always plays random moves."
        );
    }
}
