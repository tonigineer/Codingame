use crate::entities::{TreeType, Tree, Troll};
use crate::game::{Action, Game, Side};
use crate::grid;
use crate::position::{CARDINALS, Position};
use crate::prediction::{Predictable, Snapshot};

use std::collections::{HashMap, HashSet, VecDeque};

// ========================================================================
// Player
// ========================================================================

#[derive(Copy, Clone)]
struct Plan {
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

        self.priority.update(game);

        let shack = game.shack(self.side);
        let trolls = game.trolls_for(self.side);

        let mut busy: HashSet<i32> = HashSet::new();
        let mut claimed: HashSet<Position> = HashSet::new();
        let mut claimed_targets: HashSet<Position> = HashSet::new();

        let latest_plans = self.plans.clone();
        self.plans.clear();

        // --- Training new trolls
        if let Some(action) = self.training(game) {
            self.actions.push(action);
        }

        // --- Check if plans are still possible (tree may have been chopped)
        self.plans = self
            .plans
            .drain(..)
            .filter(|p| match p.action {
                Action::Harvest(_) => b"ABPL".contains(&game.grid[p.to]),
                Action::Chop(_) => b"+".contains(&game.grid[p.to]),
                _ => true,
            })
            .collect();

        // --- Planned action can be carried out
        for troll in trolls.iter() {
            if let Some(plan) = latest_plans.iter().find(|p| p.troll_id == troll.id) {
                if plan.to == troll.position {
                    self.actions.push(plan.action);
                    busy.insert(troll.id);
                    claimed.insert(troll.position);
                }
            }
        }

        // --- Check troll without plan can directly harvest
        let currently_busy = busy.clone();
        for troll in trolls.iter().filter(|t| !currently_busy.contains(&t.id)) {
            if troll.free_capacity() == 0 {
                continue;
            }

            let grid_char = &game.grid[troll.position];
            if b"ABPL+".contains(grid_char) {
                let action = match grid_char {
                    b'+' => Action::Mine(troll.id),
                    _ => Action::Harvest(troll.id),
                };
                self.actions.push(action);
                busy.insert(troll.id);
                claimed.insert(troll.position);
            }
        }

        let shack_dist_map = bfs_distance_map(shack, game, &claimed);
        let mut move_intents: Vec<(i32, Position, Position)> = Vec::new(); // (troll_id, from, to)

        // --- Movement or new plans
        let currently_busy = busy.clone();
        for troll in trolls.iter().filter(|t| !currently_busy.contains(&t.id)) {
            let troll_dist_map = bfs_distance_map(troll.position, game, &claimed);

            // --- Continue with plan and its action
            if let Some(plan) = latest_plans
                .iter()
                .find(|p| p.troll_id == troll.id && !busy.contains(&troll.id))
            {
                let destination = plan.to;

                if let Some(path) = reconstruct_path(troll.position, destination, &troll_dist_map) {
                    if !path.is_empty() {
                        let steps = path.len().min(troll.movement_speed as usize);
                        let new_position = path[steps - 1];
                        move_intents.push((troll.id, troll.position, new_position));
                    }
                };

                claimed_targets.insert(destination.clone());
                self.plans.push(*plan);
                busy.insert(troll.id);

                continue;
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
                        let new_position = path[steps - 1];
                        move_intents.push((troll.id, troll.position, new_position));
                    }
                }

