use itertools::Itertools;
use std::collections::HashMap;
use std::io::{self, BufRead};

use crate::game::entities::{Inventory, Tree, TreeType, Troll};
use crate::utils::{CARDINALS, Grid, Position};

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
    #[allow(dead_code)]
    pub fn other(self) -> Self {
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
#[allow(dead_code)]
pub enum Action {
    Move(i32, Position),
    Harvest(i32),
    Plant(i32, TreeType),
    Chop(i32),
    Mine(i32),
    Pick(i32, TreeType),
    Drop(i32),
    Train(i32, i32, i32, i32), // moveSpeed, carryCap, harvestPow, chopPow
    Wait,
}

impl Action {
    #[must_use]
    #[allow(dead_code)]
    pub fn troll_id(&self) -> Option<i32> {
        match self {
            Action::Move(id, _)
            | Action::Harvest(id)
            | Action::Plant(id, _)
            | Action::Chop(id)
            | Action::Mine(id)
            | Action::Pick(id, _)
            | Action::Drop(id) => Some(*id),
            Action::Train(..) | Action::Wait => None,
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
            Action::Wait => write!(f, "WAIT"),
        }
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
// Game
// ------------------------------------------------------------------------

#[derive(Clone)]
#[allow(dead_code)]
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
    pub shack_dist_map: HashMap<Position, (i32, Position)>,
    pub opp_shack_dist_map: HashMap<Position, (i32, Position)>,
    /// Average walking distance from each shack to the remaining trees,
    /// recomputed every turn (see [`Game::update_tree_proximity`]). Lower means
    /// the trees cluster closer to that shack; comparing the two reveals when the
    /// remaining trees sit mostly on the opponent's side of the map. Both are
    /// `0.0` once no trees remain.
    pub trees_avg_dist_mine: f32,
    pub trees_avg_dist_opp: f32,
    next_troll_id: i32,
}

impl Game {
    #[allow(dead_code)]
    pub const MAX_TURNS: i32 = 300;

    // ====================================================================
    // Live play  —  reads real stdin, writes real stdout. Use these in `main`.
    // For tests, see `create_mock` below.
    // ====================================================================

    /// Builds the initial game by reading the one-time startup block from
    /// **stdin**. Entry point for live play on `CodinGame`.
    ///
    /// For tests, prefer [`Game::create_mock`], which reads an in-memory string
    /// instead of blocking on stdin.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let stdin = io::stdin();
        Self::read_setup(&mut stdin.lock())
    }

    /// Reads one turn of state from **stdin** and applies it in place.
    ///
    /// Call once per loop iteration, before computing actions and emitting them
    /// with [`Game::output`].
    #[allow(clippy::missing_panics_doc)]
    pub fn update(&mut self) {
        let stdin = io::stdin();
        self.read_turn(&mut stdin.lock());
    }

    /// Writes `actions` to **stdout** as a single `;`-separated command line,
    /// emitting `WAIT` when there is nothing to do.
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

    // ====================================================================
    // Testing  —  reads an in-memory string instead of stdin.
    // ====================================================================

    /// Builds a fully-initialized game (startup **and** the first turn) from a
    /// single recorded input string.
    ///
    /// Testing counterpart to [`Game::new`] followed by [`Game::update`]: it
    /// reads from an in-memory cursor, so tests can feed deterministic frames.
    /// Kept `pub` (rather than `#[cfg(test)]`) so doc tests and `tests/`
    /// integration tests can reach it.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // A frame captured from a real match: startup block + one turn block.
    /// let input = "\
    /// 7 5
    /// ......0
    /// ...+...
    /// .......
    /// ...+...
    /// 1......
    /// <inventory 0>
    /// <inventory 1>
    /// 0
    /// 0";
    /// let game = Game::create_mock(input);
    /// assert_eq!(game.turn, 1);
    /// ```
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    #[allow(dead_code)]
    pub fn create_mock(input: &str) -> Self {
        let mut cursor = io::Cursor::new(input);
        let mut game = Self::read_setup(&mut cursor);
        game.read_turn(&mut cursor);
        game
    }

    // ====================================================================
    // Shared parsing core  —  works over any `BufRead`, so the live stdin path
    // and the in-memory mock path go through identical code.
    // ====================================================================

    /// Parses the one-time startup block (map size, grid, shacks, mines) from
    /// any reader. Backs both [`Game::new`] and [`Game::create_mock`].
    #[allow(clippy::missing_panics_doc)]
    fn read_setup(reader: &mut impl BufRead) -> Self {
        let (width, height) = Self::read_line(reader)
            .split_once(' ')
            .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()))
            .unwrap();
        let rows: Vec<String> = (0..height).map(|_| Self::read_line(reader)).collect();
        let grid = Grid::from(rows.join("\n"));
        let my_shack = grid.search(b'0').unwrap();
        let opp_shack = grid.search(b'1').unwrap();
        let mines = (0..height)
            .cartesian_product(0..width)
            .filter_map(|(y, x)| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let pos = Position::new(x as i32, y as i32);
                (grid.contains(pos) && grid[pos] == b'+').then_some(pos)
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
            shack_dist_map: HashMap::new(),
            opp_shack_dist_map: HashMap::new(),
            trees_avg_dist_mine: 0.0,
            trees_avg_dist_opp: 0.0,
            next_troll_id: 100,
        }
    }

    /// Parses one turn block (inventories, trees, trolls) from any reader and
    /// applies it in place. Backs both [`Game::update`] and [`Game::create_mock`].
    #[allow(clippy::missing_panics_doc)]
    fn read_turn(&mut self, reader: &mut impl BufRead) {
        self.turn += 1;

        self.inventories[0] = Inventory::parse(&Self::read_line(reader));
        self.inventories[1] = Inventory::parse(&Self::read_line(reader));

        let tree_count: usize = Self::read_line(reader).trim().parse().unwrap();
        self.trees = (0..tree_count)
            .map(|_| Tree::parse(&Self::read_line(reader)))
            .collect();

        // Manual update of tree positions (remove and add)
        for y in 0..self.height {
            for x in 0..self.width {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let p = Position::new(x as i32, y as i32);
                if matches!(self.grid[p], b'A' | b'P' | b'B' | b'L') {
                    self.grid[p] = b'.';
                }
            }
        }

        for tree in &self.trees {
            self.grid[tree.position] = tree.typ.to_byte();
        }

        // Update trolls
        let troll_count: usize = Self::read_line(reader).trim().parse().unwrap();
        self.trolls = (0..troll_count)
            .map(|_| Troll::parse(&Self::read_line(reader)))
            .collect();
        if let Some(max_id) = self.trolls.iter().map(|t| t.id).max() {
            self.next_troll_id = max_id + 1;
        }

        eprintln!("Turn {}", self.turn);
        // self.grid.print();
    }

    /// Reads one line from `reader` and strips the trailing newline.
    ///
    /// Panics on I/O error: during a match the input stream is assumed
    /// well-formed, so a read failure is unrecoverable.
    fn read_line(reader: &mut impl BufRead) -> String {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("failed to read input line");
        line.trim_end().to_string()
    }

    // --------------------------------------------------------------------
    // Accessors
    // --------------------------------------------------------------------
    #[allow(dead_code)]
    #[must_use]
    pub fn turns_remaining(&self) -> i32 {
        Game::MAX_TURNS - i32::from(self.turn)
    }

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

    #[allow(dead_code)]
    pub fn inventory_mut(&mut self, side: Side) -> &mut Inventory {
        match side {
            Side::Me => &mut self.inventories[0],
            Side::Opp => &mut self.inventories[1],
        }
    }

    #[must_use]
    pub fn trolls(&self, side: Side) -> Vec<&Troll> {
        self.trolls.iter().filter(|t| t.side == side).collect()
    }
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn troll_count(&self, side: Side) -> i32 {
        i32::try_from(self.trolls.iter().filter(|t| t.side == side).count()).unwrap()
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn tree_at(&self, pos: Position) -> Option<&Tree> {
        self.trees.iter().find(|t| t.position == pos)
    }

    /// Returns an iterator over `Position` where iron mines are located.
    pub fn mines(&self) -> impl Iterator<Item = Position> + '_ {
        (0..self.grid.width())
            .cartesian_product(0..self.grid.height())
            .map(|(c, r)| Position::new(c, r))
            .filter(|pos| self.grid[*pos] == b'+')
    }

    /// Check if a position is adjacent to water
    #[must_use]
    #[allow(dead_code)]
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
        troll.position.manhattan(shack) == 1
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
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
