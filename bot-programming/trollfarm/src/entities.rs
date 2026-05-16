use crate::game::Side;
use crate::position::Position;
use std::ops::AddAssign;

// ------------------------------------------------------------------------
// Resource / Inventory
// ------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum Resource {
    Plum(i32),
    Lemon(i32),
    Apple(i32),
    Banana(i32),
}

impl Resource {
    #[must_use]
    pub fn from_tree(typ: &TreeType, amount: i32) -> Self {
        match typ {
            TreeType::Plum => Resource::Plum(amount),
            TreeType::Lemon => Resource::Lemon(amount),
            TreeType::Apple => Resource::Apple(amount),
            TreeType::Banana => Resource::Banana(amount),
        }
    }

    #[must_use]
    pub fn amount(&self) -> i32 {
        match self {
            Resource::Plum(n) | Resource::Lemon(n) | Resource::Apple(n) | Resource::Banana(n) => *n,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.amount() == 0
    }

    #[must_use]
    pub fn tree_type(&self) -> TreeType {
        match self {
            Resource::Plum(_) => TreeType::Plum,
            Resource::Lemon(_) => TreeType::Lemon,
            Resource::Apple(_) => TreeType::Apple,
            Resource::Banana(_) => TreeType::Banana,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Inventory {
    pub plum: Resource,
    pub lemon: Resource,
    pub apple: Resource,
    pub banana: Resource,
    pub iron: i32,
    pub wood: i32,
}

impl Inventory {
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

    pub fn remove(&mut self, resource: &Resource) {
        match resource {
            Resource::Plum(n) => self.plum += Resource::Plum(-n),
            Resource::Lemon(n) => self.lemon += Resource::Lemon(-n),
            Resource::Apple(n) => self.apple += Resource::Apple(-n),
            Resource::Banana(n) => self.banana += Resource::Banana(-n),
        }
    }

    #[must_use]
    pub fn get(&self, typ: &TreeType) -> i32 {
        match typ {
            TreeType::Plum => self.plum.amount(),
            TreeType::Lemon => self.lemon.amount(),
            TreeType::Apple => self.apple.amount(),
            TreeType::Banana => self.banana.amount(),
        }
    }

    /// Score: each fruit = 1 point, wood = 4 points, iron = 0
    #[must_use]
    pub fn score(&self) -> i32 {
        self.plum.amount() + self.lemon.amount() + self.apple.amount() + self.banana.amount()
            + self.wood * 4
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

// ------------------------------------------------------------------------
// Tree
// ------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            "PLUM" => TreeType::Plum,
            "LEMON" => TreeType::Lemon,
            "APPLE" => TreeType::Apple,
            "BANANA" => TreeType::Banana,
            _ => unimplemented!("Unknown tree type: {s}"),
        }
    }

    #[must_use]
    pub fn to_str(&self) -> &'static str {
        match self {
            TreeType::Plum => "PLUM",
            TreeType::Lemon => "LEMON",
            TreeType::Apple => "APPLE",
            TreeType::Banana => "BANANA",
        }
    }
}

impl std::fmt::Display for TreeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub typ: TreeType,
    pub position: Position,
    pub size: i32,
    pub health: i32,
    pub fruits: i32,
    pub cooldown: i32,
}

impl Tree {
    #[rustfmt::skip]
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let d: Vec<&str> = line.split_whitespace().collect();
        Self {
            typ:        TreeType::from_str(d[0]),
            position:   Position::new(d[1].parse().unwrap(), d[2].parse().unwrap()),
            size:       d[3].parse().unwrap(),
            health:     d[4].parse().unwrap(),
            fruits:     d[5].parse().unwrap(),
            cooldown:   d[6].parse().unwrap(),
        }
    }

    /// Normal cooldown (not near water)
    #[must_use]
    pub fn cooldown_time(&self) -> i32 {
        match self.typ {
            TreeType::Plum => 8,
            TreeType::Lemon => 8,
            TreeType::Apple => 9,
            TreeType::Banana => 6,
        }
    }

    /// Cooldown when adjacent to water
    #[must_use]
    pub fn cooldown_time_water(&self) -> i32 {
        match self.typ {
            TreeType::Plum => 3,
            TreeType::Lemon => 3,
            TreeType::Apple => 2,
            TreeType::Banana => 4,
        }
    }

