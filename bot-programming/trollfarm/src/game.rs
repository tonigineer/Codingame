use std::collections::HashMap;
use std::io::{self, BufRead};

use itertools::Itertools;

use crate::entities::{Inventory, Resource, ResourceType, Tree, TreeType, Troll};
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

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Move(i32, Position),
    Harvest(i32),
    Plant(i32, TreeType),
    Chop(i32),
    Mine(i32),
    Pick(i32, TreeType),
    Drop(i32),
    Train(i32, i32, i32, i32), // moveSpeed, carryCap, harvestPow, chopPow
    Wait(i32),
}

impl Action {
    #[must_use]
    pub fn troll_id(&self) -> Option<i32> {
        match self {
            Action::Move(id, _)
            | Action::Harvest(id)
            | Action::Plant(id, _)
            | Action::Chop(id)
            | Action::Mine(id)
            | Action::Pick(id, _)
            | Action::Drop(id)
            | Action::Wait(id) => Some(*id),
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
            Action::Chop(id) => write!(f, "CHOP {id}"),
            Action::Mine(id) => write!(f, "MINE {id}"),
            Action::Pick(id, typ) => write!(f, "PICK {id} {typ}"),
            Action::Drop(id) => write!(f, "DROP {id}"),
            Action::Train(ms, cc, hp, cp) => write!(f, "TRAIN {ms} {cc} {hp} {cp}"),
            Action::Wait(_) => write!(f, ""),
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
    pub mines: Vec<Position>,
    pub inventories: [Inventory; 2],
    pub trees: Vec<Tree>,
    pub trolls: Vec<Troll>,
    next_troll_id: i32,
}

impl Game {
    pub const MAX_TURNS: i32 = 300;

    // --------------------------------------------------------------------
    // IO
    // --------------------------------------------------------------------

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let stdin = io::stdin();
        Self::from_reader(&mut stdin.lock())
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn update(&mut self) {
        let stdin = io::stdin();
        self.update_from_reader(&mut stdin.lock());
    }

    /// Create a Game from a combined startup + first-turn input string.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn create_mock(input: &str) -> Self {
        let mut cursor = io::Cursor::new(input);
        let mut game = Self::from_reader(&mut cursor);
        game.update_from_reader(&mut cursor);
        game
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    fn from_reader(reader: &mut impl BufRead) -> Self {
        let mut next = || {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            line.trim_end().to_string()
        };

        let (width, height) = next()
            .split_once(' ')
            .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()))
            .unwrap();

        let rows: Vec<String> = (0..height).map(|_| next()).collect();
        let grid = Grid::from(rows.join("\n"));

        let my_shack = grid.search(b'0').unwrap();
        let opp_shack = grid.search(b'1').unwrap();

        let mines = (0..height)
            .cartesian_product(0..width)
            .filter_map(|(y, x)| {
                let pos = Position::new(x as i32, y as i32);
                if grid.contains(pos) && grid[pos] == b'+' {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect();

        Self {
            turn: 0,
            width,
            height,
            grid,
            shacks: [my_shack, opp_shack],
            mines,
            inventories: [Inventory::new(), Inventory::new()],
            trees: Vec::new(),
            trolls: Vec::new(),
            next_troll_id: 100,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    fn update_from_reader(&mut self, reader: &mut impl BufRead) {
        self.turn += 1;

        let mut next = || {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            line.trim_end().to_string()
        };

        self.inventories[0] = Inventory::parse(&next());
        self.inventories[1] = Inventory::parse(&next());

        let tree_count: usize = next().trim().parse().unwrap();
        self.trees = (0..tree_count).map(|_| Tree::parse(&next())).collect();

        for tree in &self.trees {
            self.grid[tree.position] = tree.typ.to_byte();
        }

        let troll_count: usize = next().trim().parse().unwrap();
        self.trolls = (0..troll_count).map(|_| Troll::parse(&next())).collect();

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

    pub fn turns_remaining(&self) -> i32 {
        Game::MAX_TURNS - self.turn as i32
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
        i32::try_from(self.trolls.iter().filter(|t| t.side == side).count()).unwrap()
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
        self.inventory(side).score()
    }

    #[must_use]
    pub fn tree_at(&self, pos: Position) -> Option<&Tree> {
        self.trees.iter().find(|t| t.position == pos)
    }

    /// Check if a position is adjacent to water
    #[must_use]
    pub fn is_near_water(&self, pos: Position) -> bool {
        CARDINALS.iter().any(|&c| {
            let next = pos + c;
            self.grid.contains(next) && self.grid[next] == b'~'
        })
    }

    /// Check if a troll is adjacent to an IRON cell
    #[must_use]
    pub fn is_adjacent_to_iron(&self, troll: &Troll) -> bool {
        CARDINALS.iter().any(|&c| {
            let next = troll.position + c;
            self.grid.contains(next) && self.grid[next] == b'+'
        })
    }

    #[must_use]
    pub fn is_adjacent_to_shack(&self, troll: &Troll) -> bool {
        let shack = self.shack(troll.side);
        troll.position.manhattan(&shack) == 1
    }

    // --------------------------------------------------------------------
    // Training cost: PLUM for speed, LEMON for carry, APPLE for harvest, IRON for chop
    // --------------------------------------------------------------------

    #[must_use]
    pub fn train_cost(&self, side: Side, ms: i32, cc: i32, hp: i32, cp: i32) -> TrainCost {
        let n = self.troll_count(side);
        TrainCost {
            plum: n + ms * ms,
            lemon: n + cc * cc,
            apple: n + hp * hp,
            iron: n + cp * cp,
        }
    }

    #[must_use]
    pub fn can_train(&self, side: Side, ms: i32, cc: i32, hp: i32, cp: i32) -> bool {
        let cost = self.train_cost(side, ms, cc, hp, cp);
        let inv = self.inventory(side);
        inv.plum.amount >= cost.plum
            && inv.lemon.amount >= cost.lemon
            && inv.apple.amount >= cost.apple
            && inv.iron.amount >= cost.iron
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

                if self.would_drop(troll).is_some() {
                    actions.push(Action::Drop(troll.id));
                }
                if self.would_harvest(troll).is_some() {
                    actions.push(Action::Harvest(troll.id));
                }
                // Chop: tree on same cell with health > 0
                if let Some(tree) = self.tree_at(troll.position) {
                    if tree.health > 0 && troll.chop_power > 0 {
                        actions.push(Action::Chop(troll.id));
                    }
                }
                // Mine: adjacent to iron, has chop_power and free capacity
                if self.is_adjacent_to_iron(troll)
                    && troll.chop_power > 0
                    && troll.free_capacity() > 0
                {
                    actions.push(Action::Mine(troll.id));
                }
                // Plant
                if self.tree_at(troll.position).is_none() {
                    for typ in &[
                        TreeType::Plum,
                        TreeType::Lemon,
                        TreeType::Apple,
                        TreeType::Banana,
                    ] {
                        if troll.carries(typ) > 0 {
                            actions.push(Action::Plant(troll.id, *typ));
                        }
                    }
                }
                // Pick
                if self.is_adjacent_to_shack(troll) && troll.free_capacity() > 0 {
                    let inv = self.inventory(side);
                    for typ in &[
                        TreeType::Plum,
                        TreeType::Lemon,
                        TreeType::Apple,
                        TreeType::Banana,
                    ] {
                        if inv.get_by_tree(typ) > 0 {
                            actions.push(Action::Pick(troll.id, *typ));
                        }
                    }
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
    pub fn would_drop(&self, troll: &Troll) -> Option<bool> {
        (self.is_adjacent_to_shack(troll) && troll.has_cargo()).then_some(true)
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
    // Simulation — turn order: Move → Harvest → Plant → Chop → Pick →
    //                          Train → Drop → Mine → Grow
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
        self.apply_chops(&all);
        let tick_count = trees_before_plant.min(self.trees.len());
        self.apply_picks(&all);
        self.apply_trains(&all);
        self.apply_drops(&all);
        self.apply_mines(&all);
        self.tick_trees(tick_count);
    }

    fn troll_mut(&mut self, id: i32) -> Option<&mut Troll> {
        self.trolls.iter_mut().find(|t| t.id == id)
    }

    fn troll(&self, id: i32) -> Option<&Troll> {
        self.trolls.iter().find(|t| t.id == id)
    }

    fn apply_moves(&mut self, actions: &[Action]) {
        // Phase 1: resolve all intended destinations
        let mut moves: Vec<(i32, Side, Position)> = Vec::new();
        for action in actions {
            if let Action::Move(id, target) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                let dest = self.resolve_move(troll.position, *target, troll.movement_speed);
                moves.push((*id, troll.side, dest));
            }
        }

        // Phase 2: check for same-team collisions against final positions
        let mut final_positions: HashMap<(Side, Position), i32> = HashMap::new();

        // First, place all non-moving trolls
        let moving_ids: std::collections::HashSet<i32> =
            moves.iter().map(|(id, _, _)| *id).collect();
        for troll in &self.trolls {
            if !moving_ids.contains(&troll.id) {
                final_positions.insert((troll.side, troll.position), troll.id);
            }
        }

        // Then try to place moving trolls; on conflict, keep original position
        let mut resolved: Vec<(i32, Position)> = Vec::new();
        for (id, side, dest) in &moves {
            if let Some(&occupant) = final_positions.get(&(*side, *dest)) {
                if occupant != *id {
                    // Conflict — stay put
                    let original = self.troll(*id).unwrap().position;
                    final_positions.insert((*side, original), *id);
                    resolved.push((*id, original));
                    continue;
                }
            }
            final_positions.insert((*side, *dest), *id);
            resolved.push((*id, *dest));
        }

        // Phase 3: apply all moves at once
        for (id, dest) in &resolved {
            if let Some(troll) = self.troll_mut(*id) {
                troll.position = *dest;
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
                    troll.add_carried(ResourceType::from_tree(&tree_typ), *amount);
                }
            }
        }
    }

    fn apply_plants(&mut self, actions: &[Action]) {
        let mut by_pos: HashMap<Position, Vec<(i32, TreeType)>> = HashMap::new();

        for action in actions {
            if let Action::Plant(id, typ) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                if troll.carries(typ) > 0 && self.tree_at(troll.position).is_none() {
                    by_pos.entry(troll.position).or_default().push((*id, *typ));
                }
            }
        }

        for (pos, planters) in &by_pos {
            let first_typ = planters[0].1;
            let all_same = planters.iter().all(|(_, typ)| *typ == first_typ);

            if !all_same {
                continue;
            }

            let near_water = self.is_near_water(*pos);
            let initial_cd = if near_water {
                Tree::initial_cooldown_water(first_typ)
            } else {
                Tree::initial_cooldown(first_typ)
            };

            self.trees.push(Tree {
                typ: first_typ,
                position: *pos,
                size: 1,
                health: Tree::max_health(first_typ, 1),
                fruits: 0,
                cooldown: initial_cd,
            });
            self.grid[*pos] = first_typ.to_byte();

            for (troll_id, typ) in planters {
                if let Some(troll) = self.troll_mut(*troll_id) {
                    troll.remove_carried(ResourceType::from_tree(typ), 1);
                }
            }
        }
    }

    fn apply_chops(&mut self, actions: &[Action]) {
        let mut by_tree: HashMap<usize, Vec<i32>> = HashMap::new();

        for action in actions {
            if let Action::Chop(id) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                if troll.chop_power <= 0 {
                    continue;
                }
                let troll_pos = troll.position;
                if let Some(tree_idx) = self.trees.iter().position(|t| t.position == troll_pos) {
                    by_tree.entry(tree_idx).or_default().push(*id);
                }
            }
        }

        let mut trees_to_remove: Vec<usize> = Vec::new();

        for (tree_idx, troll_ids) in &by_tree {
            let total_chop: i32 = troll_ids
                .iter()
                .map(|id| self.trolls.iter().find(|t| t.id == *id).unwrap().chop_power)
                .sum();

            self.trees[*tree_idx].health -= total_chop;

            if self.trees[*tree_idx].health <= 0 {
                let wood = self.trees[*tree_idx].size;
                let tree_pos = self.trees[*tree_idx].position;
                trees_to_remove.push(*tree_idx);

                // Round-robin wood distribution
                let mut remaining = wood;
                let mut taken: HashMap<i32, i32> = HashMap::new();
                let mut active: Vec<i32> = troll_ids.clone();

                while remaining > 0 && !active.is_empty() {
                    let last_wood = remaining == 1 && active.len() > 1;

                    active.retain(|id| {
                        let troll = self.trolls.iter().find(|t| t.id == *id).unwrap();
                        let already = taken.get(id).copied().unwrap_or(0);
                        troll.free_capacity() > already
                    });

                    if active.is_empty() {
                        break;
                    }

                    if last_wood {
                        for id in &active {
                            *taken.entry(*id).or_default() += 1;
                        }
                        remaining = 0;
                    } else {
                        for id in &active {
                            if remaining == 0 {
                                break;
                            }
                            *taken.entry(*id).or_default() += 1;
                            remaining -= 1;
                        }
                    }
                }

                for (troll_id, amount) in &taken {
                    if let Some(troll) = self.troll_mut(*troll_id) {
                        troll.add_carried(ResourceType::Wood, *amount);
                    }
                }

                self.grid[tree_pos] = b'.';
            }
        }

        trees_to_remove.sort_unstable();
        for idx in trees_to_remove.into_iter().rev() {
            self.trees.remove(idx);
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
                if inv.get_by_tree(typ) <= 0 {
                    continue;
                }

                self.inventory_mut(side)
                    .remove(&Resource::from_tree(typ, 1));
                if let Some(troll) = self.troll_mut(*id) {
                    troll.add_carried(ResourceType::from_tree(typ), 1);
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

                if !troll.has_cargo() {
                    continue;
                }

                // Transfer all carried resources (fruits, iron, wood)
                let resources = troll.carried_resources();

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
                for side in &[Side::Me, Side::Opp] {
                    if !self.can_train(*side, *ms, *cc, *hp, *cp) {
                        continue;
                    }

                    let cost = self.train_cost(*side, *ms, *cc, *hp, *cp);
                    let inv = self.inventory_mut(*side);
                    inv.remove(&Resource::new(ResourceType::Plum, cost.plum));
                    inv.remove(&Resource::new(ResourceType::Lemon, cost.lemon));
                    inv.remove(&Resource::new(ResourceType::Apple, cost.apple));
                    inv.remove(&Resource::new(ResourceType::Iron, cost.iron));

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
                        carry_iron: 0,
                        carry_wood: 0,
                    });
                    self.next_troll_id += 1;
                    break;
                }
            }
        }
    }

    fn apply_mines(&mut self, actions: &[Action]) {
        for action in actions {
            if let Action::Mine(id) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                if troll.chop_power <= 0 {
                    continue;
                }
                if !self.is_adjacent_to_iron(troll) {
                    continue;
                }

                let amount = troll.chop_power.min(troll.free_capacity());
                if amount > 0 {
                    if let Some(troll) = self.troll_mut(*id) {
                        troll.add_carried(ResourceType::Iron, amount);
                    }
                }
            }
        }
    }

    fn tick_trees(&mut self, count: usize) {
        for tree in &mut self.trees[..count] {
            let near_water = CARDINALS.iter().any(|&c| {
                let next = tree.position + c;
                // Can't call self methods here, inline the check
                next.x >= 0
                    && next.y >= 0
                    && (next.x as usize) < self.width
                    && (next.y as usize) < self.height
                    && self.grid[next] == b'~'
            });

            if tree.cooldown > 0 {
                tree.cooldown -= 1;
            }
            if tree.cooldown > 0 {
                continue;
            }

            let cd = if near_water {
                tree.cooldown_time_water()
            } else {
                tree.cooldown_time()
            };

            if tree.size < 4 {
                let old_health = Tree::max_health(tree.typ, tree.size);
                tree.size += 1;
                let new_health = Tree::max_health(tree.typ, tree.size);
                // When a damaged tree grows, it gains the health difference
                tree.health += new_health - old_health;
                tree.cooldown = cd;
            } else if tree.fruits < 3 {
                tree.fruits += 1;
                tree.cooldown = cd;
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
    pub iron: i32,
}

// ------------------------------------------------------------------------
// Testing
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_INPUT: &str = "\
18 9
#...~.........~~~~
....~1.....#..~~~~
..#.~......#...~~~
+.~~~.............
...~..........~...
.............~~~.+
~~~...#......~.#..
~~~~..#.....0~....
~~~~.........~...#
10 10 6 9 5 0
10 10 6 9 5 0
12
PLUM 10 1 3 10 0 1
PLUM 7 7 3 10 0 1
LEMON 17 4 2 8 0 3
LEMON 0 4 2 8 0 3
LEMON 9 6 2 8 0 1
LEMON 8 2 2 8 0 1
APPLE 5 2 4 20 1 2
APPLE 12 6 4 20 1 2
BANANA 7 3 3 5 0 1
BANANA 10 5 3 5 0 1
BANANA 13 1 4 6 0 4
BANANA 4 7 4 6 0 4
2
0 1 5 1 1 1 1 1 0 0 0 0 0 0
1 0 12 7 1 1 1 1 0 0 0 0 0 0";

    #[test]
    fn test_create_mock() {
        let game = Game::create_mock(MOCK_INPUT);

        // Grid dimensions
        assert_eq!(game.width, 18);
        assert_eq!(game.height, 9);
        assert_eq!(game.turn, 1);

        // Shack positions: '0' at (12,7), '1' at (5,1)
        assert_eq!(game.shacks[0], Position::new(12, 7)); // me
        assert_eq!(game.shacks[1], Position::new(5, 1));  // opp

        // Mines: '+' at (0,3) and (17,5)
        assert_eq!(game.mines.len(), 2);
        assert!(game.mines.contains(&Position::new(0, 3)));
        assert!(game.mines.contains(&Position::new(17, 5)));

        // Inventories
        let my_inv = game.inventory(Side::Me);
        assert_eq!(my_inv.plum.amount, 10);
        assert_eq!(my_inv.lemon.amount, 10);
        assert_eq!(my_inv.apple.amount, 6);
        assert_eq!(my_inv.banana.amount, 9);
        assert_eq!(my_inv.iron.amount, 5);
        assert_eq!(my_inv.wood.amount, 0);

        let opp_inv = game.inventory(Side::Opp);
        assert_eq!(opp_inv.plum.amount, 10);
        assert_eq!(opp_inv.lemon.amount, 10);
        assert_eq!(opp_inv.apple.amount, 6);
        assert_eq!(opp_inv.banana.amount, 9);
        assert_eq!(opp_inv.iron.amount, 5);
        assert_eq!(opp_inv.wood.amount, 0);

        // Trees
        assert_eq!(game.trees.len(), 12);

        let plums: Vec<&Tree> = game.trees.iter().filter(|t| t.typ == TreeType::Plum).collect();
        assert_eq!(plums.len(), 2);
        assert_eq!(plums[0].position, Position::new(10, 1));
        assert_eq!(plums[0].size, 3);
        assert_eq!(plums[0].health, 10);
        assert_eq!(plums[0].fruits, 0);
        assert_eq!(plums[0].cooldown, 1);

        let apples: Vec<&Tree> = game.trees.iter().filter(|t| t.typ == TreeType::Apple).collect();
        assert_eq!(apples.len(), 2);
        assert_eq!(apples[0].size, 4);
        assert_eq!(apples[0].health, 20);
        assert_eq!(apples[0].fruits, 1);

        let bananas: Vec<&Tree> = game.trees.iter().filter(|t| t.typ == TreeType::Banana).collect();
        assert_eq!(bananas.len(), 4);

        let lemons: Vec<&Tree> = game.trees.iter().filter(|t| t.typ == TreeType::Lemon).collect();
        assert_eq!(lemons.len(), 4);

        // Trolls
        assert_eq!(game.trolls.len(), 2);

        let opp_troll = &game.trolls[0];
        assert_eq!(opp_troll.id, 0);
        assert_eq!(opp_troll.side, Side::Opp);
        assert_eq!(opp_troll.position, Position::new(5, 1));
        assert_eq!(opp_troll.movement_speed, 1);
        assert_eq!(opp_troll.carry_capacity, 1);
        assert_eq!(opp_troll.harvest_power, 1);
        assert_eq!(opp_troll.chop_power, 1);
        assert_eq!(opp_troll.total_carried(), 0);

        let my_troll = &game.trolls[1];
        assert_eq!(my_troll.id, 1);
        assert_eq!(my_troll.side, Side::Me);
        assert_eq!(my_troll.position, Position::new(12, 7));
        assert_eq!(my_troll.movement_speed, 1);
        assert_eq!(my_troll.carry_capacity, 1);
        assert_eq!(my_troll.harvest_power, 1);
        assert_eq!(my_troll.chop_power, 1);
        assert_eq!(my_troll.total_carried(), 0);

        // Grid: tree bytes stamped
        assert_eq!(game.grid[Position::new(10, 1)], b'P');
        assert_eq!(game.grid[Position::new(5, 2)], b'A');
        assert_eq!(game.grid[Position::new(7, 3)], b'B');
        assert_eq!(game.grid[Position::new(9, 6)], b'L');

        // Grid: terrain preserved
        assert_eq!(game.grid[Position::new(0, 0)], b'#');
        assert_eq!(game.grid[Position::new(4, 0)], b'~');
        assert_eq!(game.grid[Position::new(0, 3)], b'+');
        assert_eq!(game.grid[Position::new(1, 0)], b'.');
    }
}
