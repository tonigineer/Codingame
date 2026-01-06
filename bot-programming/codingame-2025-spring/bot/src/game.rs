use std::collections::HashMap;
use std::io;

pub use crate::types::{Agent, Command, Position, Tile};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub const MAX_GRADE_DISTANCE: i16 = 4;
pub const GRANADE_DAMAGE: i16 = 30;

pub const WETNESS_THRESHOLD_AREA_CONTROL: i16 = 50;
pub const WETNESS_AREA_CONTROL_DIST_MULT: i16 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub my_id: u8,
    pub width: u8,
    pub height: u8,
    pub grid: Vec<Vec<Tile>>,
    pub init_agent_count: usize,
    pub agents: HashMap<u8, Agent>,
    pub my_points: i16,
    pub opp_points: i16,
    pub my_hp: i16,
    pub opp_hp: i16,
    pub my_bombs: i16,
    pub opp_bombs: i16,
    pub turn: i16,
}

impl Game {
    pub fn new() -> Self {
        let mut input_line = String::new();

        io::stdin().read_line(&mut input_line).unwrap();
        let my_id = parse_input!(input_line, u8);

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let init_agent_count = parse_input!(input_line, usize);

        let mut agents = HashMap::new();
        for _ in 0..init_agent_count {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();

            let agent = Agent::new(&input_line);
            agents.insert(agent.agent_id, agent);
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs: Vec<&str> = input_line.split_whitespace().collect();

        let width = parse_input!(inputs[0], u8); // Width of the game map
        let height = parse_input!(inputs[1], u8); // Height of the game map

        let mut grid: Vec<Vec<Tile>> = Vec::new();

        for _ in 0..height as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split_whitespace().collect::<Vec<_>>();

            let row: Vec<Tile> = inputs
                .chunks_exact(3)
                .take(width as usize)
                .map(|chunk| {
                    let x = parse_input!(chunk[0], i16);
                    let y = parse_input!(chunk[1], i16);
                    let tile_type = parse_input!(chunk[2], u8);
                    Tile::new(x, y, tile_type)
                })
                .collect();
            grid.push(row);
        }

        Game {
            my_id,
            width,
            height,
            grid,
            init_agent_count,
            agents,
            my_points: 0,
            opp_points: 0,
            my_hp: 0,
            opp_hp: 0,
            my_bombs: 0,
            opp_bombs: 0,
            turn: -1, // turn one is 0 and corresponds to animation on website
        }
    }

