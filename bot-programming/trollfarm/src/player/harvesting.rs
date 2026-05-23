use crate::game::{Action, Game};
use crate::entities::Troll;
use crate::player::Plan;
use super::Player;
use crate::utils::*;
use crate::position::{Position, CARDINALS};

use std::collections::HashMap;
/// Minimum stats for trolls 2, 3, 4: [movement_speed, carry_capacity, harvest_power, chop_power]
/// The 1st troll (starter, carry_capacity=1) is not listed here — it already exists.
// #[rustfmt::skip]
// const TROLL_BUILDS: &[[i32; 4]] = &[
//     // 2nd troll: balanced harvester
//     [1, 1, 1, 1],
//     // 3rd troll: stronger all-rounder
//     [4, 5, 4, 5],
//     // 4th troll: dedicated chopper (no harvest power needed)
//     [4, 5, 1, 5],
// ];

// const MAX_STAT: i32 = 5;
//

impl Player {
    pub fn harvesting(&self, game: &Game, troll: &Troll) -> Option<Plan> {
        eprintln!("[harvesting {}]", troll.id);
        let troll_dist_map = bfs_distance_map(troll.position, &game.grid, &self.claimed_positions);

        // Generate again due newly claimed positions
        let shack = game.shack(self.side);
        let shack_dist_map = bfs_distance_map(shack, &game.grid, &self.claimed_positions);

        if troll.has_cargo() && troll.free_capacity() == 0 {
                if let Some((action, to)) =
                    self.find_deliver_target(game, troll, &troll_dist_map, &shack)
                {
                    return Some(Plan {
                        troll_id: troll.id,
                        to: to,
                        action: action,
                    });
                }
        }

        let mut best: Option<(Action, Position, i32)> = None;

        // --- Score each tree with fruit
        for tree in game.trees.iter() {
            if self.claimed_resources.contains(&tree.position) {
                continue;
            }

            let weight = self.priority.need_for_resource(tree.get_resource_type());
            if weight == 0 {
                continue;
            }

            let tile_dist = match troll_dist_map.get(&tree.position) {
                Some((d, _)) => *d,
                None => continue,
            };

            let shack_return = shack_dist_map
                .get(&tree.position)
                .map(|(d, _)| *d)
                .unwrap_or(9999);

            let round_trip = tile_dist + shack_return;
            if round_trip >= game.turns_remaining() {
                continue;
            }

            #[rustfmt::skip]
            let fruit_at_arrival = |tree: &crate::entities::Tree, tile_dist: i32| -> i32 {
                let mut fruits = tree.fruits;
                let remaining = tile_dist - tree.cooldown;
                if remaining >= 0 {
                    fruits += 1;
                    fruits += remaining / if game.is_near_water(*&tree.position) {
                            tree.cooldown_time_water()
                        } else {
                            tree.cooldown_time()
                        }
                }
                fruits.min(3)
            };

            let fruits = fruit_at_arrival(tree, tile_dist);
            if fruits == 0 {
                continue;
            }

            let harvestable = fruits.min(troll.free_capacity());
            let score = harvestable + weight * 2 - tile_dist;

            if best.is_none() || score > best.unwrap().2 {
                eprintln!("found action at {:?}", tree.position);
                best = Some((Action::Harvest(troll.id), tree.position, score));
            }
        }

        // // --- Score iron mining spots
        // if troll.chop_power > 0 && self.priority.iron > 0 {
        //     for &mine in game.mines.iter() {
        //         for &c in CARDINALS.iter() {
        //             let adj = mine + c;
        //             if !game.grid.contains(adj) || !b".ABPL".contains(&game.grid[adj]) {
        //                 continue;
        //             }
        //             if self.claimed_entities.contains(&adj) {
        //                 continue;
        //             }
        //             let tile_dist = match troll_dist_map.get(&adj) {
        //                 Some((d, _)) => *d,
        //                 None => continue,
        //             };
        //             let shack_return = shack_dist_map.get(&adj).map(|(d, _)| *d).unwrap_or(9999);
        //             let round_trip = tile_dist + shack_return;
        //             if round_trip >= game.turns_remaining() {
        //                 continue;
        //             }
        //             let score = troll.free_capacity() + self.priority.iron * 2 - tile_dist;
        //             if best.is_none() || score > best.unwrap().2 {
        //                 best = Some((Action::Mine(troll.id), adj, score));
        //             }
        //         }
        //     }
        // }

        // // --- Score each tree for chopping
        // if troll.chop_power > 0 && self.priority.wood > 0 {
        //     for tree in game.trees.iter() {
        //         if self.claimed_entities.contains(&tree.position) {
        //             continue;
        //         }
        //         let tile_dist = match troll_dist_map.get(&tree.position) {
        //             Some((d, _)) => *d,
        //             None => continue,
        //         };
        //         let chop_turns = (tree.health + troll.chop_power - 1) / troll.chop_power;
        //         let shack_return = shack_dist_map
        //             .get(&tree.position)
        //             .map(|(d, _)| *d)
        //             .unwrap_or(9999);
        //         let round_trip = tile_dist + chop_turns + shack_return;
        //         if tree.size < 2 && round_trip >= game.turns_remaining() {
        //             continue;
        //         }
        //         let wood_yield = tree.size;
        //         let carriable = wood_yield.min(troll.free_capacity());
        //         let score = carriable + self.priority.wood * 2 - tile_dist - chop_turns;
        //         if best.is_none() || score > best.unwrap().2 {
        //             best = Some((Action::Chop(troll.id), tree.position, score));
        //         }
        //     }
        // }

        if best.is_none() && troll.has_cargo() {
            if let Some((action, to)) =
                self.find_deliver_target(game, troll, &troll_dist_map, &shack)
            {
                eprintln!("BRING BACK");
                return Some(Plan {
                    troll_id: troll.id,
                    to: to,
                    action: action,
                });
            }
        }

        best.map(|(action, pos, _)| (action, pos));

        let to = troll.position;
        let action = Action::Harvest(troll.id);
        let troll_id = troll.id;

        Some(Plan {
            to,
            action,
            troll_id,
        })
    }

