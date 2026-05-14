use std::collections::HashMap;

use crate::game::Action;
use crate::game::GameState;
use crate::position::Position;
use crate::types::{Player, Resource, Troll};

// ------------------------------------------------------------------------
// Prediction snapshot — lightweight subset of GameState for comparison
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct PredictedTroll {
    id: i32,
    position: Position,
    carry_plum: i32,
    carry_lemon: i32,
    carry_apple: i32,
    carry_banana: i32,
}

impl PredictedTroll {
    fn from_troll(t: &Troll) -> Self {
        Self {
            id: t.id,
            position: t.position,
            carry_plum: t.carry_plum,
            carry_lemon: t.carry_lemon,
            carry_apple: t.carry_apple,
            carry_banana: t.carry_banana,
        }
    }

    fn diff(&self, actual: &Troll) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.position != actual.position {
            diffs.push(format!(
                "position: predicted {:?} got {:?}",
                self.position, actual.position
            ));
        }
        if self.carry_plum != actual.carry_plum {
            diffs.push(format!(
                "carry_plum: predicted {} got {}",
                self.carry_plum, actual.carry_plum
            ));
        }
        if self.carry_lemon != actual.carry_lemon {
            diffs.push(format!(
                "carry_lemon: predicted {} got {}",
                self.carry_lemon, actual.carry_lemon
            ));
        }
        if self.carry_apple != actual.carry_apple {
            diffs.push(format!(
                "carry_apple: predicted {} got {}",
                self.carry_apple, actual.carry_apple
            ));
        }
        if self.carry_banana != actual.carry_banana {
            diffs.push(format!(
                "carry_banana: predicted {} got {}",
                self.carry_banana, actual.carry_banana
            ));
        }
        diffs
    }
}

#[derive(Debug, Clone)]
struct PredictedResources {
    plum: i32,
    lemon: i32,
    apple: i32,
    banana: i32,
}

impl PredictedResources {
    fn from_resources(r: &crate::types::Resources) -> Self {
        Self {
            plum: r.plum.amount(),
            lemon: r.lemon.amount(),
            apple: r.apple.amount(),
            banana: r.banana.amount(),
        }
    }

    fn diff(&self, r: &crate::types::Resources) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.plum != r.plum.amount() {
            diffs.push(format!(
                "plum: predicted {} got {}",
                self.plum,
                r.plum.amount()
            ));
        }
        if self.lemon != r.lemon.amount() {
            diffs.push(format!(
                "lemon: predicted {} got {}",
                self.lemon,
                r.lemon.amount()
            ));
        }
        if self.apple != r.apple.amount() {
            diffs.push(format!(
                "apple: predicted {} got {}",
                self.apple,
                r.apple.amount()
            ));
        }
        if self.banana != r.banana.amount() {
            diffs.push(format!(
                "banana: predicted {} got {}",
                self.banana,
                r.banana.amount()
            ));
        }
        diffs
    }
}

#[derive(Debug, Clone)]
struct PredictedTree {
    position: Position,
    fruits: i32,
    size: i32,
    cooldown: i32,
}

impl PredictedTree {
    fn from_tree(t: &crate::types::Tree) -> Self {
        Self {
            position: t.position,
            fruits: t.fruits,
            size: t.size,
            cooldown: t.cooldown,
        }
    }

    fn diff(&self, actual: &crate::types::Tree) -> Vec<String> {
        let mut diffs = Vec::new();
        if self.fruits != actual.fruits {
            diffs.push(format!(
                "fruits: predicted {} got {}",
                self.fruits, actual.fruits
            ));
        }
        if self.size != actual.size {
            diffs.push(format!(
                "size: predicted {} got {}",
                self.size, actual.size
            ));
        }
        if self.cooldown != actual.cooldown {
            diffs.push(format!(
                "cooldown: predicted {} got {}",
                self.cooldown, actual.cooldown
            ));
        }
        diffs
    }
}

#[derive(Debug, Clone)]
struct PredictedState {
    my_resources: PredictedResources,
    trolls: Vec<PredictedTroll>,
    trees: Vec<PredictedTree>,
}

// ------------------------------------------------------------------------
// Bot
// ------------------------------------------------------------------------

pub struct Bot {
    troll_ids: Vec<i32>,
    final_actions: Vec<Action>,
    predicted: Option<PredictedState>,
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

    pub fn eval(&mut self, game_state: &GameState) {
        let my_actions = self.find_all_actions(game_state, Player::Me);
        let _opp_actions = self.find_all_actions(game_state, Player::Opp);

        // For now: pick first action per troll (drop > harvest > move > wait)
        self.final_actions.clear();
        for actions in my_actions.values() {
            if let Some(action) = actions.first() {
                self.final_actions.push(action.clone());
            }
        }

        eprintln!("Actions: {:?}", self.final_actions);
    }

    // ------------------------------------------------------------------------
    // Simulation — predict next state from my actions
    // ------------------------------------------------------------------------

    pub fn simulate(&mut self, game_state: &GameState) {
        let mut sim = game_state.clone();
        sim.apply_actions(&self.final_actions);

        let my_trolls = sim
            .trolls
            .iter()
            .filter(|t| t.player == Player::Me)
            .map(PredictedTroll::from_troll)
            .collect();

        let trees = sim.trees.iter().map(PredictedTree::from_tree).collect();

        self.predicted = Some(PredictedState {
            my_resources: PredictedResources::from_resources(&sim.my_resources),
            trolls: my_trolls,
            trees,
        });
    }

    // ------------------------------------------------------------------------
    // Comparison — check prediction against actual state after update
    // ------------------------------------------------------------------------

    pub fn compare(&self, game_state: &GameState) {
        let Some(predicted) = &self.predicted else {
            return; // first turn, nothing to compare
        };

        let mut ok = true;

        // Compare resources
        let res_diffs = predicted.my_resources.diff(&game_state.my_resources);
        for diff in &res_diffs {
            ok = false;
            eprintln!("[SIM MISMATCH] resources: {diff}");
        }

        // Compare my trolls
        for pred_troll in &predicted.trolls {
            match game_state.trolls.iter().find(|t| t.id == pred_troll.id) {
                Some(actual) => {
                    for diff in pred_troll.diff(actual) {
                        ok = false;
                        eprintln!("[SIM MISMATCH] troll {}: {diff}", pred_troll.id);
                    }
                }
                None => {
                    ok = false;
                    eprintln!("[SIM MISMATCH] troll {} missing in actual state", pred_troll.id);
                }
            }
        }

        // Compare trees
        for pred_tree in &predicted.trees {
            match game_state
                .trees
                .iter()
                .find(|t| t.position == pred_tree.position)
            {
                Some(actual) => {
                    for diff in pred_tree.diff(actual) {
                        ok = false;
                        eprintln!(
                            "[SIM MISMATCH] tree@({},{}): {diff}",
                            pred_tree.position.x, pred_tree.position.y
                        );
                    }
                }
                None => {
                    ok = false;
                    eprintln!(
                        "[SIM MISMATCH] tree@({},{}) missing in actual state",
                        pred_tree.position.x, pred_tree.position.y
                    );
                }
            }
        }

        if ok {
            eprintln!("[SIM] prediction matched!");
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
