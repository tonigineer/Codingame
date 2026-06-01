use crate::bot::Bot;
use crate::game::{Action, Game, Side};
use crate::utils::{Position, bfs_distance_map, reconstruct_path};

use std::collections::{HashSet};

impl Bot {
    pub fn play(&mut self, game: &mut Game) {
        self.reset_turn(game);

        // Early game
        if game.troll_count(Side::Me) == 1 {
            self.second_troll(game);
        }

        // Mid game
        self.mid_game(game);

        // Solve movement — resolve each Move into a single step, avoiding collisions.
        let mut blocked: HashSet<Position> = HashSet::new();

        for action in self.actions.iter_mut() {
            let Action::Move(id, target) = action else {
                continue;
            };

            let Some(troll) = game.trolls.iter().find(|t| t.id == *id) else {
                continue;
            };
            let dist_map = bfs_distance_map(troll.position, &game.grid, &blocked);

            if let Some(path) = reconstruct_path(troll.position, *target, &dist_map) {
                // path[0] is the current position; step as far as movement_speed allows.
                let steps = (troll.movement_speed as usize).min(path.len() - 1);
                let next = path[steps];

                *action = Action::Move(*id, next);
                blocked.insert(next);
            }
        }
    }

    fn reset_turn(&mut self, game: &mut Game) {
        self.actions.clear();

        let blocked = HashSet::new();
        game.shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Me), &game.grid, &blocked);

        for troll in game.trolls.iter_mut() {
            troll.dist_map = crate::utils::bfs_distance_map(troll.position, &game.grid, &blocked);
        }
    }
}
