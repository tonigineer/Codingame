pub mod games;
pub mod strategy;

pub trait Game {
    type PlayerMask;

    fn get_current_player_index(&self) -> usize;

    fn get_current_player_symbol(&self) -> char;

    fn apply_move(&mut self, chosen_move: usize);

    fn undo_move(&mut self, chosen_move: usize);

    fn get_possible_moves(&self) -> impl Iterator<Item = usize>;

    fn is_finished(&self) -> bool;

    fn get_winner(&self) -> Option<Self::PlayerMask>;

    fn render(&self);
}
