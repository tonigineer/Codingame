pub mod games;
pub mod strategy;

pub trait Game {
    type PlayerMask;
    type Move: Copy + Clone;

    // TODO: Could be done better, by using the type PlayerMask of trait
    fn get_current_player_index(&self) -> usize;

    fn get_current_player_symbol(&self) -> char;

    fn get_current_player(&self) -> Self::PlayerMask;

    // Actual game
    fn apply_move(&mut self, chosen_move: Self::Move);

    fn undo_move(&mut self, chosen_move: Self::Move);

    fn get_possible_moves(&self) -> impl Iterator<Item = Self::Move>;

    fn is_finished(&self) -> bool;

    fn get_winner(&self) -> Option<Self::PlayerMask>;

    fn render(&self);

    // Additional information for strategies, could be a dedicated trait?
    fn get_game_state_score(&self, player: &Self::PlayerMask) -> f32;

    fn get_game_state_hash(&self) -> u64;
}
