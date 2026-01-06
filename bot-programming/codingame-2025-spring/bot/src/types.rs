use std::fmt;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub enum PlayingSide {
    Player,
    Opponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    Empty,
    LowCover,
    HighCover,
}

impl TileType {
    pub const fn damage_multiplier(self) -> f32 {
        match self {
            TileType::Empty => 1.0,
            TileType::LowCover => 0.50,
            TileType::HighCover => 0.25,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Position { x, y }
    }

    pub fn change(&mut self, x: i16, y: i16) {
        self.x = x;
        self.y = y;
    }

    // fn is_valid(&self, game: &Game) -> bool {
    //     return self.x >= 0 && self.x < game.width && self.y >= 0 && self.y < game.height;
    // }

    pub fn distance_to(&self, other: &Position) -> i16 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as i16
    }

    pub fn cardinal_dirs(&self) -> Vec<Position> {
        const CARDINAL_DIRS: [(i16, i16); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        CARDINAL_DIRS
            .iter()
            .map(|(dx, dy)| Position::new(self.x + dx, self.y + dy))
            .collect()
    }

    pub fn moore_dirs(&self) -> Vec<Position> {
        const MOORE_DIRS: [(i16, i16); 8] = [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ];
        MOORE_DIRS
            .iter()
            .map(|(dx, dy)| Position::new(self.x + dx, self.y + dy))
            .collect()
    }

    // fn surrounding_agents(&self, game: &Game, incl_center: bool) -> (u32, u32) {
    //     let mut num_my_agents = 0;
    //     let mut num_opp_agents = 0;
    //     for dr in -1..=1 {
    //         for dc in -1..=1 {
    //             if !incl_center && dr == 0 && dc == 0 {
    //                 continue;
    //             }

    //             if self.y + dr < 0
    //                 || self.y + dr >= game.height
    //                 || self.x + dc < 0
    //                 || self.x + dc >= game.width
    //             {
    //                 continue;
    //             }

    //             if let Some(agent) = game.grid[(self.y + dr) as usize][(self.x + dc) as usize].agent
    //             {
    //                 if agent.player == game.my_id {
    //                     num_my_agents += 1;
    //                 }
    //                 if agent.player != game.my_id {
    //                     num_opp_agents += 1;
    //                 }
    //             }
    //         }
    //     }

    //     (num_my_agents, num_opp_agents)
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    pub position: Position,
    pub tile_type: TileType,
    pub agent: Option<Agent>,
}

impl Tile {
    pub fn new(x: i16, y: i16, tile_type: u8) -> Self {
        Tile {
            position: Position::new(x, y),
            tile_type: match tile_type {
                0 => TileType::Empty,
                1 => TileType::LowCover,
                2 => TileType::HighCover,
                _ => unreachable!(),
            },
            agent: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Agent {
    pub agent_id: u8,
    pub player_id: u8,
    pub shoot_cooldown: i16,
    pub optimal_range: i16,
    pub soaking_power: i16,
    pub splash_bombs: i16,
    pub position: Position,
    pub wetness: i16,
    pub alive: bool,
}

impl Agent {
    pub fn new(input_line: &str) -> Self {
        let inputs: Vec<&str> = input_line.split_whitespace().collect();

        let agent_id = parse_input!(inputs[0], u8);
        let player_id = parse_input!(inputs[1], u8);
        let shoot_cooldown = parse_input!(inputs[2], i16);
        let optimal_range = parse_input!(inputs[3], i16);
        let soaking_power = parse_input!(inputs[4], i16);
        let splash_bombs = parse_input!(inputs[5], i16);

        Agent {
            agent_id,
            player_id,
            shoot_cooldown,
            optimal_range,
            soaking_power,
            splash_bombs,
            position: Position::new(0, 0),
            wetness: 0,
            alive: true,
        }
    }

    pub fn attack_damage(&self, target_position: &Position) -> i16 {
        if self.shoot_cooldown > 0 {
            return 0;
        }

        let distance = self.position.distance_to(&target_position);

        if distance <= self.optimal_range {
            return self.soaking_power;
        }
        if distance <= self.optimal_range * 2 {
            return self.soaking_power / 2;
        }

        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    Move { position: Position },
    Throw { position: Position },
    Shoot { agent: Agent },
    HunkerDown,
    None,
}

impl Command {
    //     fn damage_multiplier(self) -> Option<f32> {
    //         match self {
    //             Command::HunkerDown => Some(0.25),
    //             _ => None,
    //         }
    //     }

    pub fn position(self) -> Option<Position> {
        match self {
            Command::Move { position } | Command::Throw { position } => Some(position),
            _ => None,
        }
    }

    //     fn actor(self) -> Option<Agent> {
    //         if let Command::Shoot { agent } = self {
    //             Some(agent)
    //         } else {
    //             None
    //         }
    //     }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Move { position } => write!(f, "; MOVE {} {}", position.x, position.y),
            Command::Throw { position } => write!(f, "; THROW {} {}", position.x, position.y),
            Command::Shoot { agent } => write!(f, "; SHOOT {}", agent.agent_id),
            Command::HunkerDown => write!(f, "; HUNKER_DOWN"),
            Command::None => write!(f, ""),
        }
    }
}