    pub fn find_deliver_target(
        &self,
        game: &Game,
        troll: &Troll,
        troll_dist_map: &HashMap<Position, (i32, Position)>,
        shack: &Position,
    ) -> Option<(Action, Position)> {
        let best_adj = CARDINALS
            .iter()
            .map(|&c| *shack + c)
            .filter(|&p| game.grid.contains(p) && b".ABPL".contains(&game.grid[p]))
            .filter_map(|p| troll_dist_map.get(&p).map(|(d, _)| (p, *d)))
            .min_by_key(|(_, d)| *d);

        best_adj.map(|(adj_pos, _)| (Action::Drop(troll.id), adj_pos))
    }

    // pub fn training(&mut self, game: &Game, side: Side) -> Option<Action> {
    //     let trolls = game.trolls_for(side);
    //     let num_trolls = trolls.len();

    //     // 1 starter + TROLL_BUILDS.len() additional trolls
    //     if num_trolls > TROLL_BUILDS.len() {
    //         return None;
    //     }

    //     // Shack blocked by a troll — can't spawn
    //     if trolls.iter().any(|t| t.position == game.shack(side)) {
    //         return None;
    //     }

    //     // num_trolls == 1 → index 0 (2nd troll build)
    //     // num_trolls == 2 → index 1 (3rd troll build)
    //     // etc.
    //     let mins = TROLL_BUILDS[num_trolls - 1];

    //     let mut best: Option<(Action, i32)> = None;

    //     for ms in mins[0]..=MAX_STAT {
    //         for cc in mins[1]..=MAX_STAT {
    //             for hp in mins[2]..=MAX_STAT {
    //                 for cp in mins[3]..=MAX_STAT {
    //                     if !game.can_train(side, ms, cc, hp, cp) {
    //                         continue;
    //                     }
    //                     let score = ms + cc + hp + cp;
    //                     if best.is_none() || score > best.as_ref().unwrap().1 {
    //                         best = Some((Action::Train(ms, cc, hp, cp), score));
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     if let Some((ref action, _)) = best {
    //         eprintln!(
    //             "[TRAINING] Troll #{} with {:?} (mins {:?})",
    //             num_trolls + 1,
    //             action,
    //             mins
    //         );
    //     }

    //     best.map(|(action, _)| action)
    // }
}
