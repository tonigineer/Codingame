pub mod common;
pub mod minimax;

use crate::Game;

pub trait Strategy {
    fn compute_move<G: Game>(&self, game: &G) -> G::Move
    where
        G: Clone,
        G::Move: Clone,
        <G as Game>::PlayerMask: Eq;
}
