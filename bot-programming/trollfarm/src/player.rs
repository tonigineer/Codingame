use crate::entities::Troll;
use crate::game::{Action, Game, Side};
use crate::position::{Position, CARDINALS};
use crate::prediction::{Predictable, Snapshot};

use std::collections::{HashMap, HashSet, VecDeque};

// ------------------------------------------------------------------------
// Harvest‑loop planner: for one troll, find the best sequence of
// (go to tree → harvest → go to shack → drop) cycles within a horizon.
// ------------------------------------------------------------------------

/// Compact state for a single troll during simulation.
#[derive(Clone, Debug)]
struct TrollSim {
    pos: Position,
    carried: i32,      // fruit currently carried
    capacity: i32,     // max carry
    speed: i32,        // move_speed (tiles per turn)
    harvest_power: i32,
}

/// One planned action in the simulation output.
#[derive(Clone, Debug)]
enum PlanStep {
    MoveTo(Position),  // single‑tile move
    Harvest,
    Drop,
}

/// Result of planning for one troll.
#[derive(Clone, Debug)]
struct TrollPlan {
    troll_id: i32,
    steps: Vec<PlanStep>,
    total_delivered: i32,
}

// ------------------------------------------------------------------------
// BFS utilities — distance maps & path reconstruction
// ------------------------------------------------------------------------

/// BFS distance map from a single source. Returns (distance, parent) for
/// every reachable walkable tile. Computed once, queried many times → O(grid).
fn bfs_distance_map(from: Position, game: &Game) -> HashMap<Position, (i32, Position)> {
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
            map.insert(next, (cur_dist + 1, cur));
            queue.push_back(next);
        }
    }
    map
}

