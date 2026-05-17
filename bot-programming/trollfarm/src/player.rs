use crate::entities::{TreeType, Tree, Troll};
use crate::game::{Action, Game, Side};
use crate::position::{CARDINALS, Position};
use crate::prediction::{Predictable, Snapshot};
use crate::utils::*;
use crate::player_training::Training;
use crate::players::{Test};

use std::collections::{HashMap, HashSet};

// ========================================================================
// Player
// ========================================================================

#[derive(Debug, Copy, Clone)]
pub struct Plan {
    troll_id: i32,
    to: Position,
    action: Action,
}

pub struct Player {
    pub side: Side,
    pub actions: Vec<Action>,
    predicted: Option<Snapshot>,
    prev_positions: HashMap<i32, Position>,
    priority: Priority,
    plans: Vec<Plan>,
}

impl Training for Player {}

impl Player {
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            side,
            actions: Vec::new(),
            predicted: None,
            prev_positions: HashMap::new(),
            priority: Priority::new(),
            plans: Vec::new(),
        }
    }

    // ====== THINK ==================================================
    pub fn think(&mut self, game: &Game) {
        self.actions.clear();

        Test::testing();

        eprintln!("{:?}", self.plans);

        self.priority.update(game);

        let shack = game.shack(self.side);
        let trolls = game.trolls_for(self.side);

        let mut busy: HashSet<i32> = HashSet::new();
        let mut claimed: HashSet<Position> = HashSet::new();
        let mut claimed_targets: HashSet<Position> = HashSet::new();

        // --- Training new trolls
        if let Some(action) = Player::training(&game, self.side) {
            self.actions.push(action);
        }

        // --- Planting new trees
        // if let Some(action) = Player::planting(&game, self.side) {
        //     self.actions.push(action);
        // }

        // --- Plant trees
        // self.planting(game, &mut busy, &mut claimed, &mut claimed_targets);

        // --- Check if plans are still possible
        let latest_plans: Vec<Plan> = self
            .plans
            .drain(..)
            .filter(|p| match p.action {
                Action::Harvest(_) => {
                    b"ABPL".contains(&game.grid[p.to])
                        && game.tree_at(p.to).map(|t| t.fruits > 0).unwrap_or(false)
                }
                Action::Chop(_) => game.tree_at(p.to).map(|t| t.health > 0).unwrap_or(false),
                _ => true,
            })
            .collect();

        // --- Planned action can be carried out (troll arrived at destination)
        for troll in trolls.iter() {
            if let Some(plan) = latest_plans.iter().find(|p| p.troll_id == troll.id) {
                if plan.to == troll.position {
                    self.actions.push(plan.action);
                    busy.insert(troll.id);
                    claimed.insert(troll.position);
                    claimed_targets.insert(troll.position);
                }
            }
        }

        // --- Check troll without plan can directly harvest (opportunistic)
        let currently_busy = busy.clone();
        for troll in trolls.iter().filter(|t| !currently_busy.contains(&t.id)) {
            if troll.free_capacity() == 0 {
                continue;
            }

            let grid_char = &game.grid[troll.position];

            if b"+".contains(grid_char) && troll.chop_power > 0 {
                self.actions.push(Action::Mine(troll.id));
                busy.insert(troll.id);
                claimed.insert(troll.position);
                claimed_targets.insert(troll.position);
                continue;
            }

            if b"ABPL".contains(grid_char) {
                let has_fruit = game
                    .tree_at(troll.position)
                    .map(|t| t.fruits > 0 && self.priority.weight_for_tree(t) > 0)
                    .unwrap_or(false);
                if has_fruit {
                    self.actions.push(Action::Harvest(troll.id));
                    busy.insert(troll.id);
                    claimed.insert(troll.position);
                    claimed_targets.insert(troll.position);
                }
            }
        }

        // Also mark targets of plans that are still in-progress (trolls
        // en route from last turn that haven't arrived yet)
        for plan in &latest_plans {
            if !busy.contains(&plan.troll_id) {
                claimed_targets.insert(plan.to);
            }
        }

        let no_blocked = HashSet::new();
        let shack_dist_map = bfs_distance_map(shack, &game.grid, &no_blocked);

        // (troll_id, from, path_tiles_up_to_speed, speed)
        let mut move_intents: Vec<(i32, Position, Vec<Position>, i32)> = Vec::new();

        // --- Movement or new plans
        let currently_busy = busy.clone();
        for troll in trolls.iter().filter(|t| !currently_busy.contains(&t.id)) {
            let troll_dist_map = bfs_distance_map(troll.position, &game.grid, &no_blocked);

            // --- Continue with existing plan
            if let Some(plan) = latest_plans
                .iter()
                .find(|p| p.troll_id == troll.id && !busy.contains(&troll.id))
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
                    // claimed_targets already has this destination from the
                    // pre-loop insertion above
                    self.plans.push(*plan);
                    busy.insert(troll.id);
                    continue;
                }
                // Fall through: plan's destination was already in claimed_targets,
                // remove it so find_best_target can reassign
                claimed_targets.remove(&plan.to);
            }

            // --- New plan: find best target based on priority ---
            if let Some((action, destination)) = self.find_best_target(
                game,
                &troll,
                &troll_dist_map,
                &shack_dist_map,
                &shack,
                &claimed_targets,
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

                claimed_targets.insert(destination);
                self.plans.push(Plan {
                    troll_id: troll.id,
                    to: destination,
                    action,
                });
                busy.insert(troll.id);
            }
        }

        // --- Resolve collisions ---
        let mut final_claimed: HashSet<Position> = claimed.clone();

        // Detect swaps (A→B and B→A) — allow both
        let mut swap_ids: HashSet<i32> = HashSet::new();
        for i in &move_intents {
            for j in &move_intents {
                let i_to = i.2.last().copied().unwrap_or(i.1);
                let j_to = j.2.last().copied().unwrap_or(j.1);
                if i.0 != j.0 && i_to == j.1 && j_to == i.1 {
                    swap_ids.insert(i.0);
                    swap_ids.insert(j.0);
                }
            }
        }

        // Process swaps first
        for (id, _from, path, _speed) in &move_intents {
            if swap_ids.contains(id) {
                if let Some(&to) = path.last() {
                    self.actions.push(Action::Move(*id, to));
                    final_claimed.insert(to);
                }
            }
        }

        // Process non-swap moves: check ALL tiles in the path
        for (id, from, path, speed) in &move_intents {
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
                        if alt_path[..steps].iter().all(|p| !final_claimed.contains(p)) {
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
                        .map(|(&p, &(d, _))| (p, p.manhattan(&plan_dest) as i32))
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

        self.prev_positions = trolls.iter().map(|t| (t.id, t.position)).collect();
    }

    // ====================================================================
    // Find best target for a troll based on priorities
    // ====================================================================

    fn find_best_target(
        &self,
        game: &Game,
        troll: &Troll,
        troll_dist_map: &HashMap<Position, (i32, Position)>,
        shack_dist_map: &HashMap<Position, (i32, Position)>,
        shack: &Position,
        claimed_targets: &HashSet<Position>,
    ) -> Option<(Action, Position)> {
        if troll.has_cargo() && troll.free_capacity() == 0 {
            return self.find_deliver_target(game, troll, troll_dist_map, shack);
        }

        let mut best: Option<(Action, Position, i32)> = None;

        // --- 2. Score each tree with fruit
        for tree in game.trees.iter() {
            if claimed_targets.contains(&tree.position) {
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
            let fruit_at_arrival = |tree: &Tree, tile_dist: i32| -> i32 {
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

        // --- 3. Score iron mining spots
        if troll.chop_power > 0 && self.priority.iron > 0 {
            for &mine in game.mines.iter() {
                for &c in CARDINALS.iter() {
                    let adj = mine + c;
                    if !game.grid.contains(adj) || !b".ABPL".contains(&game.grid[adj]) {
                        continue;
                    }
                    if claimed_targets.contains(&adj) {
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

        // --- 4. Score each tree for chopping
        if troll.chop_power > 0 && self.priority.wood > 0 {
            for tree in game.trees.iter() {
                if claimed_targets.contains(&tree.position) {
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

    fn find_deliver_target(
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

    // ========================================================================
    // Simulation
    // ========================================================================

    pub fn simulate(&mut self, game: &Game) {
        self.predicted = Some(game.snapshot(&self.actions, &[]));
    }

    pub fn compare(&self, game: &Game) {
        if let Some(snapshot) = &self.predicted {
            game.compare(snapshot);
        }
    }
}

// ====================================================================
// SET PRIORITIES
// ====================================================================

struct Priority {
    apple: i32,
    banana: i32,
    lemon: i32,
    plum: i32,
    iron: i32,
    wood: i32,
}

impl Priority {
    fn new() -> Self {
        Self {
            apple: 0,
            banana: 0,
            lemon: 0,
            plum: 0,
            iron: 0,
            wood: 0,
        }
    }

    fn update(&mut self, game: &Game) {
        let inv = game.inventory(Side::Me);

        let min_fruit_stock = 16;
        let min_iron_stock = 10;

        self.apple = (min_fruit_stock - inv.get(&TreeType::Apple)).max(0);
        self.banana = (min_fruit_stock - inv.get(&TreeType::Banana)).max(0);
        self.lemon = (min_fruit_stock - inv.get(&TreeType::Lemon)).max(0);
        self.plum = (min_fruit_stock - inv.get(&TreeType::Plum)).max(0);
        self.iron = (min_iron_stock - inv.iron).max(0);
        self.wood = (180 / game.turns_remaining().max(1)).min(1);
    }

    fn weight_for_tree(&self, tree: &Tree) -> i32 {
        match tree.typ {
            TreeType::Apple => self.apple,
            TreeType::Banana => self.banana,
            TreeType::Lemon => self.lemon,
            TreeType::Plum => self.plum,
        }
    }

    fn weight_for_type(&self, typ: TreeType) -> i32 {
        match typ {
            TreeType::Apple => self.apple,
            TreeType::Banana => self.banana,
            TreeType::Lemon => self.lemon,
            TreeType::Plum => self.plum,
        }
    }
}
