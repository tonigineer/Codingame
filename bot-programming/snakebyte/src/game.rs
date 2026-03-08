use crate::types::GameState;

pub struct Game {
    pub turn: u8,
    pub game_state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    #[must_use]
    pub fn new() -> Self {
        Self {
            turn: 0,
            game_state: GameState::new(),
        }
    }

    pub fn update(&mut self) {
        let other = self.game_state.clone();

        self.turn += 1;
        self.game_state.update();

        self.game_state.diff(&other);
    }

    pub fn apply_move(&self) {}
}
