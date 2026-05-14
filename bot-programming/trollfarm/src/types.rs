use crate::{position::Position};
use crate::{game::GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Me,
    Opp,
}

impl Player {
    fn from_id(player_id: i32) -> Self {
        match player_id {
            0 => Player::Me,
            1 => Player::Opp,
            _ => unimplemented!("PlayerID does not exist."),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Resource {
    Banana(i32),
    Plum(i32),
    Apple(i32),
    Lemon(i32),
}

impl Resource {
    #[must_use]
    pub fn from_tree(typ: &TreeType, amount: i32) -> Self {
        match typ {
            TreeType::Apple => Resource::Apple(amount),
            TreeType::Banana => Resource::Banana(amount),
            TreeType::Lemon => Resource::Lemon(amount),
            TreeType::Plum => Resource::Plum(amount),
        }
    }

    #[must_use]
    pub fn amount(&self) -> i32 {
        match self {
            Resource::Apple(n) | Resource::Banana(n) | Resource::Lemon(n) | Resource::Plum(n) => *n,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.amount() == 0
    }
}

use std::ops::AddAssign;

impl AddAssign for Resource {
    fn add_assign(&mut self, other: Self) {
        match (self, other) {
            (Resource::Plum(a), Resource::Plum(b))
            | (Resource::Lemon(a), Resource::Lemon(b))
            | (Resource::Apple(a), Resource::Apple(b))
            | (Resource::Banana(a), Resource::Banana(b)) => *a += b,
            _ => panic!("Cannot add different resource types"),
        }
    }
}

#[derive(Debug)]
pub struct Resources {
    pub plum: Resource,
    pub lemon: Resource,
    pub apple: Resource,
    pub banana: Resource,
    pub iron: i32,
    pub wood: i32,
}

impl Resources {
    #![allow(clippy::missing_panics_doc)]
    #[rustfmt::skip]
    #[must_use]
    pub fn new() -> Self {
        Self {
            plum:   Resource::Plum(0),
            lemon:  Resource::Lemon(0),
            apple:  Resource::Apple(0),
            banana: Resource::Banana(0),
            iron:   0,
            wood:   0,
        }
    }


    #[rustfmt::skip]
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let r: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();

        Self {
            plum:   Resource::Plum(r[0]),
            lemon:  Resource::Lemon(r[1]),
            apple:  Resource::Apple(r[2]),
            banana: Resource::Banana(r[3]),
            iron:   r[4],
            wood:   r[5],
        }
    }

    pub fn add(&mut self, resource: &Resource) {
        match resource {
            Resource::Plum(_) => self.plum += *resource,
            Resource::Lemon(_) => self.lemon += *resource,
            Resource::Apple(_) => self.apple += *resource,
            Resource::Banana(_) => self.banana += *resource,
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TreeType {
    Plum,
    Lemon,
    Apple,
    Banana,
}

impl TreeType {
    #[must_use]
    pub fn to_byte(&self) -> u8 {
        match self {
            TreeType::Apple => b'A',
            TreeType::Banana => b'B',
            TreeType::Lemon => b'L',
            TreeType::Plum => b'P',
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    pub typ: TreeType,
    pub position: Position,
    pub size: i32,
    pub health: i32,
    pub fruits: i32,
    pub cooldown: i32,
}

impl Tree {
    #![allow(clippy::missing_panics_doc)]
    #[rustfmt::skip]
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let d: Vec<&str> = line.split_whitespace().collect();

        let typ = match d[0] {
            "APPLE" => TreeType::Apple,
            "BANANA" => TreeType::Banana,
            "LEMON" => TreeType::Lemon,
            "PLUM" => TreeType::Plum,
            _ => unimplemented!("Unknown tree type"),
        };

        Self {
            typ,
            position:   Position::new(d[1].parse().unwrap(), d[2].parse().unwrap()),
            size:       d[3].parse().unwrap(),
            health:     d[4].parse().unwrap(),
            fruits:     d[5].parse().unwrap(),
            cooldown:   d[6].parse().unwrap(),
        }
    }

    #[must_use]
    pub fn growth_time(&self) -> i32 {
        // Adjust these to match actual game constants
        match self.size {
            1 => 5,
            2 => 7,
            3 => 10,
            _ => 0,
        }
    }

    #[must_use]
    pub fn fruit_time(&self) -> i32 {
        // Adjust to match actual game constants
        5
    }
}

#[derive(Debug)]
pub struct Troll {
    pub id: i32,
    pub player: Player,
    pub position: Position,
    pub movement_speed: i32,
    pub carry_capacity: i32,
    pub harvest_power: i32,
    pub carry_plum: i32,
    pub carry_lemon: i32,
    pub carry_apple: i32,
    pub carry_banana: i32,
}

impl Troll {
    #![allow(clippy::missing_panics_doc)]
    #[rustfmt::skip]
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let d: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();

        Self {
            id:             d[0],
            player:         Player::from_id(d[1]),
            position:       Position::new(d[2], d[3]),
            movement_speed: d[4],
            carry_capacity: d[5],
            harvest_power:  d[6],
            //              d[7] reserved
            carry_plum:     d[8],
            carry_lemon:    d[9],
            carry_apple:    d[10],
            carry_banana:   d[11],
            //              d[12], d[13] reserved
        }
    }

    // ------------------------------------------------------------------------
    // ------ Harvesting
    // ------------------------------------------------------------------------

    fn tree_here<'a>(&self, game_state: &'a GameState) -> Option<&'a Tree> {
        game_state
            .trees
            .iter()
            .find(|t| t.position == self.position)
    }

    #[must_use]
    pub fn would_harvest(&self, game_state: &GameState) -> Option<Resource> {
        let free_capacity = self.free_capacity();
        if free_capacity == 0 {
            return None;
        }

        self.tree_here(game_state).and_then(|tree| {
            let amount = free_capacity.min(self.harvest_power).min(tree.fruits);
            (amount > 0).then(|| Resource::from_tree(&tree.typ, amount))
        })
    }

    // ------------------------------------------------------------------------
    // ------ Dropping into shack
    // ------------------------------------------------------------------------

    #[must_use]
    pub fn carried_resources(&self) -> Vec<Resource> {
        [
            Resource::Apple(self.carry_apple),
            Resource::Banana(self.carry_banana),
            Resource::Lemon(self.carry_lemon),
            Resource::Plum(self.carry_plum),
        ]
        .into_iter()
        .filter(|r| !r.is_empty())
        .collect()
    }

    fn is_adjacent_to_shack(&self, game_state: &GameState) -> bool {
        self.position.manhattan(&game_state.my_shack) == 1
    }

    #[must_use]
    pub fn would_drop(&self, game_state: &GameState) -> Option<Vec<Resource>> {
        (self.is_adjacent_to_shack(game_state) && self.total_carried() > 0)
            .then(|| self.carried_resources())
    }

    // ------------------------------------------------------------------------
    // ------ Moving
    // ------------------------------------------------------------------------

    /// Sort moves by nearest trees with fruits. Just temporary, we don't want to
    /// use heuristic :)
    fn heuristic_sort_moves(&self, game_state: &GameState, moves: &mut [Position]) {
        // Bring back to shack
        if self.free_capacity() == 0 {
            moves.sort_by_key(|pos| game_state.my_shack.manhattan(pos));
            return;
        }

        // Find nearest tree with fruit
        moves.sort_by_key(|pos| {
            game_state
                .trees
                .iter()
                .filter(|t| t.fruits > 0)
                .map(|t| t.position.manhattan(pos))
                .min()
                .unwrap_or(usize::MAX)
        });
    }

    #[must_use]
    pub fn reachable_positions(&self, game_state: &GameState) -> Option<Vec<Position>> {
        let mut moves: Vec<_> = crate::position::CARDINALS
            .iter()
            .map(|&c| self.position + c)
            .filter(|p| game_state.grid.contains(*p) && b".ABPL".contains(&game_state.grid[*p]))
            .collect();
        (!moves.is_empty()).then_some({
            self.heuristic_sort_moves(game_state, &mut moves);
            moves
        })
    }

    // ------------------------------------------------------------------------
    // ------ Helper
    // ------------------------------------------------------------------------
    fn free_capacity(&self) -> i32 {
        self.carry_capacity - self.total_carried()
    }

    #[must_use]
    pub fn total_carried(&self) -> i32 {
        self.carry_plum + self.carry_lemon + self.carry_apple + self.carry_banana
    }

    pub fn add_carried(&mut self, typ: &TreeType, amount: i32) {
        match typ {
            TreeType::Plum => self.carry_plum += amount,
            TreeType::Lemon => self.carry_lemon += amount,
            TreeType::Apple => self.carry_apple += amount,
            TreeType::Banana => self.carry_banana += amount,
        }
    }

    pub fn clear_carried(&mut self) {
        self.carry_plum = 0;
        self.carry_lemon = 0;
        self.carry_apple = 0;
        self.carry_banana = 0;
    }
}