                claimed_targets.insert(destination.clone());
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
                if i.0 != j.0 && i.2 == j.1 && j.2 == i.1 {
                    swap_ids.insert(i.0);
                    swap_ids.insert(j.0);
                }
            }
        }

        // Process swaps first
        for &(id, _from, to) in &move_intents {
            if swap_ids.contains(&id) {
                self.actions.push(Action::Move(id, to));
                final_claimed.insert(to);
            }
        }

        // Process non-swap moves
        for &(id, from, to) in &move_intents {
            if swap_ids.contains(&id) {
                continue;
            }
            if !final_claimed.contains(&to) {
                self.actions.push(Action::Move(id, to));
                final_claimed.insert(to);
            } else {
                // Target blocked — try to find any free adjacent tile
                // to make progress rather than sitting still
                let alt = CARDINALS
                    .iter()
                    .map(|&c| from + c)
                    .filter(|&p| {
                        game.grid.contains(p)
                            && b".ABPL".contains(&game.grid[p])
                            && !final_claimed.contains(&p)
                    })
                    .min_by_key(|p| {
                        // Prefer tiles closer to our plan destination
                        let plan_dest = self
                            .plans
                            .iter()
                            .find(|pl| pl.troll_id == id)
                            .map(|pl| pl.to)
                            .unwrap_or(from);
                        p.manhattan(&plan_dest)
                    });

                if let Some(alt_pos) = alt {
                    self.actions.push(Action::Move(id, alt_pos));
                    final_claimed.insert(alt_pos);
                }

                eprintln!("Troll {id} is completely stuck.");
                // else: completely stuck, skip this turn
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
        //
        // --- 1. Cargo full > droppping
        //
        if troll.has_cargo() && troll.free_capacity() == 0 {
            return self.find_deliver_target(game, troll, troll_dist_map, shack);
        }

        let mut best: Option<(Action, Position, i32)> = None;

        // --- 2. Score each tree with fruit
        for tree in game.trees.iter() {
            if claimed_targets.contains(&tree.position) {
                continue;
            }

            // Skip fruits that are not needed
            let weight = self.priority.weight_for_tree(tree);
            if weight == 0 {
                continue;
            }

            // Distance to tree
            let tile_dist = match troll_dist_map.get(&tree.position) {
                Some((d, _)) => *d,
                None => continue,
            };

            // Distance from tree back to shack (for round-trip estimate)
            let shack_return = shack_dist_map.get(&tree.position).map(|(d, _)| *d).unwrap();

            // Round trip not worth it
            let round_trip = tile_dist + shack_return;
            if round_trip >= game.turns_remaining() {
                continue;
            }

            // Fruits at arrival
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

            // Favour close trees with high-priority fruit
            let harvestable = fruits.min(troll.free_capacity());
            let score = harvestable + weight - tile_dist;

            if best.is_none() || score > best.unwrap().2 {
                best = Some((Action::Harvest(troll.id), tree.position, score));
            }
        }

        // --- 3. Score iron mining spots (if troll can mine)
        if troll.chop_power > 0 && self.priority.iron > 0 {
            for &mine in game.mines.iter() {
                for &c in CARDINALS.iter() {
                    // Find a walkable tile adjacent to this iron
                    let adj = mine + c;
                    if !game.grid.contains(adj) || !b".ABPL".contains(&game.grid[adj]) {
                        continue;
                    }

                    if claimed_targets.contains(&adj) {
                        continue;
                    }

                    // Distance to tree
                    let tile_dist = match troll_dist_map.get(&adj) {
                        Some((d, _)) => *d,
                        None => continue,
                    };

                    // Distance from tree back to shack (for round-trip estimate)
                    let shack_return = shack_dist_map.get(&adj).map(|(d, _)| *d).unwrap();

                    // Round trip not worth it
                    let round_trip = tile_dist + shack_return;
                    if round_trip >= game.turns_remaining() {
                        continue;
                    }

                    let score = troll.free_capacity() + self.priority.iron - tile_dist;

                    if best.is_none() || score > best.unwrap().2 {
                        best = Some((Action::Mine(troll.id), adj, score));
                    }
                }
            }
        }

        // --- 4. Score each tree for chopping (if troll can chop and wood is needed)
        if troll.chop_power > 0 && self.priority.wood > 0 {
            for tree in game.trees.iter() {
                if claimed_targets.contains(&tree.position) {
                    continue;
                }

                let tile_dist = match troll_dist_map.get(&tree.position) {
                    Some((d, _)) => *d,
                    None => continue,
                };

                // Turns to chop the tree down
                let chop_turns = (tree.health + troll.chop_power - 1) / troll.chop_power;

                // Only chop trees of decent size; do not care at the end.
                let shack_return = shack_dist_map.get(&tree.position).map(|(d, _)| *d).unwrap();
                let round_trip = tile_dist + chop_turns + shack_return;

                if tree.size < 2 && round_trip >= game.turns_remaining() {
                    continue;
                }

                // Wood yield: tree.size pieces
                let wood_yield = tree.size;
                let carriable = wood_yield.min(troll.free_capacity());

                let score = carriable + self.priority.wood - tile_dist - chop_turns;

                if best.is_none() || score > best.unwrap().2 {
                    best = Some((Action::Chop(troll.id), tree.position, score));
                }
            }
        }

        // If carrying cargo and found nothing better, deliver
        if best.is_none() && troll.has_cargo() {
            return self.find_deliver_target(game, troll, troll_dist_map, shack);
        }

        best.map(|(action, pos, _)| (action, pos))
    }

    /// --- Find the best shack-adjacent tile to deliver cargo.
    fn find_deliver_target(
        &self,
        game: &Game,
        troll: &Troll,
        troll_dist_map: &HashMap<Position, (i32, Position)>,
        shack: &Position,
    ) -> Option<(Action, Position)> {
        // Find the closest walkable tile adjacent to the shack
        let best_adj = CARDINALS
            .iter()
            .map(|&c| *shack + c)
            .filter(|&p| game.grid.contains(p) && b".ABPL".contains(&game.grid[p]))
            .filter_map(|p| troll_dist_map.get(&p).map(|(d, _)| (p, *d)))
            .min_by_key(|(_, d)| *d);

        best_adj.map(|(adj_pos, _)| (Action::Drop(troll.id), adj_pos))
    }

    // ========================================================================
    // Traininig
    // ========================================================================

    fn training(&self, game: &Game) -> Option<Action> {
        // --- Don't spawn new trolls, chopping wood is needed now
        if game.turns_remaining() < 120 {
            return None;
        }

        let trolls = game.trolls_for(self.side);

        let best_cc = trolls.iter().map(|t| t.carry_capacity).max().unwrap();
        let best_hp = trolls.iter().map(|t| t.harvest_power).max().unwrap();
        let best_cp = trolls.iter().map(|t| t.chop_power).max().unwrap();
        let best_ms = trolls.iter().map(|t| t.movement_speed).max().unwrap();

        let mut best: Option<(Action, i32)> = None;

        for ms in 1..=5 {
            for cc in 1..=5 {
                for hp in 1..=5 {
                    for cp in 1..=5 {
                        // At least on attribute must be better in order to train a new troll
                        let dominated =
                            cc <= best_cc && hp <= best_hp && cp <= best_cp && ms <= best_ms;

                        // First turn can be `any` troll
                        if dominated && game.turn > 1 {
                            continue;
                        }

                        if !game.can_train(self.side, ms, cc, hp, cp) {
                            continue;
                        }

                        let score = cc + hp + cp + ms;

                        // Train best troll that is possible
                        if best.is_none() || score > best.as_ref().unwrap().1 {
                            best = Some((Action::Train(ms, cc, hp, cp), score));
                        }
                    }
                }
            }
        }

        best.map(|(action, _)| action)
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

// ========================================================================
// BFS utilities
// ========================================================================

fn bfs_distance_map(
    from: Position,
    game: &Game,
    blocked: &HashSet<Position>,
) -> HashMap<Position, (i32, Position)> {
    let mut map: HashMap<Position, (i32, Position)> = HashMap::new();
    let mut queue: VecDeque<Position> = VecDeque::new();
    map.insert(from, (0, from));
    queue.push_back(from);

    while let Some(cur) = queue.pop_front() {
        let cur_dist = map[&cur].0;
        for &c in CARDINALS.iter() {
            let next = cur + c;
            if map.contains_key(&next) || !game.grid.contains(next) {
                continue;
            }
            if !b".ABPL".contains(&game.grid[next]) {
                continue;
            }
            if blocked.contains(&next) {
                continue;
            }
            map.insert(next, (cur_dist + 1, cur));
            queue.push_back(next);
        }
    }
    map
}

fn reconstruct_path(
    from: Position,
    to: Position,
    dist_map: &HashMap<Position, (i32, Position)>,
) -> Option<Vec<Position>> {
    if from == to {
        return Some(Vec::new());
    }
    if !dist_map.contains_key(&to) {
        return None;
    }
    let mut path = Vec::new();
    let mut cur = to;
    while cur != from {
        path.push(cur);
        cur = dist_map[&cur].1;
    }
    path.reverse();
    Some(path)
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
}
