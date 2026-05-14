use crate::position::CARDINALS;
use crate::{position::Position};
use crate::{game::GameState};

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub enum Resource {
    Banana(i32),
    Plum(i32),
    Apple(i32),
    Lemon(i32),
}

impl Resource {
    pub fn from_tree(typ: &TreeType, amount: i32) -> Self {
        match typ {
            TreeType::Apple => Resource::Apple(amount),
            TreeType::Banana => Resource::Banana(amount),
            TreeType::Lemon => Resource::Lemon(amount),
            TreeType::Plum => Resource::Plum(amount),
        }
    }

    pub fn amount(&self) -> i32 {
        match self {
            Resource::Apple(n) | Resource::Banana(n) | Resource::Lemon(n) | Resource::Plum(n) => *n,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.amount() == 0
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
    #[rustfmt::skip]
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
}

#[derive(Debug)]
pub enum TreeType {
    Plum,
    Lemon,
    Apple,
    Banana,
}

impl TreeType {
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
    #[rustfmt::skip]
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
impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Troll {
    #[rustfmt::skip]
    pub fn parse(line: &str) -> Self {
        let d: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        impl Default for Game {
            fn default() -> Self {
                Self::new()
            }
        }

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

    pub fn adjacent_trees<'a>(&self, game_state: &'a GameState) -> Vec<&'a Tree> {
        game_state
            .trees
            .iter()
            .filter(|t| t.position.manhattan(&self.position) == 1)
            .collect()
    }

    pub fn would_harvest(&self, game_state: &GameState) -> Option<Vec<Resource>> {
        let harvest: Vec<_> = self
            .adjacent_trees(game_state)
            .iter()
            .filter_map(|tree| {
                eprintln!("{:?}", tree);
                let amount = self.carry_capacity.min(self.harvest_power).min(tree.fruits);
                eprintln!("{:?}", amount);
                (amount > 0).then(|| Resource::from_tree(&tree.typ, amount))
            })
            .collect();
        (!harvest.is_empty()).then_some(harvest)
    }

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

    pub fn is_adjacent_to_shack(&self, game_state: &GameState) -> bool {
        self.position.manhattan(&game_state.my_shack) == 1
    }

    pub fn would_drop(&self, game_state: &GameState) -> Option<Vec<Resource>> {
        self.is_adjacent_to_shack(game_state)
            .then(|| self.carried_resources())
    }

    pub fn reachable_positions(&self, game_state: &GameState) -> Option<Vec<Position>> {
        let moves: Vec<_> = crate::position::CARDINALS
            .iter()
            .map(|&c| self.position + c)
            .filter(|p| game_state.grid[*p] == b'.')
            .collect();
        (!moves.is_empty()).then_some(moves)
    }
}
