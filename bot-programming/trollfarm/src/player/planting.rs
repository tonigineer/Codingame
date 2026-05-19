use crate::entities::TreeType;
use crate::game::{Action, Game};
use crate::player::Plan;
use crate::position::Position;
use crate::utils::*;

use std::collections::{HashMap, HashSet};

use super::Player;

const MAX_PLANT_DIST: i32 = 4;
const TREE_SCAN_RADIUS: i32 = 8;
const HARVEST_SATURATION_THRESHOLD: f32 = 1.5;

struct PlantCandidate {
    troll_id: i32,
    tree_type: TreeType,
    target: Position,
    carries_seed: bool,
    score: i32,
}

impl Player {
    pub fn planting(&mut self, game: &Game) -> Option<Plan> {
        let shack = game.shack(self.side);
        let trolls = game.trolls_for(self.side);
        let inv = game.inventory(self.side);

        let no_blocked = HashSet::new();
        let shack_dist_map = bfs_distance_map(shack, &game.grid, &no_blocked);

        let plant_spots = self.find_plant_spots(game, &shack_dist_map);
        if plant_spots.is_empty() {
            return None;
        }

        let tree_priority = self.rank_tree_types_by_scarcity(game, &shack_dist_map);

        if self.harvest_saturated(game, &trolls, &shack_dist_map) {
            return None;
        }

        // --- Score all (troll, tree_type, spot) combinations
        let mut candidates: Vec<PlantCandidate> = Vec::new();

        for (tree_type, abundance) in tree_priority.iter().take(2) {
            for troll in trolls.iter() {
                let carries_seed = troll.carries_resource(tree_type.as_resource_type()) > 0;
                let could_pickup = troll.position.manhattan(&shack) == 1
                    && !troll.has_cargo()
                    && inv.get_by_tree(tree_type) > 0;

                if !carries_seed && !could_pickup {
                    continue;
                }

                let pickup_penalty = if could_pickup && !carries_seed {
                    troll.movement_speed
                } else {
                    0
                };

                for &(pos, spot_score) in &plant_spots {
                    let dist = troll.position.manhattan(&pos) as i32;
                    candidates.push(PlantCandidate {
                        troll_id: troll.id,
                        tree_type: *tree_type,
                        target: pos,
                        carries_seed,
                        score: dist + abundance + pickup_penalty - spot_score,
                    });
                }
            }
        }

        candidates.sort_by_key(|c| c.score);

        // Prefer trolls already carrying a seed (no pickup needed)
        if let Some(c) = candidates.iter().find(|c| c.carries_seed) {
            eprintln!("[PLANTING] troll {} will plant {:?} at {:?}", c.troll_id, c.tree_type, c.target);
            return Some(Plan {
                troll_id: c.troll_id,
                to: c.target,
                action: Action::Plant(c.troll_id, c.tree_type),
            });
        }

        // Otherwise, issue a pickup command (troll stays in place this turn)
        if let Some(c) = candidates.iter().find(|c| !c.carries_seed) {
            let troll_pos = trolls.iter().find(|t| t.id == c.troll_id).unwrap().position;
            eprintln!("[PLANTING] troll {} will pick {:?} then plant", c.troll_id, c.tree_type);
            return Some(Plan {
                troll_id: c.troll_id,
                to: troll_pos,
                action: Action::Pick(c.troll_id, c.tree_type),
            });
        }

        None
    }

    /// Find walkable empty tiles near the shack, scored by distance and water proximity.
    fn find_plant_spots(
        &self,
        game: &Game,
        shack_dist_map: &HashMap<Position, (i32, Position)>,
    ) -> Vec<(Position, i32)> {
        let mut spots: Vec<(Position, i32)> = shack_dist_map
            .iter()
            .filter(|(_, (dist, _))| *dist > 1 && *dist <= MAX_PLANT_DIST)
            .filter(|(pos, _)| game.grid[**pos] == b'.')
            .map(|(&pos, &(dist, _))| {
                let water_bonus = if game.is_near_water(pos) { 3 } else { 0 };
                (pos, water_bonus - dist)
            })
            .collect();

        spots.sort_by_key(|(_, score)| -score);
        spots
    }

    /// Rank tree types by scarcity near the shack (least abundant first).
    fn rank_tree_types_by_scarcity(
        &self,
        game: &Game,
        shack_dist_map: &HashMap<Position, (i32, Position)>,
    ) -> Vec<(TreeType, i32)> {
        let mut abundance: HashMap<TreeType, i32> = HashMap::from([
            (TreeType::Apple, 0),
            (TreeType::Plum, 0),
            (TreeType::Lemon, 0),
            (TreeType::Banana, 0),
        ]);

        for tree in &game.trees {
            let dist = shack_dist_map
                .get(&tree.position)
                .map(|(d, _)| *d)
                .unwrap_or(9999);
            if dist <= TREE_SCAN_RADIUS {
                let weight = if game.is_near_water(tree.position) { 3 } else { 2 };
                *abundance.entry(tree.typ).or_default() += weight;
            }
        }

        let mut ranked: Vec<(TreeType, i32)> = abundance.into_iter().collect();
        ranked.sort_by_key(|(_, score)| *score);
        ranked
    }

    /// Returns true if existing trees already produce more fruit than trolls can harvest.
    fn harvest_saturated(
        &self,
        game: &Game,
        trolls: &[&crate::entities::Troll],
        shack_dist_map: &HashMap<Position, (i32, Position)>,
    ) -> bool {
        // Fruit production rate from mature nearby trees
        let fruit_rate: f32 = game.trees.iter()
            .filter(|t| t.size >= 4)
            .filter_map(|t| {
                let dist = shack_dist_map.get(&t.position).map(|(d, _)| *d)?;
                (dist <= TREE_SCAN_RADIUS).then(|| {
                    let cd = if game.is_near_water(t.position) {
                        t.cooldown_time_water()
                    } else {
                        t.cooldown_time()
                    };
                    1.0 / cd as f32
                })
            })
            .sum();

        // Closest mature tree distance (used as optimistic travel estimate)
        let min_tree_dist = game.trees.iter()
            .filter(|t| t.size >= 4)
            .filter_map(|t| shack_dist_map.get(&t.position).map(|(d, _)| *d))
            .filter(|&d| d <= TREE_SCAN_RADIUS)
            .min()
            .unwrap_or(4);

        // Harvesting bandwidth from available trolls
        let harvest_rate: f32 = trolls.iter()
            .filter(|t| !self.trolls_busy.contains(&t.id))
            .map(|t| {
                let round_trip = (min_tree_dist * 2) as f32 / t.movement_speed as f32 + 1.0;
                let fruits_per_trip = t.harvest_power.min(t.carry_capacity) as f32;
                fruits_per_trip / round_trip
            })
            .sum();

        if harvest_rate > 0.0 && fruit_rate / harvest_rate > HARVEST_SATURATION_THRESHOLD {
            eprintln!(
                "[PLANTING] Skipping: fruit {:.2}/t, harvest {:.2}/t ({:.0}% saturated)",
                fruit_rate, harvest_rate,
                (fruit_rate / harvest_rate) * 100.0
            );
            return true;
        }

        false
    }
}
