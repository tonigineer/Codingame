use crate::types::{Troll, Player};
use crate::game::GameState;

pub struct Bot {
    troll_ids: Vec<i32>,
}

impl Bot {
    pub fn new() -> Self {
        Self { troll_ids: Vec::new() }
    }

    pub fn update(&mut self, game_state: &GameState) {
        self.troll_ids = game_state.trolls.iter()
            .filter(|t| t.player == Player::Me)
            .map(|t| t.id)
            .collect();
    }

    pub fn play(&self, game_state: &GameState) {
        eprintln!("Shack : {:?}", game_state.my_shack);

        let my_trolls: Vec<&Troll> = game_state.trolls.iter()
            .filter(|t| self.troll_ids.contains(&t.id))
            .collect();

        for troll in my_trolls {
            eprintln!("Troll : {:?}", troll);
            if let Some(resources) = troll.would_drop(game_state) {
                eprintln!("Drop : {:?}", resources);
            }
            if let Some(harvest) = troll.would_harvest(game_state) {
                eprintln!("Harvest : k{:?}", harvest);
            }
            if let Some(moves) = troll.reachable_positions(game_state) {
                eprintln!("Moves : {:?}", moves);
            }
        }
    }
}
