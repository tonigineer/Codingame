use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::time::Instant;

use itertools::Itertools;

const MAX_GRADE_DISTANCE: u32 = 4;
const GRANADE_DAMAGE: u32 = 30;

const GRENADE_OPP_AGENT_MULTIPLIER: f32 = 2.5;
const GRENADE_SURE_HIT_BONUS: f32 = 2.5;
const GRENADE_POSSIBLE_MISS_MALUS: f32 = -5.0;

const INC_POS_DMG_MULT: f32 = 0.25;
const INC_POS_DMG_DIST: i32 = 1; // 1 means, opp move towards you
// const INCOMING_POSSIBLE_DAMAGE_MULTIPLIER: f32 = 0.25;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Game {
    my_id: usize,
    agent_count: usize,
    my_agent_count: usize,
    agents: HashMap<usize, Agent>,
    width: i32,
    height: i32,
    grid: Vec<Vec<Tile>>,
    my_points: i32,
    opp_points: i32,
    my_hp: i32,
    opp_hp: i32,
    my_bombs: i32,
    opp_bombs: i32,
    turn: i8,
}

impl Game {
    fn new() -> Self {
        let mut input_line = String::new();

        io::stdin().read_line(&mut input_line).unwrap();
        let my_id = parse_input!(input_line, usize); // Your player id (0 or 1)

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let agent_count = parse_input!(input_line, i32); // Total number of agents in the game

        let mut agents = HashMap::new();
        for _ in 0..agent_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();

            let agent = Agent::new(&input_line);
            agents.insert(agent.agent_id, agent);
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs: Vec<&str> = input_line.split_whitespace().collect();

        let width = parse_input!(inputs[0], i32); // Width of the game map
        let height = parse_input!(inputs[1], i32); // Height of the game map

        let mut grid: Vec<Vec<Tile>> = Vec::new();

        for _ in 0..height as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split_whitespace().collect::<Vec<_>>();

            let row: Vec<Tile> = inputs
                .chunks_exact(3)
                .take(width as usize)
                .map(|chunk| {
                    let x = parse_input!(chunk[0], i32);
                    let y = parse_input!(chunk[1], i32);
                    let tile_type = parse_input!(chunk[2], i32);
                    Tile::new(x, y, tile_type)
                })
                .collect();
            grid.push(row);
        }

