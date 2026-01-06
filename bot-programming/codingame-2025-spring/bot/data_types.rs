mod game;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TileType {
    Empty,
    LowCover,
    HighCover,
}

impl TileType {
    const fn damage_multiplier(self) -> f32 {
        match self {
            TileType::Empty => 1.0,
            TileType::LowCover => 0.50,
            TileType::HighCover => 0.25,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    fn change(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    fn is_valid(&self, game: &Game) -> bool {
        return self.x >= 0 && self.x < game.width && self.y >= 0 && self.y < game.height;
    }

    fn distance_to(&self, other: &Position) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    fn surrounding_agents(&self, game: &Game, incl_center: bool) -> (u32, u32) {
        let mut num_my_agents = 0;
        let mut num_opp_agents = 0;
        for dr in -1..=1 {
            for dc in -1..=1 {
                if !incl_center && dr == 0 && dc == 0 {
                    continue;
                }

                if self.y + dr < 0
                    || self.y + dr >= game.height
                    || self.x + dc < 0
                    || self.x + dc >= game.width
                {
                    continue;
                }

                if let Some(agent) = game.grid[(self.y + dr) as usize][(self.x + dc) as usize].agent
                {
                    if agent.player == game.my_id {
                        num_my_agents += 1;
                    }
                    if agent.player != game.my_id {
                        num_opp_agents += 1;
                    }
                }
            }
        }

        (num_my_agents, num_opp_agents)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Tile {
    position: Position,
    tile_type: TileType,
    agent: Option<Agent>,
    possible_damage: i32,
}

impl Tile {
    fn new(x: i32, y: i32, tile_type: i32) -> Self {
        Tile {
            position: Position::new(x, y),
            tile_type: match tile_type {
                0 => TileType::Empty,
                1 => TileType::LowCover,
                2 => TileType::HighCover,
                _ => unreachable!(),
            },
            agent: None,
            possible_damage: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Agent {
    agent_id: usize,
    player: usize,
    shoot_cooldown: i32,
    optimal_range: i32,
    soaking_power: i32,
    splash_bombs: i32,
    position: Position,
    wetness: i32,
    alive: bool,
}

impl Agent {
    fn new(input_line: &str) -> Self {
        let inputs: Vec<&str> = input_line.split_whitespace().collect();

        let agent_id = parse_input!(inputs[0], i32); // Unique identifier for this agent
        let player = parse_input!(inputs[1], i32); // Player id of this agent
        let shoot_cooldown = parse_input!(inputs[2], i32); // Number of turns between each of this agent's shots
        let optimal_range = parse_input!(inputs[3], i32); // Maximum manhattan distance for greatest damage output
        let soaking_power = parse_input!(inputs[4], i32); // Damage output within optimal conditions
        let splash_bombs = parse_input!(inputs[5], i32); // Number of splash bombs this can throw this game

        Agent {
            agent_id: agent_id as usize,
            player: player as usize,
            shoot_cooldown,
            optimal_range,
            soaking_power,
            splash_bombs,
            position: Position::new(0, 0),
            wetness: 0i32,
            alive: true,
        }
    }

    fn damage(&self, position: &Position) -> i32 {
        if self.shoot_cooldown > 0 {
            return 0;
        }

        let distance = self.position.distance_to(&position) as i32;
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
enum Command {
    Move { pos: Position },
    Throw { pos: Position },
    Shoot { agent: Agent },
    HunkerDown,
    None,
}

impl Command {
    fn damage_multiplier(self) -> Option<f32> {
        match self {
            Command::HunkerDown => Some(0.25),
            _ => None,
        }
    }

    fn target_pos(self) -> Option<Position> {
        match self {
            Command::Move { pos } | Command::Throw { pos } => Some(pos),
            _ => None,
        }
    }

    fn actor(self) -> Option<Agent> {
        if let Command::Shoot { agent } = self {
            Some(agent)
        } else {
            None
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Move { pos } => write!(f, "; MOVE {} {}", pos.x, pos.y),
            Command::Throw { pos } => write!(f, "; THROW {} {}", pos.x, pos.y),
            Command::Shoot { agent } => write!(f, "; SHOOT {}", agent.agent_id),
            Command::HunkerDown => write!(f, "; HUNKER_DOWN"),
            Command::None => write!(f, ""),
        }
    }
}
