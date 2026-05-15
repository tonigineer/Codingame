use std::collections::HashMap;
use std::io::{self, BufRead};

use crate::entities::{Inventory, Resource, Tree, TreeType, Troll};
use crate::grid::Grid;
use crate::position::{Position, CARDINALS};

// ------------------------------------------------------------------------
// Side
// ------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Me,
    Opp,
}

impl Side {
    #[must_use]
    pub fn from_id(id: i32) -> Self {
        match id {
            0 => Side::Me,
            1 => Side::Opp,
            _ => unimplemented!("PlayerID does not exist."),
        }
    }

    #[must_use]
    pub fn other(&self) -> Self {
        match self {
            Side::Me => Side::Opp,
            Side::Opp => Side::Me,
        }
    }
}

// ------------------------------------------------------------------------
// Action
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Action {
    Move(i32, Position),
    Harvest(i32),
    Plant(i32, TreeType),
    Pick(i32, TreeType),
    Drop(i32),
    Train(i32, i32, i32, i32), // moveSpeed, carryCap, harvestPow, chopPow
}

impl Action {
    #[must_use]
    pub fn troll_id(&self) -> Option<i32> {
        match self {
            Action::Move(id, _)
            | Action::Harvest(id)
            | Action::Plant(id, _)
            | Action::Pick(id, _)
            | Action::Drop(id) => Some(*id),
            Action::Train(..) => None,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Action::Move(id, pos) => write!(f, "MOVE {id} {} {}", pos.x, pos.y),
            Action::Harvest(id) => write!(f, "HARVEST {id}"),
            Action::Plant(id, typ) => write!(f, "PLANT {id} {typ}"),
            Action::Pick(id, typ) => write!(f, "PICK {id} {typ}"),
            Action::Drop(id) => write!(f, "DROP {id}"),
            Action::Train(ms, cc, hp, cp) => write!(f, "TRAIN {ms} {cc} {hp} {cp}"),
        }
    }
}

// ------------------------------------------------------------------------
// Game
// ------------------------------------------------------------------------

#[derive(Clone)]
pub struct Game {
    pub turn: u16,
    pub width: usize,
    pub height: usize,
    pub grid: Grid<u8>,
    pub shacks: [Position; 2],
    pub inventories: [Inventory; 2],
    pub trees: Vec<Tree>,
    pub trolls: Vec<Troll>,
    next_troll_id: i32,
}

impl Game {
    // --------------------------------------------------------------------
    // IO
    // --------------------------------------------------------------------

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();

