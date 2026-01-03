use crate::Game;

use rand::seq::SliceRandom;

pub struct RandomMove;

impl Strategy for RandomMove {
    fn compute_move<G: Game>(&self, game: &G) -> usize {
        let mut rng = rand::thread_rng();
        let moves: Vec<usize> = game.get_possible_moves().collect();
        moves.choose(&mut rng).copied().expect("No moves available")
    }
}

pub trait Strategy {
    fn compute_move<G: Game>(&self, game: &G) -> usize;
}
