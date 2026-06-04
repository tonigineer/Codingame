use crate::bot::Bot;
use crate::game::{Action, Game, Side};
use crate::utils::{CARDINALS, Position, bfs_distance_map, reconstruct_path};

use std::collections::HashSet;

impl Bot {
    pub fn play(&mut self, game: &mut Game) {
        self.reset_turn(game);

        // Early game
        if game.troll_count(Side::Me) == 1 {
            self.second_troll(game);
        }

        // Mid game
        self.mid_game(game);

        // Late game
        self.resolve_movement(game);
    }

    /// Resolve each `Move` action into a single step, avoiding collisions.
    fn resolve_movement(&mut self, game: &Game) {
        let mut blocked: HashSet<Position> = HashSet::new();

        // Trolls without actions will be stationary and block positions
        if i32::try_from(self.actions.len()).unwrap() < game.troll_count(Side::Me) {
            for troll in &game.trolls(Side::Me) {
                let has_action = self.actions.iter().any(|a| a.troll_id() == Some(troll.id));
                if !has_action {
                    eprintln!("Troll: {} has not action", troll.id);
                    blocked.insert(troll.position);
                }
            }
        }

        // Add stationary trolls with actions
        for action in &self.actions {
            match action {
                Action::Chop(id)
                | Action::Harvest(id)
                | Action::Mine(id)
                | Action::Pick(id, _)
                | Action::Plant(id, _) => {
                    if let Some(troll) = game.trolls.iter().find(|&t| t.id == *id) {
                        blocked.insert(troll.position);
                    }
                }
                _ => {}
            }
        }

        for action in &mut self.actions {
            let Action::Move(id, target) = action else {
                continue;
            };

            let Some(troll) = game.trolls.iter().find(|t| t.id == *id) else {
                continue;
            };

            let dist_map = bfs_distance_map(troll.position, &game.grid, &blocked);
            if *target == game.shack(Side::Me) {
                let shack = game.shack(Side::Me);
                let dist = |p: &Position| dist_map.get(p).map_or(i32::MAX, |(d, _)| *d);
                if let Some(adj) = CARDINALS
                    .iter()
                    .map(|c| shack + *c)
                    .filter(|p| game.grid.contains(*p) && b"~.ABLP".contains(&game.grid[*p]))
                    .min_by_key(|p| dist(p))
                {
                    *target = adj;
                }
            }

            if let Some(path) = reconstruct_path(troll.position, *target, &dist_map) {
                let steps = usize::try_from(troll.movement_speed.max(0))
                    .unwrap_or(0)
                    .min(path.len());
                let next = path[steps - 1];

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
        game.opp_shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Opp), &game.grid, &blocked);

        for troll in &mut game.trolls {
            troll.dist_map = crate::utils::bfs_distance_map(troll.position, &game.grid, &blocked);
        }

        // Refresh the per-turn tree-proximity comparison now that both shack
        // distance maps are current.
        game.update_tree_proximity();
    }
}
