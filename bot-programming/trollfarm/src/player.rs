use crate::position::{Position, CARDINALS};
use crate::game::Game;
use crate::prediction::{Predictable, Snapshot};
use crate::game::{Action, Side};
use crate::entities::{Troll, Tree, TreeType};

use std::collections::{HashSet, HashMap};

pub struct Player {
    pub side: Side,
    pub actions: Vec<Action>,
    predicted: Option<Snapshot>,
}

impl Player {
    #[must_use]
    pub fn new(side: Side) -> Self {
        Self {
            side,
            actions: Vec::new(),
            predicted: None,
        }
    }

    // --------------------------------------------------------------------
    // Think — decide what to do
    // --------------------------------------------------------------------

    pub fn think(&mut self, game: &Game) {
        self.actions.clear();

        // Try a few troll configs and pick the best profitable one
        let train_configs = [(1, 1, 1, 0), (2, 1, 1, 0), (1, 2, 1, 0), (1, 1, 2, 0), (2, 2, 1, 0)];
        let best_train = train_configs.iter()
            .map(|&(ms, cc, hp, cp)| (ms, cc, hp, cp, self.eval_training(game, ms, cc, hp, cp)))
            .filter(|(_, _, _, _, score)| *score > 0)
            .max_by_key(|(_, _, _, _, score)| *score);

        if let Some((ms, cc, hp, cp, _)) = best_train {
            self.actions.push(Action::Train(ms, cc, hp, cp));
        }

        let all_actions = game.actions_for(self.side);
        let mut claimed: HashSet<Position> = HashSet::new();

        // Check planing actions
        let mut filtered: HashMap<i32, Vec<Action>> = HashMap::new();
        for troll in game.trolls_for(self.side) {
            let Some(actions) = all_actions.get(&troll.id) else {
                continue;
            };

            let kept: Vec<Action> = actions
                .iter()
                .filter(|a| match a {
                    Action::Plant(_, typ) => self.eval_planting(game, troll, *typ) > 0,
                    _ => true,
                })
                .cloned()
                .collect();

            filtered.insert(troll.id, kept);
        }
        let all_actions = filtered;

        // Pre-claim positions of trolls that won't move (drop/harvest/plant/wait)
        for troll in game.trolls_for(self.side) {
            let actions = match all_actions.get(&troll.id) {
                Some(a) => a,
                None => continue,
            };
            let chosen = actions
                .iter()
                .find(|a|  matches!(a, Action::Plant(_, _)))
                .or_else(|| actions.iter().find(|a | matches!(a, Action::Drop(_))))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::Harvest(_))));

            if chosen.is_some() {
                claimed.insert(troll.position);
            }
        }

        // Now assign actions, resolving move conflicts
        for troll in game.trolls_for(self.side) {
            let actions = match all_actions.get(&troll.id) {
                Some(a) => a,
                None => continue,
            };

            // Try priority: drop > harvest > plant
            let non_move = actions
                .iter()
                .find(|a|  matches!(a, Action::Plant(_, _)))
                .or_else(|| actions.iter().find(|a | matches!(a, Action::Drop(_))))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::Harvest(_))));

            if let Some(action) = non_move {
                self.actions.push(action.clone());
                continue;
            }

            // Try moves: pick first unclaimed destination
            let move_action = actions.iter().find(|a| {
                if let Action::Move(_, pos) = a {
                    !claimed.contains(pos)
                } else {
                    false
                }
            });

            if let Some(action) = move_action {
                if let Action::Move(_, pos) = action {
                    claimed.insert(*pos);
                }
                self.actions.push(action.clone());
            } else {
                claimed.insert(troll.position);
            }
        }
    }

    // --------------------------------------------------------------------
    // Heuristics
    // --------------------------------------------------------------------

    pub fn best_plant_positions(&self, game: &Game) -> Vec<Position> {
        let shack = game.shack(self.side);
        let mut candidates: Vec<Position> = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        // Seed: shack's direct neighbors (shack itself isn't walkable)
        visited.insert(shack);
        for &c in CARDINALS.iter() {
            let next = shack + c;
            if game.grid.contains(next) && b".ABPL".contains(&game.grid[next]) {
                visited.insert(next);
                queue.push_back(next);
                if game.tree_at(next).is_none() {
                    candidates.push(next);
                }
            }
        }

        if candidates.len() >= 3 {
            return candidates[..3].to_vec();
        }

        // BFS outward from those neighbors
        while let Some(pos) = queue.pop_front() {
            for &c in CARDINALS.iter() {
                let next = pos + c;
                if visited.contains(&next) || !game.grid.contains(next) {
                    continue;
                }
                if !b".ABPL".contains(&game.grid[next]) {
                    continue;
                }
                visited.insert(next);
                queue.push_back(next);
                if game.tree_at(next).is_none() {
                    candidates.push(next);
                    if candidates.len() >= 3 {
                        return candidates;
                    }
                }
            }
        }

        candidates
    }

    pub fn eval_planting(&self, game: &Game, troll: &Troll, typ: TreeType) -> i32 {
        let remaining = Game::MAX_TURNS.saturating_sub(game.turn as i32);
        if remaining <= 0 {
            return 0;
        }

        if game.tree_at(troll.position).is_some() {
            return 0;
        }

        // Only plant on one of the 3 best spots
        let spots = self.best_plant_positions(game);
        if !spots.contains(&troll.position) {
            return 0;
        }

        let shack = game.shack(self.side);
        let dist = troll.position.manhattan(&shack) as i32;

        let cooldown = Tree::initial_cooldown(typ);
        let grow_turns = 3 * cooldown;
        let turns_until_first_fruit = grow_turns + cooldown;

        if turns_until_first_fruit >= remaining {
            return 0;
        }

        let producing_turns = remaining - grow_turns;
        let max_fruits = producing_turns / cooldown;

        let cycle_cost = dist * 2 + 1;
        let deliverable = ((remaining - turns_until_first_fruit) / cycle_cost).min(max_fruits);

        if deliverable <= 1 {
            return 0;
        }

        deliverable - 1 + (10 - cooldown) - dist
    }

    pub fn eval_training(&self, game: &Game, ms: i32, cc: i32, hp: i32, cp: i32) -> i32 {
        let remaining = Game::MAX_TURNS - game.turn as i32;
        if remaining <= 0 {
            return -1;
        }

        if !game.can_train(self.side, ms, cc, hp, cp) {
            return -1;
        }

        let cost = game.train_cost(self.side, ms, cc, hp, cp);
        let total_cost = cost.plum + cost.lemon + cost.apple + cost.banana;

        let shack = game.shack(self.side);

        // Find the nearest tree with fruit (or that will produce fruit)
        let avg_tree_dist = game.trees.iter()
            .map(|t| t.position.manhattan(&shack) as i32)
            .min()
            .unwrap_or(5);

        // Troll spawns at shack. One harvest cycle:
        //   walk to tree + harvest (1 turn) + walk back + drop (1 turn)
        let round_trip = 2 * avg_tree_dist + 2;

        // First turn is spent spawning — troll can't act until next turn
        let usable_turns = remaining - 1;

        if usable_turns <= round_trip {
            return -1;
        }

        // How many full delivery cycles?
        let cycles = usable_turns / round_trip;

        // Each cycle delivers min(carry_capacity, harvest_power, fruits_available)
        // Assume trees have fruit most of the time — use min(cc, hp) as throughput
        let per_trip = cc.min(hp);
        let total_delivered = cycles * per_trip;

        // Net value: what the troll brings home minus what we spent to make it
        total_delivered - total_cost
    }

    // --------------------------------------------------------------------
    // Simulation
    // --------------------------------------------------------------------

    pub fn simulate(&mut self, game: &Game) {
        // For now we pass empty opponent actions — we don't know them
        self.predicted = Some(game.snapshot(&self.actions, &[]));
    }

    pub fn compare(&self, game: &Game) {
        if let Some(snapshot) = &self.predicted {
            game.compare(snapshot);
        }
    }
}
