use crate::game::Side;
use crate::utils::Position;
use std::ops::AddAssign;

use std::collections::HashMap;

// ------------------------------------------------------------------------
// Resource / Inventory
// ------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Plum,
    Lemon,
    Apple,
    Banana,
    Iron,
    Wood,
}

#[allow(dead_code)]
impl ResourceType {
    #[must_use]
    pub fn from_tree(typ: TreeType) -> Self {
        match typ {
            TreeType::Plum => ResourceType::Plum,
            TreeType::Lemon => ResourceType::Lemon,
            TreeType::Apple => ResourceType::Apple,
            TreeType::Banana => ResourceType::Banana,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Resource {
    pub typ: ResourceType,
    pub amount: i32,
}

#[allow(dead_code)]
impl Resource {
    #[must_use]
    pub fn new(typ: ResourceType, amount: i32) -> Self {
        Self { typ, amount }
    }

    #[must_use]
    pub fn from_tree(typ: TreeType, amount: i32) -> Self {
        Self::new(ResourceType::from_tree(typ), amount)
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.amount == 0
    }
}

impl AddAssign for Resource {
    fn add_assign(&mut self, other: Self) {
        assert!(
            self.typ == other.typ,
            "Cannot add different resource types: {:?} += {:?}",
            self.typ,
            other.typ
        );
        self.amount += other.amount;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Inventory {
    pub plum: Resource,
    pub lemon: Resource,
    pub apple: Resource,
    pub banana: Resource,
    pub iron: Resource,
    pub wood: Resource,
}

#[allow(dead_code)]
impl Inventory {
    #[rustfmt::skip]
    #[must_use]
    pub fn new() -> Self {
        Self {
            plum:   Resource::new(ResourceType::Plum, 0),
            lemon:  Resource::new(ResourceType::Lemon, 0),
            apple:  Resource::new(ResourceType::Apple, 0),
            banana: Resource::new(ResourceType::Banana, 0),
            iron:   Resource::new(ResourceType::Iron, 0),
            wood:   Resource::new(ResourceType::Wood, 0),
        }
    }

    #[rustfmt::skip]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn parse(line: &str) -> Self {
        let r: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        Self {
            plum:   Resource::new(ResourceType::Plum, r[0]),
            lemon:  Resource::new(ResourceType::Lemon, r[1]),
            apple:  Resource::new(ResourceType::Apple, r[2]),
            banana: Resource::new(ResourceType::Banana, r[3]),
            iron:   Resource::new(ResourceType::Iron, r[4]),
            wood:   Resource::new(ResourceType::Wood, r[5]),
        }
    }

    pub fn add(&mut self, resource: Resource) {
        self.get_mut(resource.typ).amount += resource.amount;
    }

    pub fn remove(&mut self, resource: Resource) {
        self.get_mut(resource.typ).amount -= resource.amount;
    }

    #[must_use]
    pub fn get(&self, typ: ResourceType) -> i32 {
        match typ {
            ResourceType::Plum => self.plum.amount,
            ResourceType::Lemon => self.lemon.amount,
            ResourceType::Apple => self.apple.amount,
            ResourceType::Banana => self.banana.amount,
            ResourceType::Iron => self.iron.amount,
            ResourceType::Wood => self.wood.amount,
        }
    }

    pub fn get_mut(&mut self, typ: ResourceType) -> &mut Resource {
        match typ {
            ResourceType::Plum => &mut self.plum,
            ResourceType::Lemon => &mut self.lemon,
            ResourceType::Apple => &mut self.apple,
            ResourceType::Banana => &mut self.banana,
            ResourceType::Iron => &mut self.iron,
            ResourceType::Wood => &mut self.wood,
        }
    }

    #[must_use]
    pub fn get_by_tree(&self, typ: TreeType) -> i32 {
        self.get(ResourceType::from_tree(typ))
    }

    /// Score: each fruit = 1 point, wood = 4 points, iron = 0
    #[must_use]
    pub fn score(&self) -> i32 {
        self.plum.amount
            + self.lemon.amount
            + self.apple.amount
            + self.banana.amount
            + self.wood.amount * 4
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

#[allow(dead_code)]
impl TreeType {
    #[must_use]
    pub fn to_byte(self) -> u8 {
        match self {
            TreeType::Apple => b'A',
            TreeType::Banana => b'B',
            TreeType::Lemon => b'L',
            TreeType::Plum => b'P',
        }
    }
}

#[allow(dead_code)]
impl TreeType {
    #[must_use]
    pub fn to_str(self) -> &'static str {
        match self {
            TreeType::Plum => "PLUM",
            TreeType::Lemon => "LEMON",
            TreeType::Apple => "APPLE",
            TreeType::Banana => "BANANA",
        }
    }

    #[must_use]
    pub fn as_resource_type(self) -> ResourceType {
        match self {
            TreeType::Apple => ResourceType::Apple,
            TreeType::Banana => ResourceType::Banana,
            TreeType::Lemon => ResourceType::Lemon,
            TreeType::Plum => ResourceType::Plum,
        }
    }
}

impl std::str::FromStr for TreeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PLUM" => Ok(TreeType::Plum),
            "LEMON" => Ok(TreeType::Lemon),
            "APPLE" => Ok(TreeType::Apple),
            "BANANA" => Ok(TreeType::Banana),
            _ => Err(format!("Unknown tree type: {s}")),
        }
    }
}

impl std::fmt::Display for TreeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", (*self).to_str())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Tree {
    pub typ: TreeType,
    pub position: Position,
    pub size: i32,
    pub health: i32,
    pub fruits: i32,
    pub cooldown: i32,
}

#[allow(dead_code)]
impl Tree {
    #[rustfmt::skip]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn parse(line: &str) -> Self {
        let d: Vec<&str> = line.split_whitespace().collect();
        Self {
            typ:        d[0].parse().unwrap(),
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
            TreeType::Plum | TreeType::Lemon => 8,
            TreeType::Apple => 9,
            TreeType::Banana => 6,
        }
    }

