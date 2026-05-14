use std::collections::HashMap;


use crate::types::Player;
use crate::game::Action;
use crate::game::GameState;

pub struct Bot {
    troll_ids: Vec<i32>,
    final_actions: Vec<Action>,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            troll_ids: Vec::new(),
            final_actions: Vec::new(),
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
            .map(|a| a.to_string())
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

    pub fn eval(&mut self, game_state: &GameState) {
        let my_actions = self.find_all_actions(game_state, Player::Me);
        let opp_actions = self.find_all_actions(game_state, Player::Opp);

        // For now: pick first action per troll (drop > harvest > move > wait)
        self.final_actions.clear();
        for (_, actions) in &my_actions {
            if let Some(action) = actions.first() {
                self.final_actions.push(action.clone());
            }
        }

        eprintln!("My actions: {:?}", my_actions);
        eprintln!("Opp actions: {:?}", opp_actions);
    }

    // ------------------------------------------------------------------------
    // ------ Strategy
    // ------------------------------------------------------------------------
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
                (troll.id, actions)
            })
            .collect()
    }
}
