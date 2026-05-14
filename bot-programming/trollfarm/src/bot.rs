use std::collections::HashMap;

use crate::game::{Action, GameState};
use crate::prediction::{Predictable, Snapshot};
use crate::types::Player;

pub struct Bot {
    troll_ids: Vec<i32>,
    final_actions: Vec<Action>,
    predicted: Option<Snapshot>,
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            troll_ids: Vec::new(),
            final_actions: Vec::new(),
            predicted: None,
        }
    }

    pub fn update(&mut self, game_state: &GameState) {
        self.troll_ids = game_state
            .trolls
            .iter()
            .filter(|t| t.player == Player::Me)
            .map(|t| t.id)
            .collect();
    }

    pub fn play(&mut self) {
        let output = self
            .final_actions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(";");

        println!(
            "{}",
            if output.is_empty() {
                "WAIT".into()
            } else {
                output
            }
        );
    }

    // ------------------------------------------------------------------------
    // Evaluation — pick actions
    // ------------------------------------------------------------------------

    pub fn think(&mut self, game_state: &GameState) {
        let my_actions = self.find_all_actions(game_state, Player::Me);
        let _opp_actions = self.find_all_actions(game_state, Player::Opp);

        // For now: pick first action per troll (drop > harvest > move > wait)
        self.final_actions.clear();
        for actions in my_actions.values() {
            if let Some(action) = actions.first() {
                self.final_actions.push(action.clone());
            }
        }
    }

    // ------------------------------------------------------------------------
    // Simulation
    // ------------------------------------------------------------------------

    pub fn simulate(&mut self, game_state: &GameState) {
        self.predicted = Some(game_state.snapshot(&self.final_actions));
    }

    pub fn compare(&self, game_state: &GameState) {
        if let Some(snapshot) = &self.predicted {
            game_state.compare(snapshot);
        }
    }

    // ------------------------------------------------------------------------
    // Strategy — enumerate possible actions per troll
    // ------------------------------------------------------------------------

    #[allow(clippy::unused_self)]
    fn find_all_actions(
        &self,
        game_state: &GameState,
        player: Player,
    ) -> HashMap<i32, Vec<Action>> {
        game_state
            .trolls
            .iter()
            .filter(|t| t.player == player)
            .map(|troll| {
                let mut actions = Vec::new();
                if troll.would_drop(game_state).is_some() {
                    actions.push(Action::Drop(troll.id));
                }
                if troll.would_harvest(game_state).is_some() {
                    actions.push(Action::Harvest(troll.id));
                }
                if let Some(moves) = troll.reachable_positions(game_state) {
                    for pos in moves {
                        actions.push(Action::Move(troll.id, pos));
                    }
                }
                actions.push(Action::Wait(troll.id));
                (troll.id, actions)
            })
            .collect()
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}