    #[must_use]
    pub fn initial_cooldown(typ: TreeType) -> i32 {
        match typ {
            TreeType::Plum => 8,
            TreeType::Lemon => 8,
            TreeType::Apple => 9,
            TreeType::Banana => 6,
        }
    }

    #[must_use]
    pub fn initial_cooldown_water(typ: TreeType) -> i32 {
        match typ {
            TreeType::Plum => 3,
            TreeType::Lemon => 3,
            TreeType::Apple => 2,
            TreeType::Banana => 4,
        }
    }

    /// Health for a given type and size
    #[rustfmt::skip]
    #[must_use]
    pub fn max_health(typ: TreeType, size: i32) -> i32 {
        match (typ, size) {
            (TreeType::Plum,   1) | (TreeType::Lemon, 1) => 6,
            (TreeType::Plum,   2) | (TreeType::Lemon, 2) => 8,
            (TreeType::Plum,   3) | (TreeType::Lemon, 3) => 10,
            (TreeType::Plum,   4) | (TreeType::Lemon, 4) => 12,
            (TreeType::Apple,  1) => 11,
            (TreeType::Apple,  2) => 14,
            (TreeType::Apple,  3) => 17,
            (TreeType::Apple,  4) => 20,
            (TreeType::Banana, 1) => 3,
            (TreeType::Banana, 2) => 4,
            (TreeType::Banana, 3) => 5,
            (TreeType::Banana, 4) => 6,
            _ => 10,
        }
    }
}

// ------------------------------------------------------------------------
// Troll
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Troll {
    pub id: i32,
    pub side: Side,
    pub position: Position,
    pub movement_speed: i32,
    pub carry_capacity: i32,
    pub harvest_power: i32,
    pub chop_power: i32,
    pub carry_plum: i32,
    pub carry_lemon: i32,
    pub carry_apple: i32,
    pub carry_banana: i32,
    pub carry_iron: i32,
    pub carry_wood: i32,
}

impl Troll {
    #[rustfmt::skip]
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let d: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        Self {
            id:             d[0],
            side:           Side::from_id(d[1]),
            position:       Position::new(d[2], d[3]),
            movement_speed: d[4],
            carry_capacity: d[5],
            harvest_power:  d[6],
            chop_power:     d[7],
            carry_plum:     d[8],
            carry_lemon:    d[9],
            carry_apple:    d[10],
            carry_banana:   d[11],
            carry_iron:     d[12],
            carry_wood:     d[13],
        }
    }

    #[must_use]
    pub fn total_carried(&self) -> i32 {
        self.carry_plum + self.carry_lemon + self.carry_apple + self.carry_banana
            + self.carry_iron + self.carry_wood
    }

    #[must_use]
    pub fn free_capacity(&self) -> i32 {
        self.carry_capacity - self.total_carried()
    }

    #[must_use]
    pub fn carried_resources(&self) -> Vec<Resource> {
        [
            Resource::Plum(self.carry_plum),
            Resource::Lemon(self.carry_lemon),
            Resource::Apple(self.carry_apple),
            Resource::Banana(self.carry_banana),
        ]
        .into_iter()
        .filter(|r| !r.is_empty())
        .collect()
    }

    #[must_use]
    pub fn carries(&self, typ: &TreeType) -> i32 {
        match typ {
            TreeType::Plum => self.carry_plum,
            TreeType::Lemon => self.carry_lemon,
            TreeType::Apple => self.carry_apple,
            TreeType::Banana => self.carry_banana,
        }
    }

    /// True if carrying anything at all (fruits, iron, or wood)
    #[must_use]
    pub fn has_cargo(&self) -> bool {
        self.total_carried() > 0
    }

    pub fn add_carried(&mut self, typ: &TreeType, amount: i32) {
        match typ {
            TreeType::Plum => self.carry_plum += amount,
            TreeType::Lemon => self.carry_lemon += amount,
            TreeType::Apple => self.carry_apple += amount,
            TreeType::Banana => self.carry_banana += amount,
        }
    }

    pub fn remove_carried(&mut self, typ: &TreeType, amount: i32) {
        self.add_carried(typ, -amount);
    }

    pub fn clear_carried(&mut self) {
        self.carry_plum = 0;
        self.carry_lemon = 0;
        self.carry_apple = 0;
        self.carry_banana = 0;
        self.carry_iron = 0;
        self.carry_wood = 0;
    }
}
