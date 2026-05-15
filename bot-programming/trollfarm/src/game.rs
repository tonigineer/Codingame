use std::collections::HashMap;
use std::io::{self, BufRead};

use crate::position::{Position, CARDINALS};
use crate::grid::Grid;
use crate::entities::{Inventory, Resource, Tree, Troll};

// ------------------------------------------------------------------------
// Game — owns all state
// ------------------------------------------------------------------------

#[derive(Clone)]
pub struct Game {
    pub turn: u16,
    pub width: usize,
    pub height: usize,
    pub grid: Grid<u8>,
    pub shacks: [Position; 2],       // [Me, Opp]
    pub inventories: [Inventory; 2], // [Me, Opp]
    pub trees: Vec<Tree>,
    pub trolls: Vec<Troll>,
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

    // --------------------------------------------------------------------
    // Action enumeration — what can a troll do?
    // --------------------------------------------------------------------

    #[must_use]
    pub fn actions_for(&self, side: Side) -> HashMap<i32, Vec<Action>> {
        self.trolls
            .iter()
            .filter(|t| t.side == side)
            .map(|troll| {
                let mut actions = Vec::new();
                if self.would_drop(troll).is_some() {
                    actions.push(Action::Drop(troll.id));
                }
                if self.would_harvest(troll).is_some() {
                    actions.push(Action::Harvest(troll.id));
                }
                if let Some(moves) = self.reachable_positions(troll) {
                    for pos in moves {
                        actions.push(Action::Move(troll.id, pos));
                    }
                }
                actions.push(Action::Wait(troll.id));
                (troll.id, actions)
            })
            .collect()
    }

    // --------------------------------------------------------------------
    // Troll queries (moved from Troll impl — they need Game context)
    // --------------------------------------------------------------------

    #[must_use]
    pub fn tree_at(&self, pos: Position) -> Option<&Tree> {
        self.trees.iter().find(|t| t.position == pos)
    }

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
        let shack = self.shack(troll.side);
        (troll.position.manhattan(&shack) == 1 && troll.total_carried() > 0)
            .then(|| troll.carried_resources())
    }

    #[must_use]
    pub fn reachable_positions(&self, troll: &Troll) -> Option<Vec<Position>> {
        let mut moves: Vec<_> = CARDINALS
            .iter()
            .map(|&c| troll.position + c)
            .filter(|p| self.grid.contains(*p) && b".ABPL".contains(&self.grid[*p]))
            .collect();

        if moves.is_empty() {
            return None;
        }

        // Heuristic sort
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
    // Simulation — play(actions_p1, actions_p2)
    // --------------------------------------------------------------------

    pub fn play(&mut self, my_actions: &[Action], opp_actions: &[Action]) {
        let all: Vec<Action> = my_actions
            .iter()
            .chain(opp_actions.iter())
            .cloned()
            .collect();
        self.apply_moves(&all);
        self.apply_harvests(&all);
        self.apply_drops(&all);
        self.tick_trees();
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

    fn tick_trees(&mut self) {
        for tree in &mut self.trees {
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
// Action
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Action {
    Move(i32, Position),
    Harvest(i32),
    Drop(i32),
    Wait(i32),
}

impl Action {
    #[must_use]
    pub fn troll_id(&self) -> i32 {
        match self {
            Action::Move(id, _) | Action::Harvest(id) | Action::Drop(id) | Action::Wait(id) => *id,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Action::Move(id, pos) => write!(f, "MOVE {id} {} {}", pos.x, pos.y),
            Action::Harvest(id) => write!(f, "HARVEST {id}"),
            Action::Drop(id) => write!(f, "DROP {id}"),
            Action::Wait(_) => write!(f, ""),
        }
    }
}

// ------------------------------------------------------------------------
// Side — which player
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
