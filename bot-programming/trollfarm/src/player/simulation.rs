use crate::game::Game;
use crate::prediction::Predictable;

use super::Player;

impl Player {
    pub fn simulate(&mut self, game: &Game) {
        self.predicted = Some(game.snapshot(&self.actions, &[]));
    }

    pub fn compare(&self, game: &Game) {
        if let Some(snapshot) = &self.predicted {
            game.compare(snapshot);
        }
    }
}
