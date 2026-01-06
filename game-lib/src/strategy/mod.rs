pub mod common;
pub mod minimax;

use crate::{Game, GameError};

pub trait Strategy {
    fn compute_move<G>(&self, game: &G) -> Result<G::Move, GameError>
    where
        G: Game + Clone,
        G::Move: Clone,
        <G as Game>::PlayerMask: Eq;
}
