use games::games::c4::ConnectFour;
use games::games::ttt::TicTacToe;
use games::strategy::common::{RandomMove, Strategy};
use games::{strategy, Game};

pub enum PlayerType<S: Strategy> {
    Human,
    AI(S),
}

struct Competition<T: Game, S: Strategy> {
    game: T,
    first_player: PlayerType<S>,
    second_player: PlayerType<S>,
}

impl<T: Game, S: Strategy> Competition<T, S> {
    fn new(game: T, first_player: PlayerType<S>, second_player: PlayerType<S>) -> Self {
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
            PlayerType::AI(strategy) => match strategy {
                RandomMove => {
                    let s = RandomMove;
                    s.compute_move(&self.game)
                }
            },
            PlayerType::Human => self.game.get_possible_moves().next().unwrap(),
        }
    }
}

fn main() {
    let game = TicTacToe::new();

    // let first_player = PlayerType::Human;
    let first_player = PlayerType::AI(RandomMove);
    let second_player = PlayerType::AI(RandomMove);

    let mut competition = Competition::new(game, first_player, second_player);
    competition.start(true);

    // let first_player = PlayerType::Human;
    // let second_player = PlayerType::AI(Strategy::Random);

    // let game = ConnectFour::new();
    // let competition = Competition::new(game, first_player, second_player);
}
