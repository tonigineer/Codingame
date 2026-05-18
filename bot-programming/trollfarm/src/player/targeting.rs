use crate::entities::Troll;
use crate::game::{Action, Game};
use crate::position::{CARDINALS, Position};

use std::collections::HashMap;

use super::Player;

impl Player {
    pub fn find_best_target(
        &self,
        game: &Game,
        troll: &Troll,
        troll_dist_map: &HashMap<Position, (i32, Position)>,
        shack_dist_map: &HashMap<Position, (i32, Position)>,
        shack: &Position,
    ) -> Option<(Action, Position)> {
        if troll.has_cargo() && troll.free_capacity() == 0 {
            return self.find_deliver_target(game, troll, troll_dist_map, shack);
        }

        let mut best: Option<(Action, Position, i32)> = None;

        // --- Score each tree with fruit
        for tree in game.trees.iter() {
            if self.claimed_entities.contains(&tree.position) {
                continue;
            }

            let weight = self.priority.weight_for_tree(tree);
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
                best = Some((Action::Harvest(troll.id), tree.position, score));
            }
        }

        // --- Score iron mining spots
        if troll.chop_power > 0 && self.priority.iron > 0 {
            for &mine in game.mines.iter() {
                for &c in CARDINALS.iter() {
                    let adj = mine + c;
                    if !game.grid.contains(adj) || !b".ABPL".contains(&game.grid[adj]) {
                        continue;
                    }
                    if self.claimed_entities.contains(&adj) {
                        continue;
                    }
                    let tile_dist = match troll_dist_map.get(&adj) {
                        Some((d, _)) => *d,
                        None => continue,
                    };
                    let shack_return = shack_dist_map.get(&adj).map(|(d, _)| *d).unwrap_or(9999);
                    let round_trip = tile_dist + shack_return;
                    if round_trip >= game.turns_remaining() {
                        continue;
                    }
                    let score = troll.free_capacity() + self.priority.iron * 2 - tile_dist;
                    if best.is_none() || score > best.unwrap().2 {
                        best = Some((Action::Mine(troll.id), adj, score));
                    }
                }
            }
        }

        // --- Score each tree for chopping
        if troll.chop_power > 0 && self.priority.wood > 0 {
            for tree in game.trees.iter() {
                if self.claimed_entities.contains(&tree.position) {
                    continue;
                }
                let tile_dist = match troll_dist_map.get(&tree.position) {
                    Some((d, _)) => *d,
                    None => continue,
                };
                let chop_turns = (tree.health + troll.chop_power - 1) / troll.chop_power;
                let shack_return = shack_dist_map
                    .get(&tree.position)
                    .map(|(d, _)| *d)
                    .unwrap_or(9999);
                let round_trip = tile_dist + chop_turns + shack_return;
                if tree.size < 2 && round_trip >= game.turns_remaining() {
                    continue;
                }
                let wood_yield = tree.size;
                let carriable = wood_yield.min(troll.free_capacity());
                let score = carriable + self.priority.wood * 2 - tile_dist - chop_turns;
                if best.is_none() || score > best.unwrap().2 {
                    best = Some((Action::Chop(troll.id), tree.position, score));
                }
            }
        }

        if best.is_none() && troll.has_cargo() {
            return self.find_deliver_target(game, troll, troll_dist_map, shack);
        }

        best.map(|(action, pos, _)| (action, pos))
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

    // --- Opportunistic actions for idle trolls on harvestable/mineable tiles
    pub fn opportunistic_actions(&mut self, game: &Game, trolls: &[&Troll]) {
        let currently_busy = self.trolls_busy.clone();
        'troll: for troll in trolls.iter().filter(|t| !currently_busy.contains(&t.id)) {
            if troll.free_capacity() == 0 {
                continue;
            }

            if troll.chop_power > 0 {
                for delta in CARDINALS {
                    let adj_position = troll.position + delta;
                    if !game.grid.contains(adj_position) {
                        continue;
                    }

                    if b"+".contains(&game.grid[adj_position]) {
                        self.actions.push(Action::Mine(troll.id));
                        self.trolls_busy.insert(troll.id);
                        self.positions_claimed.insert(troll.position);
                        self.claimed_entities.insert(troll.position);

                        continue 'troll;
                    }
                }
            }

            if b"ABPL".contains(&game.grid[troll.position]) {
                let has_fruit = game
                    .tree_at(troll.position)
                    .map(|t| t.fruits > 0 && self.priority.weight_for_tree(t) > 0)
                    .unwrap_or(false);
                if has_fruit {
                    self.actions.push(Action::Harvest(troll.id));
                    self.trolls_busy.insert(troll.id);
                    self.positions_claimed.insert(troll.position);
                    self.claimed_entities.insert(troll.position);

                    continue 'troll;
                }
            }
        }
    }
}