        let (width, height) = next()
            .split_once(' ')
            .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()))
            .unwrap();

        let rows: Vec<String> = (0..height).map(|_| next()).collect();
        let grid = Grid::from(rows.join("\n"));

        let my_shack = grid.search(b'0').unwrap();
        let opp_shack = grid.search(b'1').unwrap();

        Self {
            turn: 0,
            width,
            height,
            grid,
            shacks: [my_shack, opp_shack],
            inventories: [Inventory::new(), Inventory::new()],
            trees: Vec::new(),
            trolls: Vec::new(),
            next_troll_id: 100,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn update(&mut self) {
        self.turn += 1;

        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();

        self.inventories[0] = Inventory::parse(&next());
        self.inventories[1] = Inventory::parse(&next());

        let tree_count: usize = next().trim().parse().unwrap();
        self.trees = (0..tree_count).map(|_| Tree::parse(&next())).collect();

        for tree in &self.trees {
            self.grid[tree.position] = tree.typ.to_byte();
        }

        let troll_count: usize = next().trim().parse().unwrap();
        self.trolls = (0..troll_count).map(|_| Troll::parse(&next())).collect();

        // Track highest troll id for simulation
        if let Some(max_id) = self.trolls.iter().map(|t| t.id).max() {
            self.next_troll_id = max_id + 1;
        }

        eprintln!("Turn {}", self.turn);
    }

    pub fn output(actions: &[Action]) {
        let output = actions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(";");

        println!(
            "{}",
            if output.is_empty() {
                "WAIT".into()
            } else {
                output
            }
        );
    }

    // --------------------------------------------------------------------
    // Accessors
    // --------------------------------------------------------------------

    #[must_use]
    pub fn shack(&self, side: Side) -> Position {
        match side {
            Side::Me => self.shacks[0],
            Side::Opp => self.shacks[1],
        }
    }

    #[must_use]
    pub fn inventory(&self, side: Side) -> &Inventory {
        match side {
            Side::Me => &self.inventories[0],
            Side::Opp => &self.inventories[1],
        }
    }

    pub fn inventory_mut(&mut self, side: Side) -> &mut Inventory {
        match side {
            Side::Me => &mut self.inventories[0],
            Side::Opp => &mut self.inventories[1],
        }
    }

    #[must_use]
    pub fn trolls_for(&self, side: Side) -> Vec<&Troll> {
        self.trolls.iter().filter(|t| t.side == side).collect()
    }

    #[must_use]
    pub fn troll_count(&self, side: Side) -> i32 {
        self.trolls.iter().filter(|t| t.side == side).count() as i32
    }

    #[must_use]
    pub fn winner(&self) -> Option<Side> {
        let my_score = self.score(Side::Me);
        let opp_score = self.score(Side::Opp);
        match my_score.cmp(&opp_score) {
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(Side::Me),
            std::cmp::Ordering::Less => Some(Side::Opp),
        }
    }

    fn score(&self, side: Side) -> i32 {
        let inv = self.inventory(side);
        inv.plum.amount() + inv.lemon.amount() + inv.apple.amount() + inv.banana.amount()
    }

    #[must_use]
    pub fn tree_at(&self, pos: Position) -> Option<&Tree> {
        self.trees.iter().find(|t| t.position == pos)
    }

    // --------------------------------------------------------------------
    // Training cost calculation
    // --------------------------------------------------------------------

    /// Cost for a single attribute = num_existing_trolls + attribute^2
    #[must_use]
    pub fn train_cost(&self, side: Side, ms: i32, cc: i32, hp: i32, _cp: i32) -> TrainCost {
        let n = self.troll_count(side);
        TrainCost {
            plum: n + ms * ms,
            lemon: n + cc * cc,
            apple: n + hp * hp,
            banana: 0, // reserved (chopPower)
        }
    }

    #[must_use]
    pub fn can_train(&self, side: Side, ms: i32, cc: i32, hp: i32, cp: i32) -> bool {
        let cost = self.train_cost(side, ms, cc, hp, cp);
        let inv = self.inventory(side);
        inv.plum.amount() >= cost.plum
            && inv.lemon.amount() >= cost.lemon
            && inv.apple.amount() >= cost.apple
            && inv.banana.amount() >= cost.banana
    }

    // --------------------------------------------------------------------
    // Action enumeration
    // --------------------------------------------------------------------

    #[must_use]
    pub fn actions_for(&self, side: Side) -> HashMap<i32, Vec<Action>> {
        self.trolls
            .iter()
            .filter(|t| t.side == side)
            .map(|troll| {
                let mut actions = Vec::new();

                // Drop:
                if self.would_drop(troll).is_some() {
                    actions.push(Action::Drop(troll.id));
                }

                // Harvest:
                if self.would_harvest(troll).is_some() {
                    actions.push(Action::Harvest(troll.id));
                }

                // Plant: can plant any type the troll is carrying, if no tree here
                if self.tree_at(troll.position).is_none() {
                    for typ in &[TreeType::Plum, TreeType::Lemon, TreeType::Apple, TreeType::Banana] {
                        if troll.carries(typ) > 0 {
                            actions.push(Action::Plant(troll.id, *typ));
                        }
                    }
                }

                // Move:
                if let Some(moves) = self.reachable_positions(troll) {
                    for pos in moves {
                        actions.push(Action::Move(troll.id, pos));
                    }
                }

                // Pick: can pick from shack if adjacent and has free capacity
                if self.is_adjacent_to_shack(troll) && troll.free_capacity() > 0 {
                    let inv = self.inventory(side);
                    for typ in &[TreeType::Plum, TreeType::Lemon, TreeType::Apple, TreeType::Banana] {
                        if inv.get(typ) > 0 {
                            actions.push(Action::Pick(troll.id, *typ));
                        }
                    }
                }

                (troll.id, actions)
            })
            .collect()
    }

    // --------------------------------------------------------------------
    // Troll queries
    // --------------------------------------------------------------------

    #[must_use]
    pub fn would_harvest(&self, troll: &Troll) -> Option<Resource> {
        let free = troll.free_capacity();
        if free == 0 {
            return None;
        }
        self.tree_at(troll.position).and_then(|tree| {
            let amount = free.min(troll.harvest_power).min(tree.fruits);
            (amount > 0).then(|| Resource::from_tree(&tree.typ, amount))
        })
    }

    #[must_use]
    pub fn would_drop(&self, troll: &Troll) -> Option<Vec<Resource>> {
        (self.is_adjacent_to_shack(troll) && troll.total_carried() > 0)
            .then(|| troll.carried_resources())
    }

    #[must_use]
    pub fn is_adjacent_to_shack(&self, troll: &Troll) -> bool {
        let shack = self.shack(troll.side);
        troll.position.manhattan(&shack) == 1
    }

    #[must_use]
    pub fn reachable_positions(&self, troll: &Troll) -> Option<Vec<Position>> {
        // TODO: make it `global` for Player once every turn
        let troll_positions = self.trolls.iter().map(|t| t.position).collect::<Vec<Position>>();

        let mut moves: Vec<_> = CARDINALS
            .iter()
            .map(|&c| troll.position + c)
            .filter(|p| self.grid.contains(*p)
                && b".ABPL".contains(&self.grid[*p])
                && !troll_positions.contains(p)
            )
            .collect();

        if moves.is_empty() {
            return None;
        }

        if troll.free_capacity() == 0 {
            let shack = self.shack(troll.side);
            moves.sort_by_key(|pos| shack.manhattan(pos));
        } else {
            moves.sort_by_key(|pos| {
                self.trees
                    .iter()
                    .filter(|t| t.fruits > 0)
                    .map(|t| t.position.manhattan(pos))
                    .min()
                    .unwrap_or(usize::MAX)
            });
        }

        Some(moves)
    }

    // --------------------------------------------------------------------
    // Simulation
    // --------------------------------------------------------------------

    pub fn play(&mut self, my_actions: &[Action], opp_actions: &[Action]) {
        let all: Vec<Action> = my_actions
            .iter()
            .chain(opp_actions.iter())
            .cloned()
            .collect();
        self.apply_moves(&all);
        self.apply_harvests(&all);
        let trees_before_plant = self.trees.len();
        self.apply_plants(&all);
        self.apply_picks(&all);
        self.apply_drops(&all);
        self.apply_trains(&all);
        self.tick_trees(trees_before_plant);
    }

    fn troll_mut(&mut self, id: i32) -> Option<&mut Troll> {
        self.trolls.iter_mut().find(|t| t.id == id)
    }

    fn troll(&self, id: i32) -> Option<&Troll> {
        self.trolls.iter().find(|t| t.id == id)
    }

    fn apply_moves(&mut self, actions: &[Action]) {
        for action in actions {
            if let Action::Move(id, target) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                let speed = troll.movement_speed;
                let start = troll.position;
                let dest = self.resolve_move(start, *target, speed);

                let side = troll.side;
                let occupied = self
                    .trolls
                    .iter()
                    .any(|t| t.id != *id && t.side == side && t.position == dest);

                #[allow(clippy::collapsible_if)]
                if !occupied {
                    if let Some(troll) = self.troll_mut(*id) {
                        troll.position = dest;
                    }
                }
            }
        }
    }

    fn resolve_move(&self, from: Position, target: Position, speed: i32) -> Position {
        let mut current = from;
        for _ in 0..speed {
            let dx = (target.x - current.x).signum();
            let dy = (target.y - current.y).signum();

            let candidates = if dx != 0 && dy != 0 {
                vec![
                    Position::new(current.x + dx, current.y),
                    Position::new(current.x, current.y + dy),
                ]
            } else if dx != 0 {
                vec![Position::new(current.x + dx, current.y)]
            } else if dy != 0 {
                vec![Position::new(current.x, current.y + dy)]
            } else {
                break;
            };

            let mut moved = false;
            for candidate in candidates {
                if self.is_walkable(candidate) {
                    current = candidate;
                    moved = true;
                    break;
                }
            }
            if !moved {
                break;
            }
        }
        current
    }

    #[allow(clippy::cast_sign_loss)]
    fn is_walkable(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && (pos.x as usize) < self.width
            && (pos.y as usize) < self.height
            && b".ABPL".contains(&self.grid[pos])
    }

    fn apply_harvests(&mut self, actions: &[Action]) {
        let mut requests: Vec<(i32, usize)> = Vec::new();

        for action in actions {
            if let Action::Harvest(id) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                let troll_pos = troll.position;
                if let Some(tree_idx) = self.trees.iter().position(|t| t.position == troll_pos) {
                    requests.push((*id, tree_idx));
                }
            }
        }

        let mut by_tree: HashMap<usize, Vec<i32>> = HashMap::new();
        for (troll_id, tree_idx) in &requests {
            by_tree.entry(*tree_idx).or_default().push(*troll_id);
        }

        for (tree_idx, troll_ids) in &by_tree {
            let tree = &self.trees[*tree_idx];
            let mut remaining_fruits = tree.fruits;
            let mut taken: HashMap<i32, i32> = HashMap::new();
            let mut active: Vec<i32> = troll_ids.clone();

            while remaining_fruits > 0 && !active.is_empty() {
                let last_fruit = remaining_fruits == 1 && active.len() > 1;

                active.retain(|id| {
                    let troll = self.trolls.iter().find(|t| t.id == *id).unwrap();
                    let already = taken.get(id).copied().unwrap_or(0);
                    let free_capacity = troll.carry_capacity - troll.total_carried();
                    already < troll.harvest_power && free_capacity > already
                });

                if active.is_empty() {
                    break;
                }

                if last_fruit {
                    for id in &active {
                        *taken.entry(*id).or_default() += 1;
                    }
                    remaining_fruits = 0;
                } else {
                    for id in &active {
                        if remaining_fruits == 0 {
                            break;
                        }
                        *taken.entry(*id).or_default() += 1;
                        remaining_fruits -= 1;
                    }
                }
            }

            let tree_typ = self.trees[*tree_idx].typ;
            self.trees[*tree_idx].fruits = remaining_fruits;

            for (troll_id, amount) in &taken {
                if let Some(troll) = self.troll_mut(*troll_id) {
                    troll.add_carried(&tree_typ, *amount);
                }
            }
        }
    }

    fn apply_plants(&mut self, actions: &[Action]) {
        // Group plant requests by position to handle conflicts
        let mut by_pos: HashMap<Position, Vec<(i32, TreeType)>> = HashMap::new();

        for action in actions {
            if let Action::Plant(id, typ) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                // Must carry at least 1 of that fruit and no tree already here
                if troll.carries(typ) > 0 && self.tree_at(troll.position).is_none() {
                    by_pos
                        .entry(troll.position)
                        .or_default()
                        .push((*id, *typ));
                }
            }
        }

        for (pos, planters) in &by_pos {
            // Check if all planters on this cell plant the same type
            let first_typ = planters[0].1;
            let all_same = planters.iter().all(|(_, typ)| *typ == first_typ);

            if !all_same {
                // Different types: nothing happens, but all lose their seed
                // Actually per rules: "nothing will happen" — so no seed lost
                continue;
            }

            // All same type: plant the tree, each planter loses 1 seed
            self.trees.push(Tree {
                typ: first_typ,
                position: *pos,
                size: 1,
                health: 10,
                fruits: 0,
                cooldown: Tree::initial_cooldown(first_typ),
            });
            self.grid[*pos] = first_typ.to_byte();

            for (troll_id, typ) in planters {
                if let Some(troll) = self.troll_mut(*troll_id) {
                    troll.remove_carried(typ, 1);
                }
            }
        }
    }

    fn apply_picks(&mut self, actions: &[Action]) {
        for action in actions {
            if let Action::Pick(id, typ) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                let side = troll.side;

                if !self.is_adjacent_to_shack(troll) {
                    continue;
                }
                if troll.free_capacity() <= 0 {
                    continue;
                }

                let inv = self.inventory(side);
                if inv.get(typ) <= 0 {
                    continue;
                }

                // Remove from inventory, add to troll
                self.inventory_mut(side)
                    .remove(&Resource::from_tree(typ, 1));
                if let Some(troll) = self.troll_mut(*id) {
                    troll.add_carried(typ, 1);
                }
            }
        }
    }

    fn apply_drops(&mut self, actions: &[Action]) {
        for action in actions {
            if let Action::Drop(id) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                let side = troll.side;
                let pos = troll.position;

                let shack = self.shack(side);
                if pos.manhattan(&shack) != 1 {
                    continue;
                }

                let resources = troll.carried_resources();
                if resources.is_empty() {
                    continue;
                }

                let inventory = self.inventory_mut(side);
                for r in &resources {
                    inventory.add(r);
                }

                if let Some(troll) = self.troll_mut(*id) {
                    troll.clear_carried();
                }
            }
        }
    }

    fn apply_trains(&mut self, actions: &[Action]) {
        for action in actions {
            if let Action::Train(ms, cc, hp, cp) = action {
                // Figure out which side issued this action
                // For simulation we need to determine the side — we check both
                for side in &[Side::Me, Side::Opp] {
                    if !self.can_train(*side, *ms, *cc, *hp, *cp) {
                        continue;
                    }

                    let cost = self.train_cost(*side, *ms, *cc, *hp, *cp);
                    let inv = self.inventory_mut(*side);
                    inv.remove(&Resource::Plum(cost.plum));
                    inv.remove(&Resource::Lemon(cost.lemon));
                    inv.remove(&Resource::Apple(cost.apple));
                    inv.remove(&Resource::Banana(cost.banana));

                    let shack = self.shack(*side);
                    self.trolls.push(Troll {
                        id: self.next_troll_id,
                        side: *side,
                        position: shack,
                        movement_speed: *ms,
                        carry_capacity: *cc,
                        harvest_power: *hp,
                        chop_power: *cp,
                        carry_plum: 0,
                        carry_lemon: 0,
                        carry_apple: 0,
                        carry_banana: 0,
                    });
                    self.next_troll_id += 1;
                    break; // Only one side can train per Train action
                }
            }
        }
    }

    fn tick_trees(&mut self, count: usize) {
        for tree in &mut self.trees[..count] {
            if tree.cooldown > 0 {
                tree.cooldown -= 1;
            }
            if tree.cooldown > 0 {
                continue;
            }
            if tree.size < 4 {
                tree.size += 1;
                tree.cooldown = tree.cooldown_time();
            } else if tree.fruits < 3 {
                tree.fruits += 1;
                tree.cooldown = tree.cooldown_time();
            }
        }
    }

}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

// ------------------------------------------------------------------------
// TrainCost
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TrainCost {
    pub plum: i32,
    pub lemon: i32,
    pub apple: i32,
    pub banana: i32,
}
