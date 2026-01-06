pub mod common;
pub mod minimax;

use crate::Game;

pub trait Strategy {
    fn compute_move<G>(&self, game: &G) -> G::Move
    where
        G: Game + Clone,
        G::Move: Clone,
        <G as Game>::PlayerMask: Eq;
}
