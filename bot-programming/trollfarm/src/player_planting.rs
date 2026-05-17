use crate::position::Position;
use crate::game::Game;
use crate::player::Player;
use crate::entities::TreeType;

use std::collections::HashMap;


pub trait Planting {
    // Return tiles where it makes sense to plant trees.

    fn planting(game: &Game, player: &Player) -> Option<HashMap<TreeType, Position>> {
        // let tiles_for_tree = find_best_tree_spots();
        // let trees_neededk = what_trees_are_needed();
        None
        // if game.turn == 1 {
            //     return None;
            // }

            // let shack = game.shack(self.side);
            // // let trolls = game.trolls_for(self.side);
            // // let inv = game.inventory(self.side);

            // // --- Find plantable spots near the shack (empty walkable tiles, no tree)
            // let no_blocked = HashSet::new();
            // let shack_dist_map = bfs_distance_map(shack, game, &no_blocked);

            // let mut plant_spots: Vec<(Position, i32)> = Vec::new();
            // for (&pos, &(dist, _)) in &shack_dist_map {
            //     if dist <= 1 || dist > 8 {
            //         continue; // not too far, not the shack itself
            //     }
            //     if claimed_targets.contains(&pos) {
            //         continue;
            //     }
            //     // Must be a plain walkable tile with no tree already on it
            //     if game.grid[pos] != b'.' {
            //         continue;
            //     }
            //     if game.tree_at(pos).is_some() {
            //         continue;
            //     }
            //     // Prefer spots closer to shack, and near water
            //     let water_bonus = if game.is_near_water(pos) { 3 } else { 0 };
            //     let score = water_bonus - dist;
            //     plant_spots.push((pos, score));
            // }

            // plant_spots.sort_by_key(|(_, score)| -score);
    }

    // fn assign_troll_to_plan_tree(game: &Game, player: &Player, position: &Position, tree_type: &TreeType) -> Option<Plan> {
    //     None
    // }
}

fn find_best_tree_spots() {}




// ========================================================================
// Planting
// ========================================================================

// fn planting(
//     &mut self,
//     game: &Game,
//     busy: &mut HashSet<i32>,
//     claimed: &mut HashSet<Position>,
//     claimed_targets: &mut HashSet<Position>,
// ) {
//     if game.turn == 1 {
//         return;
//     }
//     let shack = game.shack(self.side);
//     // let trolls = game.trolls_for(self.side);
//     // let inv = game.inventory(self.side);

//     // --- Find plantable spots near the shack (empty walkable tiles, no tree)
//     let no_blocked = HashSet::new();
//     let shack_dist_map = bfs_distance_map(shack, game, &no_blocked);

//     let mut plant_spots: Vec<(Position, i32)> = Vec::new();
//     for (&pos, &(dist, _)) in &shack_dist_map {
//         if dist <= 1 || dist > 8 {
//             continue; // not too far, not the shack itself
//         }
//         if claimed_targets.contains(&pos) {
//             continue;
//         }
//         // Must be a plain walkable tile with no tree already on it
//         if game.grid[pos] != b'.' {
//             continue;
//         }
//         if game.tree_at(pos).is_some() {
//             continue;
//         }
//         // Prefer spots closer to shack, and near water
//         let water_bonus = if game.is_near_water(pos) { 3 } else { 0 };
//         let score = water_bonus - dist;
//         plant_spots.push((pos, score));
//     }

//     plant_spots.sort_by_key(|(_, score)| -score);

//     if plant_spots.is_empty() {
//         return;
//     }
//     eprintln!("{:?}", plant_spots);

//     // --- Check nearby tree's
//     let mut tree_abundance: HashMap<TreeType, i32> = HashMap::from([
//         (TreeType::Apple, 0),
//         (TreeType::Plum, 0),
//         (TreeType::Lemon, 0),
//         (TreeType::Banana, 0),
//     ]);