    /// Cooldown when adjacent to water
    #[must_use]
    pub fn cooldown_time_water(&self) -> i32 {
        match self.typ {
            TreeType::Plum | TreeType::Lemon => 3,
            TreeType::Apple => 2,
            TreeType::Banana => 4,
        }
    }

    #[must_use]
    pub fn initial_cooldown(typ: TreeType) -> i32 {
        match typ {
            TreeType::Plum | TreeType::Lemon => 8,
            TreeType::Apple => 9,
            TreeType::Banana => 6,
        }
    }

    #[must_use]
    pub fn initial_cooldown_water(typ: TreeType) -> i32 {
        match typ {
            TreeType::Plum | TreeType::Lemon => 3,
            TreeType::Apple => 2,
            TreeType::Banana => 4,
        }
    }

    /// Health for a given type and size
    #[rustfmt::skip]
    #[must_use]
    pub fn max_health(typ: TreeType, size: i32) -> i32 {
          match (typ, size) {
              (TreeType::Plum | TreeType::Lemon, 1) | (TreeType::Banana, 4) => 6,
              (TreeType::Plum | TreeType::Lemon, 2) => 8,
              (TreeType::Plum | TreeType::Lemon, 4) => 12,
             (TreeType::Apple,  1) => 11,
             (TreeType::Apple,  2) => 14,
             (TreeType::Apple,  3) => 17,
             (TreeType::Apple,  4) => 20,
             (TreeType::Banana, 1) => 3,
             (TreeType::Banana, 2) => 4,
             (TreeType::Banana, 3) => 5,
             _ => 10,
         }
    }

    #[must_use]
    pub fn get_resource_type(&self) -> ResourceType {
        match self.typ {
            TreeType::Apple => ResourceType::Apple,
            TreeType::Banana => ResourceType::Banana,
            TreeType::Lemon => ResourceType::Lemon,
            TreeType::Plum => ResourceType::Plum,
        }
    }
}

// ------------------------------------------------------------------------
// Troll
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    pub dist_map: HashMap<Position, (i32, Position)>,
}

#[allow(dead_code)]
impl Troll {
    #[rustfmt::skip]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
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
            dist_map:       HashMap::new()
        }
    }

    #[must_use]
    pub fn total_carried(&self) -> i32 {
        self.carry_plum
            + self.carry_lemon
            + self.carry_apple
            + self.carry_banana
            + self.carry_iron
            + self.carry_wood
    }

    #[must_use]
    pub fn free_capacity(&self) -> i32 {
        self.carry_capacity - self.total_carried()
    }

    #[must_use]
    pub fn carried_resources(&self) -> Vec<Resource> {
        [
            Resource::new(ResourceType::Plum, self.carry_plum),
            Resource::new(ResourceType::Lemon, self.carry_lemon),
            Resource::new(ResourceType::Apple, self.carry_apple),
            Resource::new(ResourceType::Banana, self.carry_banana),
            Resource::new(ResourceType::Iron, self.carry_iron),
            Resource::new(ResourceType::Wood, self.carry_wood),
        ]
        .into_iter()
        .filter(|r| !r.is_empty())
        .collect()
    }

    #[must_use]
    pub fn carries(&self, typ: TreeType) -> i32 {
        self.carries_resource(ResourceType::from_tree(typ))
    }

    #[must_use]
    pub fn carries_resource(&self, typ: ResourceType) -> i32 {
        match typ {
            ResourceType::Plum => self.carry_plum,
            ResourceType::Lemon => self.carry_lemon,
            ResourceType::Apple => self.carry_apple,
            ResourceType::Banana => self.carry_banana,
            ResourceType::Iron => self.carry_iron,
            ResourceType::Wood => self.carry_wood,
        }
    }

    /// True if carrying anything at all
    #[must_use]
    pub fn has_cargo(&self) -> bool {
        self.total_carried() > 0
    }

    pub fn add_carried(&mut self, typ: ResourceType, amount: i32) {
        match typ {
            ResourceType::Plum => self.carry_plum += amount,
            ResourceType::Lemon => self.carry_lemon += amount,
            ResourceType::Apple => self.carry_apple += amount,
            ResourceType::Banana => self.carry_banana += amount,
            ResourceType::Iron => self.carry_iron += amount,
            ResourceType::Wood => self.carry_wood += amount,
        }
    }

    pub fn remove_carried(&mut self, typ: ResourceType, amount: i32) {
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