    pub fn update_turn(&mut self) {
        // Reset previous turn
        self.agents.values_mut().for_each(|v| v.alive = false);
        self.grid.iter_mut().flatten().for_each(|t| {
            t.agent = None;
        });

        // Parse new turn
        self.turn += 1;

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();

        for _ in 0..parse_input!(input_line, usize) {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs: Vec<&str> = input_line.split_whitespace().collect();

            let agent_id = parse_input!(inputs[0], u8);

            if let Some(agent) = self.agents.get_mut(&agent_id) {
                agent
                    .position
                    .change(parse_input!(inputs[1], i16), parse_input!(inputs[2], i16));
                agent.shoot_cooldown = parse_input!(inputs[3], i16); // Number of turns before this agent can shoot
                agent.splash_bombs = parse_input!(inputs[4], i16);
                agent.wetness = parse_input!(inputs[5], i16); // Damage (0-100) this agent has taken
                agent.alive = true;

                self.grid[agent.position.y as usize][agent.position.x as usize].agent =
                    Some(*agent);
            }
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let my_agent_count = parse_input!(input_line, usize);

        self.agents.retain(|_, agent| agent.alive);
        assert!(
            my_agent_count
                == self
                    .agents
                    .iter()
                    .filter(|(_, v)| v.player_id == self.my_id)
                    .count(),
        );

        self.my_hp = self
            .agents
            .iter()
            .filter(|(_, v)| v.player_id == self.my_id)
            .map(|(_, a)| a.wetness)
            .sum::<i16>();
        self.opp_hp = self
            .agents
            .iter()
            .filter(|(_, v)| v.player_id != self.my_id)
            .map(|(_, a)| a.wetness)
            .sum::<i16>();
        self.my_bombs = self
            .agents
            .iter()
            .filter(|(_, v)| v.player_id == self.my_id)
            .map(|(_, a)| a.splash_bombs)
            .sum::<i16>();
        self.opp_bombs = self
            .agents
            .iter()
            .filter(|(_, v)| v.player_id != self.my_id)
            .map(|(_, a)| a.splash_bombs)
            .sum::<i16>();

        self.update_points();

        eprintln!(
            "Turn: {} Points: {}/{} DMG: {}/{} Bombs: {}/{}",
            self.turn,
            self.my_points,
            self.opp_points,
            self.my_hp,
            self.opp_hp,
            self.my_bombs,
            self.opp_bombs
        );
    }

    pub fn update_points(&mut self) {
        let (my_area, opp_area) = self.controlled_area();
        self.my_points += (my_area - opp_area).max(0);
        self.opp_points += (opp_area - my_area).max(0);
    }

    pub fn output_commands(&self, agent_commands: &HashMap<Agent, (Command, Command)>) {
        for (_, agent) in self
            .agents
            .iter()
            .filter(|(_, a)| a.player_id == self.my_id)
        {
            if let Some((c1, c2)) = agent_commands.get(&agent) {
                println!("{}{}{}", agent.agent_id, c1, c2);
            } else {
                println!("{}; HUNKER_DOWN; MESSAGE {}", agent.agent_id, "??");
            }
        }
    }

    pub fn controlled_area(&self) -> (i16, i16) {
        let mut my_points = 0;
        let mut opp_points = 0;

        for r in 0..self.height {
            for c in 0..self.width {
                let tile = self.grid[r as usize][c as usize];

                let mut my_dist: i16 = std::i16::MAX;
                let mut opp_dist: i16 = std::i16::MAX;

                for agent in self.agents.values() {
                    let mut dist = tile.position.distance_to(&agent.position);
                    if agent.wetness >= WETNESS_THRESHOLD_AREA_CONTROL {
                        dist *= WETNESS_AREA_CONTROL_DIST_MULT;
                    }

                    if agent.player_id == self.my_id {
                        my_dist = my_dist.min(dist);
                    } else {
                        opp_dist = opp_dist.min(dist);
                    }
                }

                if my_dist <= opp_dist {
                    my_points += 1
                }

                if opp_dist <= my_dist {
                    opp_points += 1
                }
            }
        }

        (my_points, opp_points)
    }

    pub fn is_valid(&self, position: &Position) -> bool {
        position.x >= 0
            && position.x < self.width as i16
            && position.y >= 0
            && position.y < self.height as i16
    }

    pub fn get_tile(&self, position: &Position) -> Tile {
        self.grid[position.y as usize][position.x as usize]
    }

    pub fn get_enemy_center(&self) -> Position {
        let (sx, sy, n) = self
            .agents
            .values()
            .filter(|a| a.player_id != self.my_id)
            .fold((0i32, 0i32, 0i32), |(sx, sy, n), a| {
                (sx + a.position.x as i32, sy + a.position.y as i32, n + 1)
            });

        if n == 0 {
            return Position::new((self.width / 2) as i16, (self.height / 2) as i16);
        }

        Position::new((sx / n) as i16, (sy / n) as i16)
    }

    pub fn surrounding_agents(&self, position: &Position, incl_center: bool) -> (i16, i16) {
        let mut num_my_agents = 0;
        let mut num_opp_agents = 0;

        for dr in -1..=1 {
            for dc in -1..=1 {
                if !incl_center && dr == 0 && dc == 0 {
                    continue;
                }

                let new_position = Position {
                    x: position.x + dc,
                    y: position.y + dr,
                };

                if !self.is_valid(&new_position) {
                    continue;
                }

                if let Some(agent) = self.get_tile(&new_position).agent {
                    if agent.player_id == self.my_id {
                        num_my_agents += 1;
                    }
                    if agent.player_id != self.my_id {
                        num_opp_agents += 1;
                    }
                }
            }
        }

        (num_my_agents, num_opp_agents)
    }
}