        Game {
            my_id: my_id as usize,
            agent_count: agent_count as usize,
            my_agent_count: agent_count as usize / 2,
            agents,
            width,
            height,
            grid,
            my_points: 0,
            opp_points: 0,
            my_hp: 0,
            opp_hp: 0,
            my_bombs: 0,
            opp_bombs: 0,
            turn: -1, // turn one is 0 and corresponds to animation on website
        }
    }

    fn update_turn(&mut self) {
        self.turn += 1;

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();

        self.agent_count = parse_input!(input_line, usize);

        for _ in 0..self.agent_count {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs: Vec<&str> = input_line.split_whitespace().collect();

            let agent_id = parse_input!(inputs[0], usize);

            if let Some(agent) = self.agents.get_mut(&agent_id) {
                agent
                    .position
                    .change(parse_input!(inputs[1], i32), parse_input!(inputs[2], i32));
                agent.shoot_cooldown = parse_input!(inputs[3], i32); // Number of turns before this agent can shoot
                agent.splash_bombs = parse_input!(inputs[4], i32);
                agent.wetness = parse_input!(inputs[5], i32); // Damage (0-100) this agent has taken
                agent.alive = true;

                self.grid[agent.position.y as usize][agent.position.x as usize].agent =
                    Some(*agent);
            }
        }

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        self.my_agent_count = parse_input!(input_line, usize);

        self.agents.retain(|_, agent| agent.alive);

        self.my_hp = self
            .agents
            .iter()
            .filter(|(_, v)| v.player == self.my_id)
            .map(|(_, a)| a.wetness)
            .sum::<i32>();
        self.opp_hp = self
            .agents
            .iter()
            .filter(|(_, v)| v.player != self.my_id)
            .map(|(_, a)| a.wetness)
            .sum::<i32>();
        self.my_bombs = self
            .agents
            .iter()
            .filter(|(_, v)| v.player == self.my_id)
            .map(|(_, a)| a.splash_bombs)
            .sum::<i32>();
        self.opp_bombs = self
            .agents
            .iter()
            .filter(|(_, v)| v.player != self.my_id)
            .map(|(_, a)| a.splash_bombs)
            .sum::<i32>();

        self.update_points();

        eprintln!(
            "Turn: {} -- Points: {}/{}",
            self.turn, self.my_points, self.opp_points
        );
        eprintln!("DMG taken: {}/{}", self.my_hp, self.opp_hp);
        eprintln!("Bombs: {}/{}", self.my_bombs, self.opp_bombs);
    }

    fn reset_turn(&mut self) {
        self.agents.values_mut().for_each(|v| v.alive = false);

        self.grid.iter_mut().flatten().for_each(|t| {
            t.agent = None;
            t.possible_damage = 0;
        });
    }

    fn update_points(&mut self) {
        let (my_area, opp_area) = self.controlled_area();
        self.my_points += (my_area - opp_area).max(0);
        self.opp_points += (opp_area - my_area).max(0);
    }

    fn controlled_area(&self) -> (i32, i32) {
        let mut my_points = 0;
        let mut opp_points = 0;

        for r in 0..self.height {
            for c in 0..self.width {
                let tile = self.grid[r as usize][c as usize];

                let mut my_dist: i32 = 1000;
                let mut opp_dist: i32 = 1000;

                for agent in self.agents.values() {
                    let mut dist = tile.position.distance_to(&agent.position);
                    if agent.wetness >= 50 {
                        dist *= 2;
                    }

                    if agent.player == self.my_id {
                        my_dist = my_dist.min(dist as i32);
                    } else {
                        opp_dist = opp_dist.min(dist as i32);
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

    fn check_cover(&self, pos_cover: &Position, pos_attack: &Position) -> f32 {
        let dy = (pos_attack.y - pos_cover.y).signum();
        let ny = pos_cover.y + dy;
        let nx = pos_cover.x;

        let reduction = self.grid[ny as usize][nx as usize]
            .tile_type
            .damage_multiplier();

        let dx = (pos_attack.x - pos_cover.x).signum();
        let ny = pos_cover.y;
        let nx = pos_cover.x + dx;

        reduction.min(
            self.grid[ny as usize][nx as usize]
                .tile_type
                .damage_multiplier(),
        )
    }

    fn enemy_center(&self) -> Position {
        let (sx, sy, n) = self
            .agents
            .values()
            .filter(|a| a.player != self.my_id)
            .fold((0i32, 0i32, 0i32), |(sx, sy, n), a| {
                (sx + a.position.x as i32, sy + a.position.y as i32, n + 1)
            });

        if n == 0 {
            return Position::new(self.width / 2, self.height / 2);
        }

        Position::new((sx / n) as i32, (sy / n) as i32)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Bot {
    my_agents: HashSet<Agent>,
    opp_agents: HashSet<Agent>,
    moves: HashMap<Agent, Vec<(f32, Command, Command)>>,
}

impl Bot {
    fn new(game: &Game) -> Self {
        let my_agents: HashSet<Agent> = game
            .agents
            .iter()
            .filter(|(_, a)| a.player == game.my_id && a.alive)
            .map(|(_, agent)| (agent.clone()))
            .collect();

        let opp_agents: HashSet<Agent> = game
            .agents
            .iter()
            .filter(|(_, a)| a.player != game.my_id && a.alive)
            .map(|(_, agent)| (agent.clone()))
            .collect();

        let moves = HashMap::with_capacity(game.my_agent_count);

        Bot {
            my_agents,
            opp_agents,
            moves,
        }
    }

    fn send_commands(&mut self) {
        for (agent, moves) in self.moves.iter_mut() {
            moves.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));

            // if agent.agent_id == 6 {
            //     moves.iter().for_each(|m| eprintln!("{:?}", m));
            // }

            if let Some((_, c1, c2)) = moves.first() {
                let message = format!("{}{}", c1, c2)
                    .replace("; ", "")
                    .replace("MOVE ", "M")
                    .replace("SHOOT ", "S")
                    .replace("THROW ", "T")
                    .replace("HUNKER_DOWN", "H")
                    .replace(" ", "/");

                println!("{}{}{}; MESSAGE {}", agent.agent_id, c1, c2, message);
            } else {
                println!("{}; HUNKER_DOWN; MESSAGE WAIT???", agent.agent_id);
            }
        }
    }

    fn debug(&self, agent_ids: &Vec<usize>) {
        for (agent, moves) in self.moves.clone().iter_mut() {
            if !agent_ids.contains(&agent.agent_id) {
                continue;
            }

            moves.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
            for (score, cmd1, cmd2) in moves.iter() {
                eprintln!("{} {} {}", score, cmd1, cmd2);
            }
        }
    }

    fn decide_on_moving(&mut self, game: &Game) {
        let (my_area, opp_area) = game.controlled_area();
        let ref_score = my_area - opp_area;

        for agent in self.my_agents.iter() {
            let moves: &mut Vec<(f32, Command, Command)> = self.moves.entry(*agent).or_default();

            // Remain
            moves.push((-0.1, Command::HunkerDown, Command::None));

            // Control territory
            for dr in -1..=1 {
                for dc in -1..=1 {
                    if dr != 0 && dc != 0 {
                        continue;
                    }

                    let nr = agent.position.y + dr;
                    let nc = agent.position.x + dc;

                    if nr < 0 || nr >= game.height || nc < 0 || nc >= game.width {
                        continue;
                    }

                    let tile = game.grid[nr as usize][nc as usize];
                    if tile.tile_type != TileType::Empty {
                        continue;
                    }
                    // if tile.agent != None { continue }

                    let mut game_update = game.clone();
                    game_update
                        .agents
                        .get_mut(&agent.agent_id)
                        .unwrap()
                        .position
                        .change(nc, nr);

                    let (my_area, opp_area) = game_update.controlled_area();

                    moves.push((
                        0.0 + ((my_area - opp_area) - ref_score) as f32,
                        Command::Move { pos: tile.position },
                        Command::HunkerDown,
                    ));

                    moves.push((
                        0.0 + ((my_area - opp_area) - ref_score) as f32,
                        Command::Move { pos: tile.position },
                        Command::None,
                    ));
                }
            }

            // Move towards enemy
            if moves.iter().all(|v| v.0 <= 0.0) {
                let mut enemy_center = game.enemy_center();
                let distance = enemy_center.distance_to(&agent.position);
                let x = agent.position.x;
                let y = agent.position.y;

                for (dr, dc) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let new_position = Position::new(x + dc, y + dr);
                    if !new_position.is_valid(&game) {
                        continue;
                    }

                    let new_distance = new_position.distance_to(&enemy_center);

                    if distance > new_distance
                        && game.grid[(y + dr) as usize][(x + dc) as usize].tile_type
                            == TileType::Empty
                    {
                        enemy_center.change(x + dc, y + dr);
                        moves.push((
                            0.3,
                            Command::Move { pos: new_position },
                            Command::HunkerDown,
                        ));

                        moves.push((0.2, Command::Move { pos: new_position }, Command::None));

                        break;
                    }
                }

                let agent_in_need = self
                    .my_agents
                    .iter()
                    .min_by_key(|a| a.position.distance_to(&enemy_center))
                    .unwrap();

                moves.push((
                    0.15,
                    Command::Move {
                        pos: agent_in_need.position,
                    },
                    Command::HunkerDown,
                ));
            }

            // Consider incoming damage
            for (score, cmd_first, cmd_second) in moves.iter_mut() {
                let mut total_damage = 0.0f32;

                for opp_agent in self.opp_agents.iter() {
                    let next_position = if let Some(pos) = cmd_first.target_pos() {
                        pos
                    } else {
                        agent.position
                    };

                    let mut distance = next_position.distance_to(&opp_agent.position) as i32;
                    distance -= INC_POS_DMG_DIST;

                    if opp_agent.shoot_cooldown == 0 {
                        // shooting damage
                        let damage = if distance <= opp_agent.optimal_range {
                            opp_agent.soaking_power
                        } else if distance <= opp_agent.optimal_range * 2 {
                            opp_agent.soaking_power / 2
                        } else {
                            0
                        };

                        let mut damage_multiplier =
                            game.check_cover(&next_position, &opp_agent.position);

                        if matches!(cmd_first, Command::HunkerDown)
                            || matches!(cmd_second, Command::HunkerDown)
                        {
                            damage_multiplier -= Command::HunkerDown.damage_multiplier().unwrap();
                        }
                        total_damage +=
                            (damage as f32 / game.my_agent_count as f32) * damage_multiplier;
                        total_damage *= INC_POS_DMG_MULT;
                    }

                    // grenade damage
                    if opp_agent.splash_bombs > 0 {
                        if distance <= MAX_GRADE_DISTANCE as i32 {
                            total_damage += GRANADE_DAMAGE as f32;
                        }
                    }
                }

                let new_score = *score - total_damage;
                *score = new_score;
            }
        }
    }

    fn decide_on_grenades(&mut self, game: &Game) {
        const OFFSETS: [(i32, i32); 8] = [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ];

        for r in 1..game.height - 1 {
            for c in 1..game.width - 1 {
                let tile = game.grid[r as usize][c as usize];

                let (n_my_agents, n_opp_agents) = tile.position.surrounding_agents(&game, true);

                // enemy found and prevent friendly fire
                if n_my_agents > 0 || n_opp_agents == 0 {
                    continue;
                }

                for agent in self.my_agents.iter() {
                    if agent.splash_bombs == 0 {
                        continue;
                    }

                    let moves: &mut Vec<(f32, Command, Command)> =
                        self.moves.entry(*agent).or_default();

                    'next_move: for (score, cmd_1, cmd_2) in moves.clone().iter() {
                        if !matches!(cmd_2, Command::None) {
                            continue;
                        }

                        let new_position = if let Some(pos) = cmd_1.target_pos() {
                            pos
                        } else {
                            continue;
                        };

                        let distance = new_position.distance_to(&tile.position);
                        if distance > MAX_GRADE_DISTANCE {
                            continue;
                        }

                        // skip if agent after move within splash radius
                        for (dy, dx) in OFFSETS {
                            if new_position.x == tile.position.x + dx
                                && new_position.y == tile.position.y + dy
                            {
                                continue 'next_move;
                            }
                        }

                        let mut new_score =
                            score + n_opp_agents as f32 * GRENADE_OPP_AGENT_MULTIPLIER;
                        if tile.agent != None {
                            new_score += GRENADE_SURE_HIT_BONUS;
                        } else {
                            if n_opp_agents == 1 {
                                new_score += GRENADE_POSSIBLE_MISS_MALUS;
                            }
                        }

                        moves.push((new_score, *cmd_1, Command::Throw { pos: tile.position }));
                    }
                }
            }
        }
    }

    fn decide_on_shooting(&mut self, game: &Game) {
        // Opp can be eliminated
        for opp_agent in self.opp_agents.iter() {
            let mut damage_taken = opp_agent.wetness as f32;

            for agent in self.my_agents.iter() {
                if agent.shoot_cooldown > 0 {
                    continue;
                }

                let mut max_damage: f32 = 0.0;

                let moves: &mut Vec<(f32, Command, Command)> =
                    self.moves.entry(*agent).or_default();
                for (_, cmd_1, _) in moves.iter() {
                    let new_position = if let Some(pos) = cmd_1.target_pos() {
                        pos
                    } else {
                        agent.position
                    };

                    let damage_multiplier =
                        game.check_cover(&new_position, &opp_agent.position) - 0.25;
                    max_damage = max_damage
                        .max(agent.damage(&opp_agent.position) as f32 * damage_multiplier);
                }

                damage_taken += max_damage;
            }

            if damage_taken >= 100.0 {
                eprintln!("KILLLLLLLLLLLLLLLLLLLLLLLLLLLLLL");
                for agent in self.my_agents.iter() {
                    if agent.shoot_cooldown > 0 {
                        continue;
                    }

                    let mut max_damage: f32 = 0.0;

                    let moves: &mut Vec<(f32, Command, Command)> =
                        self.moves.entry(*agent).or_default();
                    let mut cmd: (f32, Command, Command) =
                        (0.0, Command::HunkerDown, Command::HunkerDown);

                    for (_, cmd_1, _) in moves.iter() {
                        let new_position = if let Some(pos) = cmd_1.target_pos() {
                            pos
                        } else {
                            agent.position
                        };

                        let damage_multiplier =
                            game.check_cover(&new_position, &opp_agent.position);
                        // max_damage = agent.damage(&opp_agent.position) as f32 * damage_multiplier;
                        if agent.damage(&opp_agent.position) as f32 * damage_multiplier > max_damage
                        {
                            max_damage = max_damage
                                + agent.damage(&opp_agent.position) as f32 * damage_multiplier;
                            cmd = (10.0, *cmd_1, Command::Shoot { agent: *opp_agent })
                        }
                    }

                    moves.push(cmd);

                    damage_taken += max_damage;

                    if damage_taken >= 100.0 {
                        break;
                    }
                }
            }
        }

        // Choose be target
        for agent in self.my_agents.iter() {
            if agent.shoot_cooldown > 0 {
                continue;
            }

            let moves: &mut Vec<(f32, Command, Command)> = self.moves.entry(*agent).or_default();

            for (score, cmd_1, _) in moves.clone() {
                for opp_agent in self.opp_agents.iter() {
                    let new_position = if let Some(pos) = cmd_1.target_pos() {
                        pos
                    } else {
                        agent.position
                    };

                    let mut new_agent = agent.clone();
                    new_agent.position = new_position;

                    let damage = new_agent.damage(&opp_agent.position);
                    let damage_multiplier =
                        game.check_cover(&opp_agent.position, &new_agent.position);

                    moves.push((
                        score + damage as f32 * damage_multiplier,
                        cmd_1,
                        Command::Shoot { agent: *opp_agent },
                    ))
                }
            }
        }
    }
}

macro_rules! timeit {
    ($label:expr, $expr:expr) => {{
        let __start = Instant::now();
        let __ret = $expr;
        eprintln!(
            "{:<15} {:>6.2} ms",
            $label,
            __start.elapsed().as_secs_f64() * 1_000.0
        );
        __ret
    }};
}

fn main() {
    let agent_ids = Vec::from([1]);
    let mut game = Game::new();

    loop {
        game.update_turn();
        let start = Instant::now();
        let mut iterations = 0;
        let mut bot = Bot::new(&game);
        loop {
            let new_game = game.clone();
            let mut new_bot = bot.clone();
            // timeit!("decide moves", new_bot.decide_on_moving(&new_game));
            // timeit!("decide grenades", new_bot.decide_on_grenades(&new_game));
            // timeit!("decide shooting", new_bot.decide_on_shooting(&new_game));

            new_bot.decide_on_moving(&new_game);
            new_bot.decide_on_grenades(&new_game);
            new_bot.decide_on_shooting(&new_game);

            let now = Instant::now();
            if now.duration_since(start).as_millis() > 45 {
                eprintln!("{} {}ms", iterations, now.duration_since(start).as_millis());
                bot = new_bot.clone();
                break;
            }
            iterations += 1;
            eprintln!("{} {}", iterations, now.duration_since(start).as_millis());
        }
        // bot.debug(&agent_ids);
        bot.send_commands();
        game.reset_turn();
    }
}