/// Reconstruct path from a distance map. Returns the sequence of tiles
/// to walk (excluding `from`, including `to`).
fn reconstruct_path(from: Position, to: Position, dist_map: &HashMap<Position, (i32, Position)>) -> Option<Vec<Position>> {
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

/// Single BFS path (convenience, used only for the final deliver at end).
fn bfs_path(from: Position, to: Position, game: &Game) -> Option<Vec<Position>> {
    let map = bfs_distance_map(from, game);
    reconstruct_path(from, to, &map)
}

// ------------------------------------------------------------------------
// Pre‑computed shack adjacency info
// ------------------------------------------------------------------------

struct ShackInfo {
    /// Walkable tiles adjacent to the shack (where a troll can stand to Drop).
    adj_tiles: Vec<Position>,
    /// Distance from each walkable tile on the map to the nearest shack‑adjacent
    /// tile. Computed once via multi‑source BFS from all adj_tiles.
    dist_to_shack_adj: HashMap<Position, (i32, Position)>,
}

impl ShackInfo {
    fn compute(shack: Position, game: &Game) -> Self {
        let adj_tiles: Vec<Position> = CARDINALS
            .iter()
            .map(|&c| shack + c)
            .filter(|&p| game.grid.contains(p) && b".ABPL".contains(&game.grid[p]))
            .collect();

        // Multi‑source BFS from all shack‑adjacent tiles
        let mut dist_map: HashMap<Position, (i32, Position)> = HashMap::new();
        let mut queue: VecDeque<Position> = VecDeque::new();
        for &a in &adj_tiles {
            dist_map.insert(a, (0, a));
            queue.push_back(a);
        }
        while let Some(cur) = queue.pop_front() {
            let cur_dist = dist_map[&cur].0;
            for &c in CARDINALS.iter() {
                let next = cur + c;
                if dist_map.contains_key(&next) || !game.grid.contains(next) {
                    continue;
                }
                if !b".ABPL".contains(&game.grid[next]) {
                    continue;
                }
                dist_map.insert(next, (cur_dist + 1, cur));
                queue.push_back(next);
            }
        }

        ShackInfo { adj_tiles, dist_to_shack_adj: dist_map }
    }

    /// Distance from `pos` to the nearest shack‑adjacent tile.
    fn dist_from(&self, pos: Position) -> Option<i32> {
        self.dist_to_shack_adj.get(&pos).map(|(d, _)| *d)
    }

    /// Nearest shack‑adjacent tile reachable from `pos`, plus path.
    fn nearest_adj_from(&self, pos: Position, game: &Game) -> Option<(Position, Vec<Position>)> {
        // We need the path from pos → nearest adj tile.
        // The multi‑source BFS gives us dist, but the parent pointers go
        // *toward* the shack, so we can't directly reconstruct forward path.
        // Instead, do a quick single‑source BFS from pos, then pick the
        // closest adj tile.
        if self.adj_tiles.contains(&pos) {
            return Some((pos, Vec::new()));
        }
        let map = bfs_distance_map(pos, game);
        self.adj_tiles
            .iter()
            .filter_map(|&a| {
                map.get(&a).and_then(|(d, _)| {
                    reconstruct_path(pos, a, &map).map(|path| (a, *d, path))
                })
            })
            .min_by_key(|(_, d, _)| *d)
            .map(|(a, _, path)| (a, path))
    }
}

// ------------------------------------------------------------------------
// Per‑troll greedy planner
// ------------------------------------------------------------------------

/// For a single troll, greedily plan harvest loops over `horizon` steps.
///
/// Algorithm:
///   While steps remain in the horizon:
///     1. BFS from current pos once → get distances to all trees and shack.
///     2. Score each tree by fruit‑per‑step for the full round trip.
///     3. Walk → harvest → walk to shack → drop. Repeat.
fn plan_troll(
    troll: &TrollSim,
    troll_id: i32,
    shack_info: &ShackInfo,
    game: &Game,
    horizon: i32,
    used_trees: &mut HashSet<Position>,
) -> TrollPlan {
    let mut plan = TrollPlan {
        troll_id,
        steps: Vec::new(),
        total_delivered: 0,
    };

    let mut pos = troll.pos;
    let mut carried = troll.carried;
    let capacity = troll.capacity;
    let mut steps_left = horizon;

    // Snapshot of fruit availability (decremented as we harvest in simulation)
    let mut fruit_remaining: HashMap<Position, i32> = game
        .trees
        .iter()
        .filter(|t| t.fruits > 0)
        .map(|t| (t.position, t.fruits))
        .collect();

    // Safety: cap iterations to prevent infinite loops
    let max_iters = (horizon + 5) as usize;
    let mut iters = 0;

    loop {
        iters += 1;
        if steps_left <= 0 || iters > max_iters {
            break;
        }

        // --- Deliver if full or if we must deliver before time runs out ---
        if carried > 0 && carried >= capacity {
            if !try_deliver(&mut plan, &mut pos, &mut carried, &mut steps_left, shack_info, game) {
                break;
            }
            continue;
        }

        // One BFS from current position to score all reachable trees
        let dist_map = bfs_distance_map(pos, game);

        // Check if we can even reach the shack to deliver later
        let dist_to_shack = shack_info.adj_tiles.iter()
            .filter_map(|a| dist_map.get(a).map(|(d, _)| *d))
            .min()
            .unwrap_or(9999);

        // Must deliver now if not enough time to do anything else
        if carried > 0 && steps_left <= dist_to_shack + 1 {
            if !try_deliver(&mut plan, &mut pos, &mut carried, &mut steps_left, shack_info, game) {
                break;
            }
            continue;
        }

        // --- Find best tree ---
        let free_space = capacity - carried;
        if free_space <= 0 {
            continue;
        }

        let mut best: Option<(Position, i32)> = None; // (tree_pos, score)

        for (&tree_pos, &fruits) in &fruit_remaining {
            if fruits <= 0 || used_trees.contains(&tree_pos) {
                continue;
            }
            let dist_to_tree = match dist_map.get(&tree_pos) {
                Some((d, _)) => *d,
                None => continue,
            };
            let dist_tree_to_shack = match shack_info.dist_from(tree_pos) {
                Some(d) => d,
                None => continue,
            };

            let harvestable = fruits.min(free_space);
            let harvest_turns = (harvestable + troll.harvest_power - 1) / troll.harvest_power;
            let total_steps = dist_to_tree + harvest_turns + dist_tree_to_shack + 1;

            if total_steps > steps_left {
                continue;
            }

            let score = harvestable * 1000 / total_steps.max(1);
            if best.is_none() || score > best.unwrap().1 {
                best = Some((tree_pos, score));
            }
        }

        if let Some((tree_pos, _)) = best {
            // Walk to tree using the dist_map we already have
            if let Some(path) = reconstruct_path(pos, tree_pos, &dist_map) {
                let walk_dist = path.len() as i32;
                for &step in &path {
                    plan.steps.push(PlanStep::MoveTo(step));
                }
                pos = tree_pos;
                steps_left -= walk_dist;
            }

            // Harvest
            let avail = fruit_remaining.get(&tree_pos).copied().unwrap_or(0);
            let harvestable = avail.min(capacity - carried);
            let mut harvested = 0;
            while harvested < harvestable && steps_left > 0 {
                let pick = troll.harvest_power.min(harvestable - harvested);
                plan.steps.push(PlanStep::Harvest);
                harvested += pick;
                carried += pick;
                steps_left -= 1;
            }

            if let Some(fr) = fruit_remaining.get_mut(&tree_pos) {
                *fr -= harvested;
                if *fr <= 0 {
                    used_trees.insert(tree_pos);
                }
            }
        } else {
            // No reachable tree — deliver if carrying, else stop
            if carried > 0 {
                if !try_deliver(&mut plan, &mut pos, &mut carried, &mut steps_left, shack_info, game) {
                    break;
                }
                continue;
            }
            break;
        }
    }

    // Final deliver if we still have cargo
    if carried > 0 {
        try_deliver(&mut plan, &mut pos, &mut carried, &mut steps_left, shack_info, game);
    }

    plan
}

/// Try to walk to a shack‑adjacent tile and drop. Returns false if impossible.
fn try_deliver(
    plan: &mut TrollPlan,
    pos: &mut Position,
    carried: &mut i32,
    steps_left: &mut i32,
    shack_info: &ShackInfo,
    game: &Game,
) -> bool {
    if *carried == 0 {
        return true;
    }
    if let Some((adj_pos, path)) = shack_info.nearest_adj_from(*pos, game) {
        let walk_dist = path.len() as i32;
        if walk_dist + 1 > *steps_left {
            return false;
        }
        for &step in &path {
            plan.steps.push(PlanStep::MoveTo(step));
        }
        *pos = adj_pos;
        *steps_left -= walk_dist;

        plan.steps.push(PlanStep::Drop);
        plan.total_delivered += *carried;
        *carried = 0;
        *steps_left -= 1;
        true
    } else {
        false
    }
}

// ========================================================================
// Player
// ========================================================================

pub struct Player {
    pub side: Side,
    pub actions: Vec<Action>,
    predicted: Option<Snapshot>,
    prev_positions: HashMap<i32, Position>,
}

impl Player {
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            side,
            actions: Vec::new(),
            predicted: None,
            prev_positions: HashMap::new(),
        }
    }

    fn remaining(&self, game: &Game) -> i32 {
        Game::MAX_TURNS - game.turn as i32
    }

    // ====================================================================
    // THINK — main entry point
    // ====================================================================

    pub fn think(&mut self, game: &Game) {
        self.actions.clear();

        let remaining = self.remaining(game);
        let shack = game.shack(self.side);

        // ------------------------------------------------------------------
        // Part 1: Training — want trolls with carry_capacity >= 2, then >= 3
        // Train config: (move_speed, carry_capacity, harvest_power, chop_power)
        // We want cc=2 first, then cc=3. Keep other stats at 1/0.
        // ------------------------------------------------------------------
        if let Some(train_action) = self.pick_training(game) {
            self.actions.push(train_action);
        }

        // ------------------------------------------------------------------
        // Part 2: Simulate 20 steps ahead — greedy harvest loops per troll
        // ------------------------------------------------------------------
        let trolls = game.trolls_for(self.side);
        let horizon = 20.min(remaining);

        // Pre‑compute shack adjacency + distance map (one BFS for the whole turn)
        let shack_info = ShackInfo::compute(shack, game);

        // Build troll sims
        let troll_sims: Vec<(i32, TrollSim)> = trolls
            .iter()
            .map(|t| {
                (
                    t.id,
                    TrollSim {
                        pos: t.position,
                        carried: t.total_carried(),
                        capacity: t.free_capacity() + t.total_carried(),
                        speed: 1,
                        harvest_power: t.harvest_power,
                    },
                )
            })
            .collect();

        // Sort trolls by distance to nearest fruit tree (closest first gets priority)
        let mut ordered: Vec<(i32, TrollSim)> = troll_sims;
        ordered.sort_by_key(|(_, sim)| {
            game.trees
                .iter()
                .filter(|t| t.fruits > 0)
                .map(|t| sim.pos.manhattan(&t.position) as i32)
                .min()
                .unwrap_or(9999)
        });

        // Plan each troll greedily; used_trees prevents double-claiming
        let mut used_trees: HashSet<Position> = HashSet::new();
        let mut plans: HashMap<i32, TrollPlan> = HashMap::new();

        for (troll_id, sim) in &ordered {
            let plan = plan_troll(sim, *troll_id, &shack_info, game, horizon, &mut used_trees);
            eprintln!(
                "[PLAN] troll {} : {} steps, delivers {} fruit",
                troll_id,
                plan.steps.len(),
                plan.total_delivered
            );
            plans.insert(*troll_id, plan);
        }

        // Extract the FIRST action from each troll's plan
        let mut claimed_tiles: HashSet<Position> = HashSet::new();

        for troll in &trolls {
            if let Some(plan) = plans.get(&troll.id) {
                if let Some(first_step) = plan.steps.first() {
                    match first_step {
                        PlanStep::MoveTo(target) => {
                            if !claimed_tiles.contains(target) {
                                claimed_tiles.insert(*target);
                                self.actions.push(Action::Move(troll.id, *target));
                            }
                            // else: collision, skip this turn (will re-plan next tick)
                        }
                        PlanStep::Harvest => {
                            self.actions.push(Action::Harvest(troll.id));
                        }
                        PlanStep::Drop => {
                            self.actions.push(Action::Drop(troll.id));
                        }
                    }
                }
                // No steps = idle, that's fine
            }
        }

        // Save positions for next turn
        self.prev_positions = trolls.iter().map(|t| (t.id, t.position)).collect();
    }

    // ====================================================================
    // TRAINING — cc2 first, then cc3
    // ====================================================================

    fn pick_training(&self, game: &Game) -> Option<Action> {
        let remaining = self.remaining(game);
        if remaining < 30 {
            return None;
        }

        // Priority order: train a troll with cc=2 first, then cc=3
        // Config: (ms, cc, hp, cp)
        let configs = [
            (1, 2, 1, 0), // cc=2 is our first priority
            (1, 3, 1, 0), // cc=3 is our second priority
        ];

        for &(ms, cc, hp, cp) in &configs {
            if game.can_train(self.side, ms, cc, hp, cp) {
                eprintln!("[TRAIN] training troll with cc={cc}");
                return Some(Action::Train(ms, cc, hp, cp));
            }
        }

        None
    }

    // ====================================================================
    // SIMULATION (prediction comparison, unchanged)
    // ====================================================================

    pub fn simulate(&mut self, game: &Game) {
        self.predicted = Some(game.snapshot(&self.actions, &[]));
    }

    pub fn compare(&self, game: &Game) {
        if let Some(snapshot) = &self.predicted {
            game.compare(snapshot);
        }
    }
}
