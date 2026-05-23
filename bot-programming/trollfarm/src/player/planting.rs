use crate::entities::TreeType;
use crate::game::{Action, Game};
use crate::player::Plan;
use crate::position::Position;
use crate::utils::*;

use std::collections::{HashMap, HashSet};

use super::Player;

/// Fruit types the first troll should plant, balanced.
const FRUIT_TYPES: [TreeType; 3] = [TreeType::Plum, TreeType::Apple, TreeType::Lemon];

impl Player {
    pub fn planting(&mut self, game: &Game) -> Option<Plan> {
        let shack = game.shack(self.side);
        let trolls = game.trolls_for(self.side);
        let inv = game.inventory(self.side);

        // Only the first troll (carry_capacity == 1) does planting
        let troll = match trolls.iter().find(|t| t.carry_capacity == 1) {
            Some(t) => *t,
            None => return None,
        };

        let no_blocked = HashSet::new();
        let shack_dist_map = bfs_distance_map(shack, &game.grid, &no_blocked);

        let spots = self.find_early_plant_spots(game, &shack_dist_map);
        if spots.is_empty() {
            return None;
        }

        let adjacent_to_shack = troll.position.manhattan(&shack) == 1;

        // --- Priority 1: Already carrying any fruit seed → plant it
        let carried_type = FRUIT_TYPES
            .iter()
            .find(|t| troll.carries_resource(t.as_resource_type()) > 0)
            .copied();

        eprintln!("{:?}", carried_type);

        if let Some(typ) = carried_type {
            let best_spot = self.best_spot_for_troll(troll.position, &spots);
            eprintln!(
                "[PLANTING] troll {} will plant {:?} at {:?}",
                troll.id, typ, best_spot
            );
            return Some(Plan {
                troll_id: troll.id,
                to: best_spot,
                action: Action::Plant(troll.id, typ),
            });
        }

        // // --- Priority 1b: A Pick plan is about to execute this turn
        // //     (troll is at plan destination and plan action is Pick).
        // //     Look ahead: after picking, the troll will have a seed,
        // //     so pre-create the Plant plan for the best spot.
        // if let Some(plan) = self.plans.iter().find(|p| p.troll_id == troll.id) {
        //     if let Action::Pick(_, pick_type) = plan.action {
        //         if plan.to == troll.position {
        //             let best_spot = self.best_spot_for_troll(troll.position, &spots);
        //             eprintln!(
        //                 "[PLANTING] troll {} pick {:?} executing, pre-planning plant at {:?}",
        //                 troll.id, pick_type, best_spot
        //             );
        //             return Some(Plan {
        //                 troll_id: troll.id,
        //                 to: best_spot,
        //                 action: Action::Plant(troll.id, pick_type),
        //             });
        //         }
        //     }
        // }

        // // Pick the tree type we have the fewest of nearby (balanced planting)
        let tree_type = self.least_abundant_fruit(game, &shack_dist_map)?;
        let seed_in_inv = inv.get_by_tree(&tree_type) > 0;

        // // --- Priority 2: Adjacent to shack, no cargo, seed available → pick up now
        // if adjacent_to_shack && !troll.has_cargo() && seed_in_inv {
        //     eprintln!(
        //         "[PLANTING] troll {} picking {:?}",
        //         troll.id, tree_type
        //     );
        //     return Some(Plan {
        //         troll_id: troll.id,
        //         to: troll.position,
        //         action: Action::Pick(troll.id, tree_type),
        //     });
        // }

        // --- Priority 3: Move toward shack / plant spot
        if !troll.has_cargo() && seed_in_inv {
            let best_spot = self.best_spot_for_troll(troll.position, &spots);

            let spot_dist = shack_dist_map
                .get(&best_spot)
                .map(|(d, _)| *d)
                .unwrap_or(9999);

            if spot_dist == 1 {
                // Spot is shack-adjacent: move there, pick on arrival
                eprintln!(
                    "[PLANTING] troll {} moving to plant spot {:?} (shack-adjacent, will pick {:?})",
                    troll.id, best_spot, tree_type
                );
                return Some(Plan {
                    troll_id: troll.id,
                    to: best_spot,
                    action: Action::Pick(troll.id, tree_type),
                });
            }

            // Dist 2+: walk to shack-adjacent tile first
            let shack_adj = crate::position::CARDINALS
                .iter()
                .map(|&c| shack + c)
                .filter(|&p| game.grid.contains(p) && b".ABPL".contains(&game.grid[p]))
                .min_by_key(|p| {
                    let to_spot = p.manhattan(&best_spot) as i32;
                    let to_troll = troll.position.manhattan(p) as i32;
                    to_troll + to_spot
                });

            if let Some(adj) = shack_adj {
                eprintln!(
                    "[PLANTING] troll {} walking to shack via {:?} to pick {:?}",
                    troll.id, adj, tree_type
                );
                return Some(Plan {
                    troll_id: troll.id,
                    to: adj,
                    action: Action::Pick(troll.id, tree_type),
                });
            }
        }

        None
    }

    /// Find empty tiles near shack for planting:
    ///   - dist 1: adjacent to shack (pick seed, plant, repeat without travel)
    ///   - dist 2: one step away
    ///   - dist 3: only if near water (worth the extra walk for faster growth)
    fn find_early_plant_spots(
        &self,
        game: &Game,
        shack_dist_map: &HashMap<Position, (i32, Position)>,
    ) -> Vec<(Position, i32)> {
        let mut spots: Vec<(Position, i32)> = shack_dist_map
            .iter()
            .filter(|(pos, (dist, _))| {
                let empty = game.grid[**pos] == b'.' && game.tree_at(**pos).is_none();
                if !empty {
                    return false;
                }
                match *dist {
                    1 | 2 => true,
                    3 => game.is_near_water(**pos),
                    _ => false,
                }
            })
            .map(|(&pos, &(dist, _))| {
                let water_bonus = if game.is_near_water(pos) { 3 } else { 0 };
                (pos, water_bonus - dist)
            })
            .collect();

        spots.sort_by_key(|(_, score)| -score);
        spots
    }

    /// Pick the fruit type (Plum, Apple, Lemon) with the fewest trees near the shack.
    fn least_abundant_fruit(
        &self,
        game: &Game,
        shack_dist_map: &HashMap<Position, (i32, Position)>,
    ) -> Option<TreeType> {
        let mut counts: HashMap<TreeType, i32> = FRUIT_TYPES
            .iter()
            .map(|&t| (t, 0))
            .collect();

        for tree in &game.trees {
            if !FRUIT_TYPES.contains(&tree.typ) {
                continue;
            }
            let dist = shack_dist_map
                .get(&tree.position)
                .map(|(d, _)| *d)
                .unwrap_or(9999);
            if dist <= 4 {
                *counts.entry(tree.typ).or_default() += 1;
            }
        }

        counts
            .into_iter()
            .min_by_key(|(_, count)| *count)
            .map(|(typ, _)| typ)
    }

    /// Pick the best spot considering troll distance and spot score.
    fn best_spot_for_troll(&self, troll_pos: Position, spots: &[(Position, i32)]) -> Position {
        spots
            .iter()
            .min_by_key(|(pos, score)| (troll_pos.manhattan(pos) as i32 - score))
            .map(|(pos, _)| *pos)
            .unwrap()
    }
}