//     for tree in game.trees.iter() {
//         let dist = shack_dist_map
//             .get(&tree.position)
//             .map(|(d, _)| *d)
//             .unwrap_or(9999);
//         if dist <= 8 {
//             *tree_abundance.entry(tree.typ).or_insert(0) += {
//                 if game.is_near_water(tree.position) {
//                     3
//                 } else {
//                     2
//                 }
//             };
//         }
//     }

// // We want a balanced mix; types with fewer nearby trees get priority
// let types = [
//     TreeType::Apple,
//     TreeType::Plum,
//     TreeType::Lemon,
//     TreeType::Banana,
// ];
// let mut type_priority: Vec<(TreeType, i32)> = types
//     .iter()
//     .map(|&typ| {
//         let nearby = nearby_counts.get(&typ).copied().unwrap_or(0);
//         // let need = self.priority.weight_for_type(typ);
//         // // Score: higher = more wanted. Prefer types we need and don't have nearby.
//         // let score = need * 2 - nearby * 3;
//         (typ, score)
//     })
//     .collect();

// tree_abundance.sort_by_key(|(_, score)| score);
// eprintln!("{:?}", tree_abundance);
// }
// // --- Find a troll that can plant
// // Candidates: idle or nearby trolls that aren't busy
// for &(spot, _) in plant_spots.iter().take(3) {
//     // Find best seed type for this spot
//     let best_type = type_priority
//         .iter()
//         .find(|(typ, score)| {
//             *score > 0
//                 && (inv.get(typ) > 0
//                     || trolls
//                         .iter()
//                         .any(|t| !busy.contains(&t.id) && t.carries(typ) > 0))
//         })
//         .map(|(typ, _)| *typ);

//     let seed_type = match best_type {
//         Some(t) => t,
//         None => continue, // no seeds available
//     };

//     // Find closest non-busy troll to this spot
//     let best_troll = trolls
//         .iter()
//         .filter(|t| !busy.contains(&t.id))
//         .filter(|t| t.free_capacity() > 0 || t.carries(&seed_type) > 0)
//         .min_by_key(|t| t.position.manhattan(&spot));

//     let troll = match best_troll {
//         Some(t) => t,
//         None => continue,
//     };

//     // Does the troll already carry this seed?
//     let has_seed = troll.carries(&seed_type) > 0;

//     if has_seed && troll.position == spot {
//         // Standing on spot with seed — plant!
//         self.actions.push(Action::Plant(troll.id, seed_type));
//         busy.insert(troll.id);
//         claimed.insert(troll.position);
//         claimed_targets.insert(spot);
//         continue;
//     }

//     if !has_seed && troll.position.manhattan(&shack) == 1 && inv.get(&seed_type) > 0 {
//         // Adjacent to shack without seed — pick it up
//         self.actions.push(Action::Pick(troll.id, seed_type));
//         busy.insert(troll.id);
//         claimed.insert(troll.position);
//         // Plan: next turn move to spot, then plant
//         self.plans.push(Plan {
//             troll_id: troll.id,
//             to: spot,
//             action: Action::Plant(troll.id, seed_type),
//         });
//         claimed_targets.insert(spot);
//         continue;
//     }

//     // Need to walk somewhere first
//     if has_seed {
//         // Has seed, walk to spot
//         self.plans.push(Plan {
//             troll_id: troll.id,
//             to: spot,
//             action: Action::Plant(troll.id, seed_type),
//         });
//     } else {
//         // Need to go to shack first to pick up seed, then to spot
//         // For now: plan to walk to shack-adjacent, pick will happen
//         // opportunistically when adjacent
//         let shack_adj = CARDINALS
//             .iter()
//             .map(|&c| shack + c)
//             .find(|&p| game.grid.contains(p) && b".ABPL".contains(&game.grid[p]));

//         if let Some(adj) = shack_adj {
//             self.plans.push(Plan {
//                 troll_id: troll.id,
//                 to: adj,
//                 action: Action::Pick(troll.id, seed_type),
//             });
//             // We'll need a second plan for planting after picking —
//             // that gets created when the Pick plan completes
//         }
//     }

//     busy.insert(troll.id);
//     claimed_targets.insert(spot);
// }
// }
