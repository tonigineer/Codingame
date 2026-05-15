use crate::game::Game;
use crate::prediction::{Predictable, Snapshot};
use crate::game::{Action, Side};

pub struct Player {
    pub side: Side,
    pub actions: Vec<Action>,
    predicted: Option<Snapshot>,
}

impl Player {
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            side,
            actions: Vec::new(),
            predicted: None,
        }
    }

    // --------------------------------------------------------------------
    // Think — decide what to do
    // --------------------------------------------------------------------

    pub fn think(&mut self, game: &Game) {
        let all_actions = game.actions_for(self.side);

        // For now: pick first action per troll (drop > harvest > move > wait)
        self.actions.clear();
        for actions in all_actions.values() {
            if let Some(action) = actions.first() {
                self.actions.push(action.clone());
            }
        }
    }

    // --------------------------------------------------------------------
    // Simulation
    // --------------------------------------------------------------------

    pub fn simulate(&mut self, game: &Game) {
        // For now we pass empty opponent actions — we don't know them
        self.predicted = Some(game.snapshot(&self.actions, &[]));
    }

    pub fn compare(&self, game: &Game) {
        if let Some(snapshot) = &self.predicted {
            game.compare(snapshot);
        }
    }
}
