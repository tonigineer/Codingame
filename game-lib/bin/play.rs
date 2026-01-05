use games::games::c4::ConnectFour;
use games::games::ttt::TicTacToe;
use games::strategy::common::{prompt_user_move, RandomMove};
use games::strategy::minimax::Minimax;
use games::strategy::Strategy;
use games::Game;

pub enum PlayerType<S: Strategy> {
    Human,
    AI(S),
}

struct Competition<G: Game, S: Strategy> {
    game: G,
    first_player: PlayerType<S>,
    second_player: PlayerType<S>,
}

impl<G: Game<Move = usize>, S: Strategy> Competition<G, S>
where
    G: Clone,
    <G as Game>::PlayerMask: Eq,
{
    fn new(game: G, first_player: PlayerType<S>, second_player: PlayerType<S>) -> Self {
        Competition {
            game,
            first_player,
            second_player,
        }
    }

    fn start(&mut self, render_game: bool) {
        while !self.game.is_finished() {
            if render_game {
                self.game.render();
            }

            let player = self.determine_player();
            let chosen_move = self.get_move_for_player(player);
            self.game.apply_move(chosen_move);
        }

        if render_game {
            self.game.render();
        }
    }

    fn determine_player(&self) -> &PlayerType<S> {
        [&self.first_player, &self.second_player][self.game.get_current_player_index()]
    }

    fn get_move_for_player(&self, player: &PlayerType<S>) -> usize {
        match player {
            PlayerType::AI(strategy) => strategy.compute_move(&self.game),
            PlayerType::Human => prompt_user_move(&self.game),
        }
    }
}

fn play_tictactoe() {
    let game = TicTacToe::new();

    let first_player = PlayerType::Human;
    let second_player = PlayerType::AI(Minimax);
    // let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true);
}

fn play_connect_four() {
    let game = ConnectFour::new();

    let first_player = PlayerType::Human;
    let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true);
}

fn main() {
    play_tictactoe();
    // play_connect_four();
}

#[cfg(test)]
mod tests {
    use super::*;

    use games::games::c4::{self, ConnectFour};
    use games::games::ttt::{self, TicTacToe};
    use games::strategy::common::{FirstPossibleMove, RandomMove};
    use games::Game;

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
