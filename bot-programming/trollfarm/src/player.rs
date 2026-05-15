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
        self.actions.clear();

        // Train a troll if we can afford one (1/1/1/0 is cheapest)
        if game.can_train(self.side, 1, 1, 1, 0) {
            self.actions.push(Action::Train(1, 1, 1, 0));
        }

        let all_actions = game.actions_for(self.side);

        for actions in all_actions.values() {
            // Priority: drop > harvest > plant > move > wait
            let chosen = actions.iter().find(|a| matches!(a, Action::Drop(_)))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::Harvest(_))))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::Plant(_, _))))
                .or_else(|| actions.first());

            if let Some(action) = chosen {
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
