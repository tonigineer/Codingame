use std::io::{self, BufRead};
use std::fmt;
use crate::grid::Grid;
use crate::position::Position;
use crate::types::{Player, Resources, Tree, Troll};

pub struct GameState {
    pub me: Player,
    pub opp: Player,
    pub width: usize,
    pub height: usize,
    pub grid: Grid<u8>,
    pub my_shack: Position,
    pub opp_shack: Position,
    pub my_resources: Resources,
    pub opp_resources: Resources,
    pub my_score: i32,
    pub opp_score: i32,
    pub trees: Vec<Tree>,
    pub trolls: Vec<Troll>,
}
#[derive(Debug, Clone)]
pub enum Action {
    Move(i32, Position),
    Harvest(i32),
    Drop(i32),
    Wait(i32),
}

impl Action {
    pub fn troll_id(&self) -> i32 {
        match self {
            Action::Move(id, _) | Action::Harvest(id) | Action::Drop(id) | Action::Wait(id) => *id,
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Move(id, pos) => write!(f, "MOVE {} {} {}", id, pos.x, pos.y),
            Action::Harvest(id) => write!(f, "HARVEST {}", id),
            Action::Drop(id) => write!(f, "DROP {}", id),
            Action::Wait(_) => write!(f, ""),
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();

        let (width, height) = next()
            .split_once(" ")
            .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()))
            .unwrap();

        let rows: Vec<String> = (0..height).map(|_| next()).collect();
        let grid = Grid::from(rows.join("\n"));

        let my_shack = grid.search(b'0').unwrap();
        let opp_shack = grid.search(b'1').unwrap();

        Self {
            me: Player::Me,
            opp: Player::Opp,
            width,
            height,
            grid,
            my_shack,
            opp_shack,
            my_resources: Resources::new(),
            opp_resources: Resources::new(),
            my_score: 0,
            opp_score: 0,
            trees: Vec::new(),
            trolls: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines().map(|l| l.unwrap());
        let mut next = || lines.next().unwrap();

        self.my_resources = Resources::parse(&next());
        self.opp_resources = Resources::parse(&next());

        let tree_count: usize = next().trim().parse().unwrap();
        self.trees = (0..tree_count).map(|_| Tree::parse(&next())).collect();

        for tree in &self.trees {
            self.grid[tree.position] = tree.typ.to_byte();
        }

        self.grid.print();

        let troll_count: usize = next().trim().parse().unwrap();
        self.trolls = (0..troll_count).map(|_| Troll::parse(&next())).collect();
    }

    fn winner(&self) -> Option<Player> {
        match self.my_score.cmp(&self.opp_score) {
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(Player::Me),
            std::cmp::Ordering::Less => Some(Player::Opp),
        }
    }

    // ------------------------------------------------------------------------
    // Simulation
    // ------------------------------------------------------------------------
    pub fn apply_actions(&mut self, actions: &[Action]) {
        self.apply_moves(actions);
        self.apply_harvests(actions);
        self.apply_drops(actions);
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

                // Find the closest reachable cell towards target, up to movementSpeed steps
                let dest = self.resolve_move(start, *target, speed);

                // Check no friendly troll already occupies dest
                let player = troll.player;
                let occupied = self
                    .trolls
                    .iter()
                    .any(|t| t.id != *id && t.player == player && t.position == dest);

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

            // Try horizontal first, then vertical, then stop
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
                break; // already at target
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

    fn is_walkable(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && (pos.x as usize) < self.width
            && (pos.y as usize) < self.height
            && self.grid[pos] == b'.'
    }

    fn apply_harvests(&mut self, actions: &[Action]) {
        // Collect all harvest requests: (troll_id, tree_index)
        let mut requests: Vec<(i32, usize)> = Vec::new();

        for action in actions {
            if let Action::Harvest(id) = action {
                let Some(troll) = self.troll(*id) else {
                    continue;
                };
                let troll_pos = troll.position;

                // Find tree on same cell
                if let Some(tree_idx) = self.trees.iter().position(|t| t.position == troll_pos) {
                    requests.push((*id, tree_idx));
                }
            }
        }

        // Group by tree to handle contention
        let mut by_tree: std::collections::HashMap<usize, Vec<i32>> =
            std::collections::HashMap::new();
        for (troll_id, tree_idx) in &requests {
            by_tree.entry(*tree_idx).or_default().push(*troll_id);
        }

        for (tree_idx, troll_ids) in &by_tree {
            let tree = &self.trees[*tree_idx];
            let mut remaining_fruits = tree.fruits;

            // Round-robin: each troll takes one fruit at a time
            let mut taken: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
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
                    // Last fruit gets duplicated for all active harvesters
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

            // Apply harvested amounts
            let tree_typ = self.trees[*tree_idx].typ.clone();
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
                let player = troll.player;
                let pos = troll.position;

                let shack = if player == Player::Me {
                    self.my_shack
                } else {
                    self.opp_shack
                };
                if pos.manhattan(&shack) != 1 {
                    continue;
                }

                let resources = troll.carried_resources();
                if resources.is_empty() {
                    continue;
                };

                let inventory = if player == Player::Me {
                    &mut self.my_resources
                } else {
                    &mut self.opp_resources
                };

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
                continue;
            }
            if tree.size < 4 {
                tree.size += 1;
                tree.cooldown = tree.growth_time();
            } else if tree.fruits < 3 {
                tree.fruits += 1;
                tree.cooldown = tree.fruit_time();
            }
        }
    }
}

pub struct Game {
    pub turn: u8,
    pub game_state: GameState,
}

impl Game {
    #[must_use]
    pub fn new() -> Self {
        Self {
            turn: 0,
            game_state: GameState::new(),
        }
    }

    pub fn update(&mut self) {
        self.turn += 1;
        self.game_state.update();
        eprintln!("Turn {} finished", self.turn);
    }
}
