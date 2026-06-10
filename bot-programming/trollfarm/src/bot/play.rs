use crate::bot::Bot;
use crate::game::{Action, Game, Side};
use crate::utils::{CARDINALS, Position, bfs_distance_map, reconstruct_path};

use std::collections::HashSet;

impl Bot {
    pub fn play(&mut self, game: &mut Game) {
        self.reset_turn(game);
        Self::debug(game);

        // Early game
        if game.troll_count(Side::Me) == 1 {
            self.second_troll(game);
        }

        // Conditional 3rd troll: when a bigger workforce is outscaling a lead
        // we hold, convert part of the lead into a worker. Strictly additive —
        // off-latch behavior is unchanged.
        self.update_third_troll_mission(game);
        if self.third_troll_mission
            && let Some((ms, cc, hp, cp)) = Bot::third_troll_plan(game)
            && game.can_train(Side::Me, ms, cc, hp, cp)
        {
            eprintln!("[3RD] training {ms}/{cc}/{hp}/{cp} at turn={}", game.turn);
            self.actions.push(Action::Train(ms, cc, hp, cp));
        }

        // Late game
        self.late_game(game);

        // Finalize actions
        self.resolve_movement(game);

        // Feed next turn's stuck detector (see `reset_turn`).
        self.remember_turn(game);
    }

    /// Resolve each `Move` action into a single step, avoiding collisions.
    /// NB: opponent trolls are deliberately NOT in the blocked set — treating
    /// their (moving) bodies as walls re-routed paths so badly it cost ~40
    /// margin against every reference; pushing into their cells is kept on
    /// purpose — moves resolve simultaneously, so a push succeeds the moment
    /// they leave. Gridlock against OUR OWN bodies is prevented earlier, in
    /// `reset_turn`'s ally-blocked per-troll dist maps.
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

        // Add stationary trolls with actions (Drop included — a troll banking
        // at a shack-adjacent cell blocks it just like any other in-place
        // action; omitting it cost a failed move per shared banking turn).
        for action in &self.actions {
            match action {
                Action::Chop(id)
                | Action::Harvest(id)
                | Action::Mine(id)
                | Action::Pick(id, _)
                | Action::Drop(id)
                | Action::Plant(id, _) => {
                    if let Some(troll) = game.trolls.iter().find(|&t| t.id == *id) {
                        blocked.insert(troll.position);
                    }
                }
                _ => {}
            }
        }

        let stuck = self.stuck_ids(game);
        let my_bodies: Vec<(i32, Position)> = game
            .trolls
            .iter()
            .filter(|t| t.side == Side::Me)
            .map(|t| (t.id, t.position))
            .collect();
        let all_bodies: Vec<(i32, Position)> =
            game.trolls.iter().map(|t| (t.id, t.position)).collect();

        for action in &mut self.actions {
            let Action::Move(id, target) = action else {
                continue;
            };

            let Some(troll) = game.trolls.iter().find(|t| t.id == *id) else {
                continue;
            };

            // A stuck troll's step is planned with MY other trolls' bodies as
            // walls (matching its re-planned candidates from `reset_turn`),
            // so an existing detour around a blocking ally is actually taken
            // — the candidate map alone didn't help when the winning action
            // (e.g. banking) scores off the global shack map. Falls back to
            // the plain map when the walls leave no route.
            let mut local_blocked = blocked.clone();
            if stuck.contains(id) {
                local_blocked.extend(
                    my_bodies
                        .iter()
                        .filter(|(tid, _)| tid != id)
                        .map(|(_, p)| *p),
                );
            }
            let dist_map = bfs_distance_map(troll.position, &game.grid, &local_blocked);
            if *target == game.shack(Side::Me) {
                let shack = game.shack(Side::Me);
                let dist = |p: &Position| dist_map.get(p).map_or(i32::MAX, |(d, _)| *d);
                // Prefer an UNOCCUPIED banking cell: both trolls funneling to
                // the same nearest shack-adjacent cell blocked each other for
                // 36 failed moves in one arena loss. A body on the cell may
                // be gone next turn, so occupied cells stay eligible — just
                // ranked last.
                let occupied = |p: &Position| all_bodies.iter().any(|(tid, b)| tid != id && b == p);
                if let Some(adj) = CARDINALS
                    .iter()
                    .map(|c| shack + *c)
                    .filter(|p| game.grid.contains(*p) && b"~.ABLP".contains(&game.grid[*p]))
                    .min_by_key(|p| (occupied(p), dist(p)))
                {
                    *target = adj;
                }
            }

            let path = reconstruct_path(troll.position, *target, &dist_map)
                .filter(|p| !p.is_empty())
                .or_else(|| {
                    // walls left no route — plain map, old behavior
                    let dm = bfs_distance_map(troll.position, &game.grid, &blocked);
                    reconstruct_path(troll.position, *target, &dm).filter(|p| !p.is_empty())
                });

            if let Some(path) = path {
                let steps = usize::try_from(troll.movement_speed.max(0))
                    .unwrap_or(0)
                    .min(path.len());
                let next = path[steps.saturating_sub(1)];

                *action = Action::Move(*id, next);
                blocked.insert(next);
            }
        }
    }
}
