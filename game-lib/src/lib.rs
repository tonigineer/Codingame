/// Generic trait for board games, to allow implementing other games in the future.
pub trait Game {
    type Player;
    type Move;

    fn first_move() -> Self;
    fn get_possible_moves(&self) -> Vec<Self::Move>;
    fn apply_move(&mut self, move_: Self::Move) -> Result<(), String>;
    fn undo_move(&mut self) -> Result<(), String>;
    fn current_player(&self) -> Self::Player;
    fn is_finished(&self) -> bool;
    fn get_winner(&self) -> Option<Self::Player>;
    fn render(&self);
}
