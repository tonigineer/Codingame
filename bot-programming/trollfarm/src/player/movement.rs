use crate::entities::Troll;
use crate::game::{Action, Game};
use crate::position::Position;
use crate::utils::*;

use std::collections::HashSet;

use super::{Plan, Player};

impl Player {
    /// Assigns movement intents for all non-busy trolls: either continues an
    /// existing plan or finds a new target via `find_best_target`.
    ///
    /// Returns a list of `(troll_id, from, path_tiles, speed)` tuples.
    pub fn assign_moves(
        &mut self,
        game: &Game,
        trolls: &[&Troll],
        shack: &Position,
        latest_plans: &[Plan],
    ) -> Vec<(i32, Position, Vec<Position>, i32)> {
        let no_blocked = HashSet::new();
        let shack_dist_map = bfs_distance_map(*shack, &game.grid, &no_blocked);

        let mut move_intents: Vec<(i32, Position, Vec<Position>, i32)> = Vec::new();

        let currently_busy = self.trolls_busy.clone();
        for troll in trolls.iter().filter(|t| !currently_busy.contains(&t.id)) {
            let troll_dist_map = bfs_distance_map(troll.position, &game.grid, &no_blocked);

            // --- Continue with existing plan
            if let Some(plan) = latest_plans
                .iter()
                .find(|p| p.troll_id == troll.id && !self.trolls_busy.contains(&troll.id))
            {
                let destination = plan.to;

                let mut moved = false;
                if let Some(path) = reconstruct_path(troll.position, destination, &troll_dist_map) {
                    if !path.is_empty() {
                        let steps = path.len().min(troll.movement_speed as usize);
                        let path_slice: Vec<Position> = path[..steps].to_vec();
                        move_intents.push((
                            troll.id,
                            troll.position,
                            path_slice,
                            troll.movement_speed,
                        ));
                        moved = true;
                    }
                }

                if moved {
                    self.plans.push(*plan);
                    self.trolls_busy.insert(troll.id);
                    continue;
                }
                // Plan's destination unreachable — remove from claimed
                self.claimed_entities.remove(&plan.to);
            }

            // --- New plan: find best target based on priority
            if let Some((action, destination)) = self.find_best_target(
                &game,
                &troll,
                &troll_dist_map,
                &shack_dist_map,
                shack,
            ) {
                if let Some(path) = reconstruct_path(troll.position, destination, &troll_dist_map) {
                    if !path.is_empty() {
                        let steps = path.len().min(troll.movement_speed as usize);
                        let path_slice: Vec<Position> = path[..steps].to_vec();
                        move_intents.push((
                            troll.id,
                            troll.position,
                            path_slice,
                            troll.movement_speed,
                        ));
                    }
                }

                self.claimed_entities.insert(destination);
                self.plans.push(Plan {
                    troll_id: troll.id,
                    to: destination,
                    action,
                });
                self.trolls_busy.insert(troll.id);
            }
        }

        move_intents
    }

    /// Resolves movement collisions (swaps, blocked paths, fallback routing)
    /// and pushes the final MOVE actions.
    pub fn resolve_collisions(
        &mut self,
        game: &Game,
        move_intents: &[(i32, Position, Vec<Position>, i32)],
    ) {
        let mut final_claimed: HashSet<Position> = self.positions_claimed.clone();

        // Detect swaps (A→B and B→A) — allow both
        let mut swap_ids: HashSet<i32> = HashSet::new();
        for i in move_intents {
            for j in move_intents {
                let i_to = i.2.last().copied().unwrap_or(i.1);
                let j_to = j.2.last().copied().unwrap_or(j.1);
                if i.0 != j.0 && i_to == j.1 && j_to == i.1 {
                    swap_ids.insert(i.0);
                    swap_ids.insert(j.0);
                }
            }
        }

        // Process swaps first
        for (id, _from, path, _speed) in move_intents {
            if swap_ids.contains(id) {
                if let Some(&to) = path.last() {
                    self.actions.push(Action::Move(*id, to));
                    final_claimed.insert(to);
                }
            }
        }

        // Process non-swap moves: check ALL tiles in the path
        for (id, from, path, speed) in move_intents {
            if swap_ids.contains(id) {
                continue;
            }

            let all_clear = path.iter().all(|p| !final_claimed.contains(p));

            if all_clear && !path.is_empty() {
                let to = *path.last().unwrap();
                self.actions.push(Action::Move(*id, to));
                final_claimed.insert(to);
            } else {
                let plan_dest = self
                    .plans
                    .iter()
                    .find(|pl| pl.troll_id == *id)
                    .map(|pl| pl.to)
                    .unwrap_or(*from);

                let alt_dist_map = bfs_distance_map(*from, &game.grid, &final_claimed);

                let mut found = false;
                if let Some(alt_path) = reconstruct_path(*from, plan_dest, &alt_dist_map) {
                    if !alt_path.is_empty() {
                        let steps = alt_path.len().min(*speed as usize);
                        if alt_path[..steps]
                            .iter()
                            .all(|p| !final_claimed.contains(p))
                        {
                            let alt_to = alt_path[steps - 1];
                            self.actions.push(Action::Move(*id, alt_to));
                            final_claimed.insert(alt_to);
                            found = true;
                        }
                    }
                }

                if !found {
                    let mut candidates: Vec<(Position, i32)> = alt_dist_map
                        .iter()
                        .filter(|(p, (d, _))| {
                            *d >= 1 && *d <= *speed as i32 && !final_claimed.contains(p)
                        })
                        .map(|(&p, &(_, _))| (p, p.manhattan(&plan_dest) as i32))
                        .collect();

                    candidates.sort_by_key(|(_, dist_to_goal)| *dist_to_goal);

                    if let Some(&(alt_pos, _)) = candidates.first() {
                        self.actions.push(Action::Move(*id, alt_pos));
                        final_claimed.insert(alt_pos);
                    } else {
                        eprintln!("Troll {id} is completely stuck.");
                    }
                }
            }
        }
    }
}
