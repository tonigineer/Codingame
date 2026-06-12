pub mod baseline;
pub mod minimax;

use crate::{Game, GameError};

/// Anything that can pick a move for the player currently to move.
///
/// The trait is object-safe, so a `Competition` can hold any mix of
/// strategies as `Box<dyn Strategy<G>>`.
pub trait Strategy<G: Game> {
    fn compute_move(&mut self, game: &G) -> Result<G::Move, GameError>;

    /// One-line summary of the last `compute_move`, reprinted by
    /// `Competition` under the board after every render (which clears the
    /// screen) so it stays visible while the opponent thinks.
    fn status_line(&self) -> Option<String> {
        None
    }
}
